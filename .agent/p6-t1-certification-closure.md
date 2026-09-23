# P6.T1 aggregate certification closure packet

## Goal

Close **P6.T1 / T1.H** without changing product behavior.

Branch: `feature/p2p-mvp-core`

Aggregate T1 base:

`c6ce20397a0290da9a25012deafd13bb8088ffbd`

Exact certified candidate before T1.H docs:

`d19e51f424368f9b05650266ab9adeb4c57d8975`

Do not activate P6.T2 until T1.H is explicitly PASS.

## Frozen acceptance

- **HP-P6.T1-1:** Owner sees Processing → Ready and can create an invite for their ready package.
- **EC-P6.T1-1:** Other-owner assets or failed/non-ready publications do not expose an invite action; server rejection remains enforced.

Parent P6 owner-side HP-1 is also exercised. Viewer-side P6 HP-2 / EC-1 / EC-2 remain T2/T3 work and are not claimed here.

## Code-review scope

Review only the T1 source/test/dependency changes:

```text
mobile/src/screens/MyContentScreen.tsx
mobile/src/screens/HomeScreen.tsx
mobile/src/navigation/RootNavigator.tsx
mobile/src/p2p/dashboard/MyContentModel.ts
mobile/src/p2p/dashboard/useMyContentState.ts
mobile/src/p2p/dashboard/useMyContentInvite.ts
mobile/__tests__/MyContentScreen.test.tsx
mobile/__tests__/HomeScreen.test.tsx
mobile/__tests__/RootNavigator.test.tsx
mobile/package.json
mobile/package-lock.json
```

The aggregate diff is from `c6ce2039` to the current branch HEAD; T1.H docs are excluded by `REVIEW_PATHS`.

## Mandatory independent reviewer command

Run from the repository root on the local Mac with Ollama available:

```bash
PEER_REVIEW_RRI=55 \
PEER_REVIEW_PHASE=code \
PEER_REVIEW_CALLER=unknown \
PEER_REVIEW_TASK_ID=P6.T1 \
PEER_REVIEW_ARTIFACT=.agent/peer-code-review-p6-t1.json \
PEER_REVIEW_BASE=c6ce20397a0290da9a25012deafd13bb8088ffbd \
REVIEW_PATHS="mobile/src/screens/MyContentScreen.tsx mobile/src/screens/HomeScreen.tsx mobile/src/navigation/RootNavigator.tsx mobile/src/p2p/dashboard/MyContentModel.ts mobile/src/p2p/dashboard/useMyContentState.ts mobile/src/p2p/dashboard/useMyContentInvite.ts mobile/__tests__/MyContentScreen.test.tsx mobile/__tests__/HomeScreen.test.tsx mobile/__tests__/RootNavigator.test.tsx mobile/package.json mobile/package-lock.json" \
make qa-peer-workflow-review
```

Expected RRI-55 route: local Gemma primary, GPT-OSS fallback, D14 only if both local reviewers are unusable.

A PASS artifact is required. If the reviewer returns findings, every finding must be dispositioned and any repair must re-run the exact-head CI before owner verification.

## Aggregate executable evidence already green

Candidate head `d19e51f4`, CI run `35789842735`:

- 15/15 jobs PASS
- mobile 63/63 suites
- mobile 461/461 tests
- P3 T2c3 harness PASS
- workspace tests + Redis integration PASS
- workspace line coverage 90.43%
- maintainability / fmt / clippy / cargo-check / release-build / deny / qa-docs / config-secrets / roadmap-drift / peer-workflow-review smoke / python-complexity / s3-integration PASS

## Owner verification checkpoint

Only after the independent reviewer is PASS (or all findings are repaired and re-certified), the owner may verify T1.H.

Recommended focused owner command:

```bash
git checkout feature/p2p-mvp-core
git pull --ff-only
cd mobile
npm test -- --runInBand __tests__/MyContentScreen.test.tsx __tests__/RootNavigator.test.tsx __tests__/HomeScreen.test.tsx
```

Owner statement required by repository policy:

```text
I verified every happy path and edge case defined for P6.T1 has executable evidence
at an appropriate layer that replicates the expected behavior.
```

Do not write this statement on the owner's behalf.
