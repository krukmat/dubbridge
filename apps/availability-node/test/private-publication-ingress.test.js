import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { once } from 'node:events';
import https from 'node:https';
import { X509Certificate } from 'node:crypto';

test('private publication ingress rejects unauthorized client identities', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'private-publication-ingress-'));

  try {
    execFileSync('openssl', ['genrsa', '-out', join(dir, 'ca.key'), '2048'], { stdio: 'pipe' });
    execFileSync('openssl', ['req', '-x509', '-new', '-nodes', '-key', join(dir, 'ca.key'), '-out', join(dir, 'ca.crt'), '-days', '1', '-subj', '/CN=Test CA'], { stdio: 'pipe' });

    execFileSync('openssl', ['genrsa', '-out', join(dir, 'server.key'), '2048'], { stdio: 'pipe' });
    writeFileSync(join(dir, 'server.cnf'), `
[req]
distinguished_name = req_distinguished_name
[req_distinguished_name]
[server_ext]
subjectAltName = DNS:localhost
extendedKeyUsage = serverAuth
`);
    execFileSync('openssl', ['req', '-new', '-key', join(dir, 'server.key'), '-out', join(dir, 'server.csr'), '-config', join(dir, 'server.cnf'), '-subj', '/CN=localhost'], { stdio: 'pipe' });
    execFileSync('openssl', ['x509', '-req', '-in', join(dir, 'server.csr'), '-CA', join(dir, 'ca.crt'), '-CAkey', join(dir, 'ca.key'), '-set_serial', '1000', '-out', join(dir, 'server.crt'), '-days', '1', '-extfile', join(dir, 'server.cnf'), '-extensions', 'server_ext'], { stdio: 'pipe' });

    execFileSync('openssl', ['genrsa', '-out', join(dir, 'client-allowed.key'), '2048'], { stdio: 'pipe' });
    writeFileSync(join(dir, 'client-allowed.cnf'), `
[req]
distinguished_name = req_distinguished_name
[req_distinguished_name]
[client_ext]
extendedKeyUsage = clientAuth
`);
    execFileSync('openssl', ['req', '-new', '-key', join(dir, 'client-allowed.key'), '-out', join(dir, 'client-allowed.csr'), '-config', join(dir, 'client-allowed.cnf'), '-subj', '/CN=Allowed Client'], { stdio: 'pipe' });
    execFileSync('openssl', ['x509', '-req', '-in', join(dir, 'client-allowed.csr'), '-CA', join(dir, 'ca.crt'), '-CAkey', join(dir, 'ca.key'), '-set_serial', '1001', '-out', join(dir, 'client-allowed.crt'), '-days', '1', '-extfile', join(dir, 'client-allowed.cnf'), '-extensions', 'client_ext'], { stdio: 'pipe' });

    execFileSync('openssl', ['genrsa', '-out', join(dir, 'client-unlisted.key'), '2048'], { stdio: 'pipe' });
    writeFileSync(join(dir, 'client-unlisted.cnf'), `
[req]
distinguished_name = req_distinguished_name
[req_distinguished_name]
[client_ext]
extendedKeyUsage = clientAuth
`);
    execFileSync('openssl', ['req', '-new', '-key', join(dir, 'client-unlisted.key'), '-out', join(dir, 'client-unlisted.csr'), '-config', join(dir, 'client-unlisted.cnf'), '-subj', '/CN=Unlisted Client'], { stdio: 'pipe' });
    execFileSync('openssl', ['x509', '-req', '-in', join(dir, 'client-unlisted.csr'), '-CA', join(dir, 'ca.crt'), '-CAkey', join(dir, 'ca.key'), '-set_serial', '1002', '-out', join(dir, 'client-unlisted.crt'), '-days', '1', '-extfile', join(dir, 'client-unlisted.cnf'), '-extensions', 'client_ext'], { stdio: 'pipe' });

    execFileSync('openssl', ['genrsa', '-out', join(dir, 'client-rogue.key'), '2048'], { stdio: 'pipe' });
    writeFileSync(join(dir, 'client-rogue.cnf'), `
[req]
distinguished_name = req_distinguished_name
[req_distinguished_name]
[client_ext]
extendedKeyUsage = clientAuth
`);
    execFileSync('openssl', ['req', '-x509', '-new', '-nodes', '-key', join(dir, 'client-rogue.key'), '-out', join(dir, 'client-rogue.crt'), '-days', '1', '-config', join(dir, 'client-rogue.cnf'), '-extensions', 'client_ext', '-subj', '/CN=Rogue Client'], { stdio: 'pipe' });

    const caCert = readFileSync(join(dir, 'ca.crt'));
    const serverKey = readFileSync(join(dir, 'server.key'));
    const serverCert = readFileSync(join(dir, 'server.crt'));
    const allowedClientKey = readFileSync(join(dir, 'client-allowed.key'));
    const allowedClientCert = readFileSync(join(dir, 'client-allowed.crt'));
    const unlistedClientKey = readFileSync(join(dir, 'client-unlisted.key'));
    const unlistedClientCert = readFileSync(join(dir, 'client-unlisted.crt'));
    const rogueClientKey = readFileSync(join(dir, 'client-rogue.key'));
    const rogueClientCert = readFileSync(join(dir, 'client-rogue.crt'));

    const allowedFingerprint = new X509Certificate(allowedClientCert).fingerprint256;

    const fixtureUrl = new URL(
      '../../../docs/fixtures/mvp0-p2p-publication-contract-v1.json',
      import.meta.url
    );
    const fixture = JSON.parse(readFileSync(fixtureUrl, 'utf8')).availability_node;
    const requestBody = fixture.request;
    const successBody = fixture.success_first_publish.body;

    const serverModule = await import('../dist/server.js');
    const contractModule = await import('../dist/contract.js');
    const errorBody = contractModule.createPublicationError("service_identity_rejected").body;

    let calls = 0;
    const publisher = async () => {
      calls++;
      return { status: 201, evidence: successBody };
    };

    const server = serverModule.createPrivatePublicationServer(
      { key: serverKey, cert: serverCert, ca: caCert },
      [allowedFingerprint],
      publisher
    );

    assert.strictEqual(server.listening, false);

    server.listen(0, '127.0.0.1');
    await once(server, 'listening');
    const port = server.address().port;

    const makeRequest = (options) => {
      return new Promise((resolve, reject) => {
        const clientCredentials =
          options.key === undefined && options.cert === undefined
            ? {}
            : { key: options.key, cert: options.cert };
        const req = https.request({
          host: '127.0.0.1',
          port,
          path: `/v1/publications/${requestBody.publication_id}`,
          method: 'PUT',
          ca: [caCert],
          ...clientCredentials,
          servername: 'localhost',
          minVersion: 'TLSv1.3',
          agent: false,
          headers: {
            'Content-Type': 'application/json; charset=utf-8'
          }
        }, (res) => {
          let data = '';
          res.on('data', chunk => data += chunk);
          res.on('end', () => {
            resolve({ status: res.statusCode, body: JSON.parse(data) });
          });
        });
        req.on('error', reject);
        req.write(JSON.stringify(requestBody));
        req.end();
      });
    };

    try {
      await assert.rejects(makeRequest({}));
      assert.strictEqual(calls, 0);
      await assert.rejects(makeRequest({ key: rogueClientKey, cert: rogueClientCert }));
      assert.strictEqual(calls, 0);
      const unlistedRes = await makeRequest({ key: unlistedClientKey, cert: unlistedClientCert });
      assert.strictEqual(unlistedRes.status, 403);
      assert.deepStrictEqual(unlistedRes.body, errorBody);
      assert.strictEqual(calls, 0);
      const allowedRes = await makeRequest({ key: allowedClientKey, cert: allowedClientCert });
      assert.strictEqual(allowedRes.status, 201);
      assert.deepStrictEqual(allowedRes.body, successBody);
      assert.strictEqual(calls, 1);
    } finally {
      const closed = once(server, 'close');
      server.close();
      await closed;
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
