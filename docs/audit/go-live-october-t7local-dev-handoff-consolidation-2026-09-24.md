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

The 2026-09-24 model intentionally allowed two lanes to advance in parallel.
That sequencing worked: MVP0-P2P reached **P6 PASS** and
**DEV-HANDOFF SATISFIED**.

**Disposition update 2026-09-26:** `T7c PASS` is satisfied and
`T7local` is **CLOSED — OWNER ACCEPTED**. Runtime evidence proves
B3/C1/C2, review navigation/detail/playback, and C3 through the real gateway
with read-only PostgreSQL persistence proof. C4 is explicitly owner-accepted;
E4 freshness is owner-waived. No further T7local rerun is required.

The current execution shape is therefore:

```text
P3 PASS + P4 PASS + P5-DEV + P6 PASS
                  |
                  v
        DEV-HANDOFF @ 84ea5edc
                  |
          T7c PASS
                  |
      T7local CLOSED
      OWNER ACCEPTED
      C3 runtime-proven
      C4 owner-accepted
      E4 owner-waived
                  |
                  v
       T6p-a still DEFERRED
   (standing contract still requires
    T7local PASS + freshness)
```

`DEV-HANDOFF = P3 PASS + P4 PASS + P5-DEV + P6 PASS`, pinned at
`84ea5edc` on 2026-09-25.

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

## Sequencing result

T7local was deliberately kept independent of P6 so the roadmap did not recreate
the original deployment cycle. P6 closed first, followed by T7c and the bounded
T7local closure.

The exact C3 evidence head is
`582be62cf23c0a790744f478c82cfb07580a1e07`. On that head, the real gateway
returned `state=approved` for review task
`a69a99bf-4809-49ed-82fa-6b07e938ce12`, and the read-only PostgreSQL probe
confirmed persisted verdict `approved` at
`2026-09-26 06:48:14.294122+00`.

T7local is now **CLOSED — OWNER ACCEPTED**. C4 is owner-accepted and E4
freshness is owner-waived; neither is represented as independently runtime-proven.
The standing T6p-a contract still requires T7local PASS plus freshness, so T6p-a
remains deferred until that gate is deliberately amended or satisfied.

## T6p-a freshness gate

**Current disposition 2026-09-26:** the T7local task itself is closed by owner
acceptance and must not be rerun. Its E4 freshness check is owner-waived because
relevant gateway changes landed after the C3 evidence head; no technical
`PASS_NO_RERUN` claim is made.

This does **not** silently amend T6p-a. Its standing activation contract still
requires:

1. `T7local PASS`;
2. `T7c PASS`;
3. MVP0-P2P `DEV-HANDOFF`;
4. a T7local freshness disposition.

Only items 2 and 3 are technically satisfied today. T7local is closed by owner
acceptance rather than a full technical PASS, and freshness is waived rather
than technically certified. Therefore **T6p-a remains DEFERRED** until the
owner deliberately changes that downstream gate or equivalent release evidence
satisfies it.

## Branch policy

The integrated October go-live line remains
`feature/p2p-mvp-core`. Historical branch
`claude/s-230-s-150-prerequisites-k4i4k1` is not the go-live integration
branch and must not be revived for this work.

## Canonical sequence

```text
DEV-HANDOFF @ 84ea5edc  [SATISFIED]
          |
T7local -> T7c
    |
    +-> E4 freshness vs 84ea5edc
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

Pre-execution check on 2026-09-25: `84ea5edc..1067d2b2` is documentation-only,
so no additional runtime regression is currently implied. Recompute this against
the actual T7local evidence HEAD.
