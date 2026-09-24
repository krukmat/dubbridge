---
type: Audit
title: "Go-Live October 2026 — T7local / DEV-HANDOFF consolidation"
date: 2026-09-24
status: accepted
slice: S-230
related: MVP0-P2P
---

# Go-Live October 2026 — T7local / DEV-HANDOFF consolidation

## Decision

The October plan keeps two development lanes intentionally parallel:

```text
S-230:     T5d PASS -> T7local -> T7c ------------------+
                                                         |
MVP0-P2P:  P3 PASS + P4 PASS + P5-DEV -> P6 -> P6 PASS |
                                      \                  |
                                       +--> DEV-HANDOFF --+
                                                         |
                                                         v
                                T7local freshness vs exact DEV-HANDOFF head
                                                         |
                                                         v
                                                       T6p-a
```

`DEV-HANDOFF = P3 PASS + P4 PASS + P5-DEV + P6 PASS`.

P5.T3/P5-CERT remains outside DEV-HANDOFF and returns in the release
certification lane (compatible T7p evidence or mandatory P7.T2 resolution).

## T7local scope

T7local is the **base S-230 mobile smoke** against the local Docker Compose
gateway. It is not the invited-P2P certification task.

Required base flow:

```text
login
 -> upload
 -> rights
 -> finalize / preparation
 -> review
 -> publish
 -> normal HLS playback
```

Explicitly outside T7local evidence:

- My Content / Invite creation as a P2P acceptance claim;
- invitation claim;
- P4 P2P sync and ciphertext verification;
- P5 loopback P2P playback;
- HPKE/Keystore certification;
- P5.T3/P5-CERT;
- P7 exact-artifact certification.

The normal app may contain P6/P2P code because the lanes share a binary. Scope
is defined by behavior exercised and evidence claimed, not by whether P2P source
is present in the APK.

## Gateway correction

`infra/local/docker-compose.yml` exposes:

```text
api      host 8080 -> container 8080
gateway  host 8082 -> container 8081
```

Therefore the canonical Android-emulator target for T7local is:

```text
EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://10.0.2.2:8082
```

Using `:8080` bypasses the gateway and does not satisfy T7local.

The T7local runtime evidence must include:

```text
GET http://localhost:8082/health/live
GET http://localhost:8082/health/ready
```

until the shared preflight script owns those checks directly.

## Why T7local does not wait for P6

Making T7local depend on P6 would remove useful parallelism and recreate the
schedule pressure the 2026-09-06 re-sequencing was designed to avoid. T7local
can therefore PASS on an earlier head while P6 continues.

That earlier PASS remains valid for **T7local itself**. It is not automatically
treated as fresh enough for T6p-a.

## T6p-a freshness gate

Before T6p-a activation:

1. Record the exact T7local evidence head.
2. Record the exact DEV-HANDOFF head.
3. Compare changes between them for the base-flow/local-entry surfaces.
4. If there is no relevant change, record
   `T7LOCAL_FRESHNESS=PASS_NO_RERUN`.
5. If relevant paths changed, execute a bounded regression on the exact
   DEV-HANDOFF head:
   - gateway live + ready;
   - login/session establishment;
   - representative upload/finalize/preparation;
   - review + publish;
   - normal HLS playback;
   - mobile QA.
6. Record supplemental evidence. Do not execute P2P certification as part of
   this freshness regression.

Relevant surfaces include mobile runtime configuration/auth/shared
navigation/API client/base screens, gateway, the base API routes used by the
flow, and `infra/local/docker-compose.yml`.

A failed freshness regression blocks T6p-a and is a finding. It is not silently
patched inside the deployment-input freeze.

## Branch policy

The integrated October go-live line remains
`feature/p2p-mvp-core`. Historical branch
`claude/s-230-s-150-prerequisites-k4i4k1` is not the go-live integration
branch and must not be revived for this work.

## Canonical sequence

```text
NOW
├─ MVP0-P2P: P6 -> P6 PASS -> DEV-HANDOFF
└─ S-230:    T7local -> T7c
                     \       /
                      freshness
                         |
                         v
                       T6p-a
                         |
                 T6p-b -> T6p-c -> T6p-d
                         |
                   T7 + T7p / P7
                         |
                        T9g
```
