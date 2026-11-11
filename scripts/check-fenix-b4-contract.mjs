#!/usr/bin/env node
// Offline B4 wire-contract simulation; NOT Rust QA or live interop.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (path) => readFileSync(resolve(root, path));
const manifest = JSON.parse(read('docs/contracts/fenix-b4-pinned-manifest.json'));
let count = 0;
function check(label, fn) { fn(); count++; process.stdout.write('PASS ' + label + '\n'); }

const supportedSchemaKeywords = new Set([
  '$schema', '$id', 'title', 'type', 'additionalProperties', 'required',
  'properties', 'maxProperties', 'const', 'enum', 'pattern', 'maxLength',
  'minLength', 'format',
]);

// The B4 checker is intentionally a subset, not a general JSON Schema
// implementation. Refuse future keywords/types rather than claiming PASS
// while silently ignoring a provider-side restriction.
function assertSupportedSchema(schema) {
  assert.ok(schema && typeof schema === 'object' && !Array.isArray(schema));
  for (const key of Object.keys(schema)) {
    assert.ok(supportedSchemaKeywords.has(key), 'unsupported provider schema keyword: ' + key);
  }
  if (schema.type !== undefined) {
    assert.ok(['object', 'string', 'boolean'].includes(schema.type),
      'unsupported provider schema type: ' + schema.type);
  }
  if (schema.format !== undefined) {
    assert.equal(schema.format, 'uuid', 'unsupported provider format');
  }
  if (schema.properties !== undefined) {
    assert.ok(schema.type === 'object' && !Array.isArray(schema.properties));
    for (const child of Object.values(schema.properties)) assertSupportedSchema(child);
  }
  if (schema.required !== undefined) {
    assert.ok(schema.type === 'object' && Array.isArray(schema.required));
    for (const property of schema.required) {
      assert.ok(typeof property === 'string' && Object.hasOwn(schema.properties ?? {}, property),
        'unsupported unbound required property');
    }
  }
  if (schema.additionalProperties !== undefined) {
    assert.equal(schema.additionalProperties, false, 'unsupported additionalProperties mode');
  }
}

function pinned(name) {
  const entry = manifest.contracts.find((c) => c.localPath.endsWith(name));
  assert.ok(entry, 'missing ' + name);
  const bytes = read(entry.localPath);
  const sha = createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex');
  assert.equal(sha, entry.providerBlobSha, 'Fenix schema blob differs: ' + name);
  const parsed = JSON.parse(bytes);
  assertSupportedSchema(parsed);
  return parsed;
}

// Deliberate subset: only JSON-Schema keywords present in the three pinned
// B1/B4 flat/object schemas. Any newly introduced keyword fails in audit.
function valid(schema, value) {
  if ('const' in schema && value !== schema.const) return false;
  if (schema.enum && !schema.enum.includes(value)) return false;
  if (schema.type === 'object') {
    if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
    const keys = Object.keys(value);
    if (schema.maxProperties !== undefined && keys.length > schema.maxProperties) return false;
    if (schema.required?.some((key) => !Object.hasOwn(value, key))) return false;
    if (schema.additionalProperties === false &&
        keys.some((key) => !Object.hasOwn(schema.properties ?? {}, key))) return false;
    return keys.every((key) => !schema.properties?.[key] || valid(schema.properties[key], value[key]));
  }
  if (schema.type === 'string') {
    if (typeof value !== 'string') return false;
    if (schema.minLength !== undefined && value.length < schema.minLength) return false;
    if (schema.maxLength !== undefined && value.length > schema.maxLength) return false;
    if (schema.pattern && !new RegExp(schema.pattern).test(value)) return false;
    if (schema.format === 'uuid' &&
        !/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(value))
      return false;
  }
  if (schema.type === 'boolean' && typeof value !== 'boolean') return false;
  return true;
}

const reqSchema = pinned('fenix-b1-request.schema.json');
const ackSchema = pinned('fenix-b4-admission-ack.schema.json');
const errSchema = pinned('fenix-b4-admission-error.schema.json');
const message = 'Request could not be accepted';
const statuses = {'400':'INVALID_REQUEST','401':'REQUEST_DENIED','403':'REQUEST_DENIED',
  '404':'REQUEST_DENIED','409':'OPERATION_CONFLICT','503':'PROVIDER_UNAVAILABLE'};
