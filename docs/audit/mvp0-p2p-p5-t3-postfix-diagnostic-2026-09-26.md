---
type: Audit
title: "P5.T3 post-fix Android diagnostic — device identity conflict"
status: active
---

# P5.T3 post-fix Android diagnostic — 2026-09-26

**Result:** `CLAIM_FAILED` before the invitation claim. P5.T3 remains open; no
SYNC, VERIFY, playback, secret-boundary, or no-fallback evidence was produced.

## Scope and environment

- Branch `feature/p2p-mvp-core`, HEAD
  `850cee85391d2c5d24483b55f744ba6d036ef7d4`.
- Android emulator `emulator-5554` (`sdk_gphone64_arm64`, Android 14).
- Compose-managed gateway on host `8082`, API, availability node, PostgreSQL,
  Redis, and MinIO were running.
- `bash infra/local/p2p/preflight.sh --with-runtime` passed before the run.
- The certification build passed with JDK 17 and the P5 harness enabled. The
  first build attempt under the inherited JDK 26 failed in Gradle's Android
  toolchain; selecting the repository-compatible JDK 17 resolved that setup
  issue without a source change.
- Local PostgreSQL was one migration behind the checked-out repository
  (`_sqlx_migrations` ended at 38). Migration 39 was applied byte-for-byte and
  recorded with its repository checksum before creating the fresh invitation.

Only one fresh invitation was submitted to the harness. It was valid and
unclaimed immediately before the tap. The token stayed in mode-0600 local
diagnostic files and is not reproduced here.

## Observed boundary

The harness returned to `stage=idle` with `CLAIM_FAILED`.

Post-run state proves that the invitation claim transaction was never reached:

- invitation `a0925cc0-91da-4880-8699-98cd107b5c96` remained unclaimed and
  unrevoked;
- zero `p2p_audience_authorizations` rows reference that invitation;
- no claim/device audit event was emitted for the run;
- the token file's SHA-256 matched the invitation's stored `token_hash`, and
  the invitation still had more than 40 minutes before expiry when submitted.

Therefore this result does not reopen the gateway-routing or token-validity
findings from the 2026-09-22 attempt and says nothing about P4 sync or P5
playback behavior.

## Classification

The first failing operation is `P2PAudienceService.ensureDevice`, which runs
inside the harness's broad CLAIM stage before `claimP2pInvitation`.

Immediately before this diagnostic, an unrelated T7local Maestro run used the
same emulator. Its tracked flow launches the app with `clearState: true`
(`mobile/maestro/t7local/playback-real.yaml`). Clearing application state also
removes the app-owned Android Keystore key. The P5 native module then generated
a new P-256 key under the fixed alias `dubbridge-p2p-k1-v1`.

The intended P5 viewer still had an active server device registered on
2026-09-22 under that same alias and its previous public key:
`8c7828fa-84eb-4a89-9d6b-280de61d1046`. The repository's
`register_or_get_active_device` contract is idempotent only when both alias and
public key match. A different public key while an active device exists returns
`DbError::Conflict`. The mobile service propagates that non-OK device result and
the certification harness collapses it to `CLAIM_FAILED`.

This is a high-confidence inference from the deterministic state transition and
the checked-in server contract. The harness intentionally hides the underlying
HTTP status, so this run does not contain a direct on-screen `409` observation.

## Recommended resolution

1. Run release evidence on an emulator reserved for the exact candidate. Do
   the final `clearState` before provisioning the viewer device, then avoid any
   other flow that clears app state until certification finishes.
2. For the next bounded rerun, use a fresh viewer with no active device, or use
   an explicit operator-controlled device replacement path before minting the
   invitation. Do not update the stored public key in place: replacement must
   revoke the prior device and dispositions its authorizations/envelopes.
3. Add an authenticated device-replacement/rotation capability for recovery
   after app-data loss. Preserve the single-active-device rule, audit the
   replacement, and revoke capabilities bound to the displaced key.
4. Split harness diagnostics into at least `DEVICE_REGISTRATION_FAILED` and
   `INVITATION_CLAIM_FAILED`, while continuing to suppress raw tokens, keys, and
   transport bodies. This will prevent a pre-claim identity conflict from being
   misclassified as an invitation failure.

No product source was changed in this diagnostic. The actionable product gap is
device recovery/rotation; the immediate certification fix is deterministic
emulator isolation and provisioning order. Full P5.T3 closure still requires a
fresh exact-artifact run through CLAIM -> SYNC -> VERIFY -> PLAYBACK plus the
secret/no-fallback and teardown evidence.

