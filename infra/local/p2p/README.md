# Local P2P runtime and P5 certification bridge

This runbook exists to produce a **real** P3 invitation token for the existing
Android P5 certification harness. It does not seed or fake a `ready`
publication.

## 1. Generate local mTLS material

From the repository root:

```bash
bash infra/local/p2p/generate-mtls.sh
```

The generated CA, server certificate, client identity and client SHA-256
fingerprint live under `tmp/p2p-mtls/` (git-ignored).

Export the generated client fingerprint for Compose:

```bash
export DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS="$(cat tmp/p2p-mtls/client-fingerprint.txt)"
```

The server certificate is valid for `availability-node`, `localhost`, and
`127.0.0.1`. The worker client always uses real TLS + a real client
certificate; there is no insecure development bypass.

## 2. Verify the Availability Node before the full stack

```bash
npm --prefix apps/availability-node ci
npm --prefix apps/availability-node run build
npm --prefix apps/availability-node run test:bootstrap
```

To run it directly on the host instead of Compose:

```bash
export DUBBRIDGE_P2P_CIPHERTEXT_ROOT="$PWD/tmp/p2p-ciphertext"
export DUBBRIDGE_P2P_AVAILABILITY_DRIVE_ROOT="$PWD/tmp/p2p-availability-drive"
export DUBBRIDGE_P2P_AVAILABILITY_INDEX_ROOT="$PWD/tmp/p2p-availability-index"
export DUBBRIDGE_P2P_AVAILABILITY_SERVER_KEY_PEM="$PWD/tmp/p2p-mtls/server-key.pem"
export DUBBRIDGE_P2P_AVAILABILITY_SERVER_CERT_PEM="$PWD/tmp/p2p-mtls/server-cert.pem"
export DUBBRIDGE_P2P_AVAILABILITY_CA_PEM="$PWD/tmp/p2p-mtls/ca.pem"
export DUBBRIDGE_P2P_AVAILABILITY_ALLOWED_CLIENT_FINGERPRINTS="$(cat tmp/p2p-mtls/client-fingerprint.txt)"

npm --prefix apps/availability-node start
```

The direct-host bootstrap binds to `127.0.0.1:8443` by default. Compose
explicitly binds the container listener to `0.0.0.0:8443` while exposing only
the configured local Docker port.

## 3. Start the local app/P2P stack with Compose

Create the local runtime directories once:

```bash
mkdir -p \
  tmp/p2p-ciphertext \
  tmp/p2p-availability-drive \
  tmp/p2p-availability-index
```

Then:

```bash
docker compose -f infra/local/docker-compose.yml --profile app up \
  postgres redis minio minio-init availability-node api worker-runner
```

Relevant wiring:

```text
worker-runner
  -> HTTPS/mTLS availability-node:8443
  -> shared /var/lib/dubbridge/p2p-ciphertext

availability-node
  -> verifies packages/<publication>/<lineage>/manifest.json
  -> Hyperdrive ciphertext publication
  -> Hyperswarm announce
  -> durable external evidence returned to worker

worker-runner
  -> PostgreSQL reconciliation
  -> durable same-lineage P2P_READY
```

`availability-node` never receives the worker's client private key; it only
receives server TLS material and the allowed SHA-256 client-certificate
fingerprint. The worker receives the CA plus `client-identity.pem`.

## 4. Produce a real owner publication

Use the normal application/API flow with the same authenticated owner account
used later to create the invitation. Upload/prepare a short test asset and let
the existing S-120 -> P2 activation run. Do not mutate `p2p_publications`
directly.

With an owner access token in `OWNER_JWT`, poll the authoritative dashboard:

```bash
curl -fsS \
  -H "Authorization: Bearer ${OWNER_JWT}" \
  http://localhost:8080/p2p/content
```

Continue only when the target asset is returned with:

```json
{"state":"ready"}
```

The `asset_id`, `publication_id`, `lineage_id` and descriptor must come from
that real owner-scoped read model.

## 5. Create the real P3 invitation token

Set the `ASSET_ID` returned above and call the existing owner endpoint:

```bash
curl -fsS \
  -X POST \
  -H "Authorization: Bearer ${OWNER_JWT}" \
  -H "Content-Type: application/json" \
  -d '{"ttl_seconds":3600}' \
  "http://localhost:8080/assets/${ASSET_ID}/p2p/invitations"
```

The response contains the one-time raw `token`. Keep it out of logs and do
not commit it. The database persists only its SHA-256 hash.

If this endpoint returns `404`, first verify that the ready publication's
asset really belongs to the authenticated owner. If it returns `409`, verify
that the publication still satisfies the authoritative same-lineage READY
predicate.

## 6. Re-run P5 on the Android emulator

From `mobile/`:

```bash
export JAVA_HOME="/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home"
export PATH="$JAVA_HOME/bin:$PATH"
npm run android:p2p-cert
```

In the app:

```text
login as viewer
-> P5 device certification
-> paste the real invitation token
-> Run P5 certification
```

Expected:

```text
claim -> sync -> verify -> playback -> stage=playing
```

Then run `Stop certification playback`, verify clean teardown, and start once
again. The invalid-token check must still fail closed with `CLAIM_FAILED`.

An emulator PASS is not a physical-device PASS; Android hardware-backed
Keystore behavior and real-device networking remain separate certification
evidence.
