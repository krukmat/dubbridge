---
type: Audit
date: 2026-11-24
status: design
---

# S1–S4 threat and certification matrix

Baseline DubBridge 6959c509ab6496b0271a32a07ba7030800f4ba0f; physical W2 historical PASS;
E3 candidate tests 81 PASS / one expected skip on 0713e1f.
E1 demonstrated same-UID rollback. No new test execution or host changes.

| Threat | Design control | Required local acceptance (synthetic data only) |
|---|---|---|
| Agent reads or writes protected state | Separate service UID, no agent-accessible secrets or writable parent | Agent identity denied read/write/rename/delete of protected synthetic files; privileged owner can inspect through controlled channel |
| Agent modifies service/runtime or boot configuration | Root-owned install/config, verified exact version/dependencies | Agent cannot replace installed binaries/imports/config/launchd definition; no unattended privilege escalation |
| Same-UID rollback of revoked lease | Volatile active registry, protected journal | Old serialized bytes cannot restore authority; revocation remains final; absence of state cannot become ALLOW |
| Restart or time rollback | Fresh service generation, monotonic duration, no lease recovery | Old IPC/session/lease rejected before and after restart; wall-time anomaly denied; no automatic new HAA request |
| Concurrent request/consume | Protected unique IDs, transaction/lock | Parallel attempts produce <=1 request/consume/lease per session; uncertain transport never replays |
| Unauthorized IPC | Peer identity and bounded typed operation contract | Wrong client/role, unknown fields, fake RRI/card/lease, stale session, arbitrary argv/callback denied |
| Worker executes untrusted code | Separate worker rights, no protected target/keys | Scratch command cannot access service state, mutate target or invoke privileged path; actual test stdout/exit/provenance recorded |
| Git/filesystem bypass | Protected target/credentials, bounded patch, safe paths and exact operations | Direct/normalized CLI, symlinks/hardlinks, hooks, Git config, dirty/changed HEAD cannot modify protected target outside authorized effect |
| Forged HAA authority | Protected HAA server/signing key, separate principals, operator pin, trusted approver app | Fake service/key, wrong audience/role/signature and stale grant blocked; approver private key not exported; signed app installation not agent-writable |
| Fake plan completion | Protected canonical transaction, evidence and manifest pin | Editing Markdown projection does not close plan; changed evidence/HEAD, pending/rejected approval, lease-as-close, crash/uncertain consume never DONE |
| Outside mediated boundary | Explicit threat scope and separately protected remote credentials | Document direct owner/root/remote/cloud privileges; no blanket non-bypass claim |

Stop on any unexpected allow, missing proof, unsigned/unknown deployment,
credential exposure or RRI-policy block. Do not collect or publish keys,
tokens, biometrics, raw environment or sensitive host state. Use redacted
UID/permission/outcome evidence and exact tested Git SHAs.

## Decision
S1/S2 design and S3 decomposition plus S4 matrix are prepared.
ADR-049 is proposed; no implementation, no privileged changes, no physical
approval. E4 and activation remain NO-GO. The owner already waived
independent reviewers for this integration; no independent verdict claimed.