## Addendum — fresh-viewer rerun and local P2P topology (2026-09-26)

Recommendation 2 was executed later the same morning (session artifacts under
`.agent/p5-t3-recovery/`, not committed; secrets stayed in mode-0600 files):

- A fresh test viewer in an isolated workspace registered the current native
  key as device `28d45aac-417e-4a84-a2c4-4a1084bacb24`.
- Two fresh invitations were claimed through the harness:
  `ec49928a-…` (authorization `3f612579-…`, 07:14:31Z) and `ca27c15d-…`
  (authorization `187cf489-…`, 07:21:20Z). **CLAIM is no longer the blocker.**
- Both runs stopped at `SYNC_FAILED` (final harness screen re-captured at
  09:35 local: `stage=idle`, `SYNC_FAILED`). No VERIFY/PLAYBACK evidence exists.

Between the two runs a client repair (`P5.T3-r1`, committed afterwards as `3f3ffbd`;
`mobile/src/p2p/runtime/product-package-runtime.ts`: `findingPeers()` +
`swarm.flush()` + wait-for-first-update on an empty drive, with
`mobile/__tests__/p2p/product-package-discovery.test.ts`) was applied. It is
consistent with the installed Hyperswarm/Hyperdrive APIs and passes the full
mobile Jest suite, typecheck, lint and worklet drift check, but it has no
recorded RRI/review/RED evidence and is **not validated on device** (the
second run failed identically; that run is not proven to have loaded the
regenerated worklet).

### Root cause of SYNC_FAILED in this environment

Probe evidence (same drive key, same libraries):

| Probe location | Connections | Manifest read |
|---|---|---|
| macOS host (`mobile/` deps), 20 s extra wait | 0 | none |
| macOS host, after AN re-announce | 0 | none |
| Container on the Compose network | 1 at 838 ms | 626 bytes |

The local Docker engine is Colima (`colima list`: no VM address). The
Availability Node's Hyperswarm uses the public DHT with a random UDP port
behind Colima's VM NAT plus the LAN router; Compose publishes only `8443/tcp`.
Because host and AN share the same public IP, HyperDHT tries the AN's local
address (`172.18.0.6`), which is not routable from macOS (`ping` 100% loss,
route via `en0` default gateway); hairpin hole-punching through the double NAT
also fails. The emulator's user-mode network egresses through the host, so it
inherits the same unreachability (inference, consistent with the identical
`SYNC_FAILED`).

This is a **local certification-topology blocker**, not evidence of a P4/P5
product defect and not a reason to add HTTP fallback.

### Unblock options (owner decision)

**Decision 2026-09-26:** option 3. P5.T3 is re-scoped as a general test inside the physical tests (`docs/audit/mvp0-p2p-p5-t3-sequencing-replan-2026-09-22.md` § Amendment 2026-09-26); options 1 and 2 are not pursued.

**Scope limit:** every run above is on an emulator. P5-CERT requires physical
Android evidence (`docs/tasks/mvp0-p2p-p5-local-playback.md`, P5-CERT
definition), so options 1 and 2 are de-risking runs only and cannot close
P5.T3. The emulator's user-mode NAT is also an extra layer a physical phone
does not have; a physical phone on the LAN would still not reach the
Colima-hosted AN (`172.18.0.0/16` is not routable from the LAN either).

1. **Routable Colima VM** — restart Colima with a network address and add a
   host route for the Compose subnet to the VM. Keeps the real AN process;
   needs `sudo` and restarts the whole local stack.
2. **Host-side seeder from a copy of the AN's ciphertext store** — no system
   change, reversible; deviation to record: the seeding peer is not the AN
   process (ciphertext and drive key unchanged).
3. **Release lane** — take P5.T3 evidence against the deployed, publicly
   reachable AN (T6p/T7p exact RC or P7.T2), as the sequencing replan already
   allows.

### Value of an emulator de-risking run (option 2)

- **What it would prove early:** the first Android-runtime execution of SYNC
  (P5.T3-r1 cold-drive read), VERIFY, K1/HPKE unwrap with an app-owned
  Keystore key, loopback session, player rendering and teardown. None of these
  has run on Android yet; today the first such run would be the physical T7p
  run, late in the 2026-10-30 window and behind the blocked T6p-a gate.
- **What it cannot prove:** P5-CERT (physical device), real NAT traversal to a
  deployed AN, hardware-backed Keystore, device performance/thermal behavior.
- **Controls:** cheap host-probe against the seeder before any device run to
  separate environment from product failures; timebox to one seeder and at most
  two device runs; label results as de-risking, never certification.
