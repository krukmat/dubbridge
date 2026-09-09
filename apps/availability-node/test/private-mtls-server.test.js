import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

test('unbound HTTPS server factory enforces fail-closed mTLS options', async () => {
  const tmpDir = mkdtempSync(join(tmpdir(), 'mtls-test-'));

  try {
    // Generate temporary one-day self-signed RSA-2048 certificate
    execFileSync('openssl', [
      'req', '-x509', '-newkey', 'rsa:2048', '-nodes',
      '-days', '1',
      '-out', join(tmpDir, 'cert.pem'),
      '-keyout', join(tmpDir, 'key.pem'),
      '-subj', '/CN=test'
    ], { stdio: 'ignore' });

    const key = readFileSync(join(tmpDir, 'key.pem'));
    const cert = readFileSync(join(tmpDir, 'cert.pem'));
    const ca = cert;

    const { createPrivateMtlsServer } = await import('../dist/mtls.js');

    const server = createPrivateMtlsServer(
      {
        key,
        cert,
        ca,
        requestCert: false,
        rejectUnauthorized: false,
        minVersion: 'TLSv1.2'
      },
      (req, res) => {
        res.end();
      }
    );

    assert.ok(!server.listening, 'Server should not be listening');
    assert.strictEqual(server.requestCert, true, 'requestCert should be true');
    assert.strictEqual(server.rejectUnauthorized, true, 'rejectUnauthorized should be true');
    assert.strictEqual(server.minVersion, 'TLSv1.3', 'minVersion should be TLSv1.3');

    assert.throws(() => {
      createPrivateMtlsServer(undefined, () => {});
    });

    assert.throws(() => {
      createPrivateMtlsServer({ key: Buffer.alloc(0), cert, ca }, () => {});
    });

    assert.throws(() => {
      createPrivateMtlsServer({ key, cert: Buffer.alloc(0), ca }, () => {});
    });

    assert.throws(() => {
      createPrivateMtlsServer({ key, cert, ca: Buffer.alloc(0) }, () => {});
    });

    assert.throws(() => {
      createPrivateMtlsServer({ key, cert }, () => {});
    });

  } finally {
    rmSync(tmpDir, { recursive: true, force: true });
  }
});