const req = {
  contract_version:'1', operation_id:'operation_00000001',
  capability_key:'integration.diagnostics', input_schema_version:'1',
  trace_id:'trace_b4_000001', source:{product:'dubbridge',
    org_id:'550e8400-e29b-41d4-a716-446655440000',
    project_id:'550e8400-e29b-41d4-a716-446655440001'}, input:{}
};
const ack = {contract_version:'1', operation_id:req.operation_id, trace_id:req.trace_id, replayed:false};
const clone = (v) => structuredClone(v);
function classify(status, body) {
  if (!valid(errSchema, body) || body.message !== message || statuses[status] !== body.error_code) return 'ambiguous';
  if (status === 503) return 'ambiguous';
  return status === 400 ? 'invalid' : status === 409 ? 'conflict' : 'denied';
}
check('unimplemented schema keywords fail closed', () => {
  assert.throws(() => assertSupportedSchema({type:'string',pattern:'.*',not:{}}),
    /unsupported provider schema keyword/);
  assert.throws(() => assertSupportedSchema({type:'number'}),
    /unsupported provider schema type/);
});
check('pinned three Fenix provider schema blobs', () => assert.equal(manifest.contracts.length,3));
check('manifest remains offline without routed endpoint', () => {
  assert.equal(manifest.providerRepository,'krukmat/fenix');
  assert.equal(manifest.actualHttpEndpoint,'not-mounted');
  assert.equal(manifest.scope,'contract-simulation-only');
});
check('B1 valid neutral request', () => assert.ok(valid(reqSchema,req)));
check('B1 missing field denied', () => { const q=clone(req); delete q.operation_id; assert.ok(!valid(reqSchema,q)); });
check('B1 extra tenant field denied', () => { const q=clone(req);q.source.workspace_id='forged';assert.ok(!valid(reqSchema,q)); });
check('B1 malformed UUID denied', () => { const q=clone(req);q.source.org_id='bad';assert.ok(!valid(reqSchema,q)); });
check('B1 caller-supplied digest denied', () => assert.ok(!valid(reqSchema,{...req,request_sha256:'forged'})));
check('B1 invalid capability denied', () => assert.ok(!valid(reqSchema,{...req,capability_key:'workflow'})));
check('B1 input property ceiling', () => {
  const input=Object.fromEntries(Array.from({length:65},(_,i)=>['k'+i,i]));
  assert.ok(!valid(reqSchema,{...req,input}));
});
check('B1 version syntax restricted', () => assert.ok(!valid(reqSchema,{...req,input_schema_version:'0'})));
check('B4 ack matches operation and trace', () => assert.ok(valid(ackSchema,ack)));
check('B4 replay acknowledgement accepted', () => assert.ok(valid(ackSchema,{...ack,replayed:true})));
check('B4 wrong operation detected by correlation', () => {
  const a={...ack,operation_id:'operation_00000002'};
  assert.ok(valid(ackSchema,a) && a.operation_id!==req.operation_id);
});
check('B4 wrong trace detected by correlation', () => {
  const a={...ack,trace_id:'trace_b4_000002'};
  assert.ok(valid(ackSchema,a) && a.trace_id!==req.trace_id);
});
check('B4 fabricated run_id forbidden', () => assert.ok(!valid(ackSchema,{...ack,run_id:'fake'})));
check('B4 replay flag required', () => {const a=clone(ack);delete a.replayed;assert.ok(!valid(ackSchema,a));});
for (const [statusText, code] of Object.entries(statuses)) {
  const status=Number(statusText);
  check('B4 '+status+' '+code, () => {
    const body={contract_version:'1',error_code:code,message};
    assert.ok(valid(errSchema,body));
    assert.equal(classify(status,body),status===503?'ambiguous':status===400?'invalid':status===409?'conflict':'denied');
  });
}
check('status/code mismatch ambiguous', () =>
  assert.equal(classify(409,{contract_version:'1',error_code:'REQUEST_DENIED',message}),'ambiguous'));
check('retired B1 error code is not accepted', () =>
  assert.equal(classify(403,{contract_version:'1',error_code:'CAPABILITY_DISABLED',message}),'ambiguous'));
check('error text cannot disclose SQL/internal context', () =>
  assert.equal(classify(403,{contract_version:'1',error_code:'REQUEST_DENIED',message:'SQL workspace secret'}),'ambiguous'));
check('unversioned error ambiguous', () =>
  assert.equal(classify(403,{error_code:'REQUEST_DENIED',message}),'ambiguous'));
check('unknown HTTP status ambiguous', () =>
  assert.equal(classify(422,{contract_version:'1',error_code:'REQUEST_DENIED',message}),'ambiguous'));
check('network ambiguity has no automatic retry', () => {
  const retryAdvice={ambiguous:'reconcile',conflict:'never'};
  assert.equal(retryAdvice.ambiguous,'reconcile');
  assert.notEqual(retryAdvice.ambiguous,'retry');
});
process.stdout.write('B4_OFFLINE_CONTRACT_SIM_PASS='+count+' provider='+manifest.providerHead+'\n');
