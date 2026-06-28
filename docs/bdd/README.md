# BDD Mapping

Canonical home for repository BDD artifacts: `docs/bdd/`.

## Convention

- All DubBridge `.feature` specs live in `docs/bdd/`.
- Scenario IDs remain stable and behavioral.
- Mobile-owned executable evidence may still live in `mobile/maestro/` or mobile
  tests even when the canonical `.feature` file lives in `docs/bdd/`.
- Retrospective slices may map to shipped unit/integration evidence or runner
  artifacts when no standalone Maestro flow exists.

## Canonical spec files

- `p4-workspace.feature`
- `p6-compliance.feature`
- `s-050-mobile-client.feature`
- `s-055-maestro-suite.feature`
- `s-060-mobile-asset-lifecycle.feature`
- `s-120-media-preparation.feature`
- `s-125-hls-playback-delivery.feature`
- `s-127-mobile-review-player.feature`
- `s-160-review.feature`
- `s-200-mobile-auth.feature`

## S-050 — First-party mobile client
Spec: `docs/bdd/s-050-mobile-client.feature`

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| SC-AUTH-1 | Sign in through the mobile gateway handoff | `MBF-T1` | `mobile/__tests__/mobile.auth-flow.test.tsx`; `mobile/__tests__/auth.provider.test.tsx` | — | HP |
| SC-AUTH-2 | Login fails closed when the handoff is missing or invalid | `MBF-T1` | `mobile/__tests__/mobile.auth-flow.test.tsx`; `mobile/__tests__/auth.provider.test.tsx` | — | EC |
| SC-AUTH-3 | Token-like session values are rejected on device | `MBF-T1` | `mobile/__tests__/auth.provider.test.tsx`; `mobile/__tests__/auth.session.test.ts` | — | EC |
| SC-NAV-1 | Auth state controls the root navigation tree | `MBF-T1` | `mobile/__tests__/RootNavigator.test.tsx`; `mobile/__tests__/mobile.auth-flow.test.tsx` | — | HP / EC |
| SC-ASSET-1 | Browse my asset list and open asset detail | `MBF-T1` | `mobile/__tests__/asset.screens.test.tsx`; `mobile/__tests__/mobile.auth-flow.test.tsx` | — | HP |
| SC-ASSET-2 | Asset surfaces handle empty, failed, or unavailable responses clearly | `MBF-T1` | `mobile/__tests__/asset.screens.test.tsx` | — | EC |

## S-055 — Maestro screenshot / visual-audit suite
Spec: `docs/bdd/s-055-maestro-suite.feature`

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| SC-SUITE-1 | Capture the unauthenticated auth surface | `MBF-T2` | `mobile/maestro/auth-surface.yaml` | `mobile/maestro/auth-surface.yaml` | HP |
| SC-SUITE-2 | Bootstrap an authenticated session without UI login | `MBF-T2` | `mobile/maestro/authenticated-audit.yaml` | `mobile/maestro/authenticated-audit.yaml` | HP |
| SC-SUITE-3 | Screenshot artifacts remain free of sensitive session values | `MBF-T2` | `mobile/maestro/seed-and-run.sh` | — | EC |

## S-060 — Mobile asset lifecycle
Spec: `docs/bdd/s-060-mobile-asset-lifecycle.feature`

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| SC-LIST-1 | Browse my assets (populated list) | `T2` | `docs/tasks/s-060-mobile-asset-lifecycle.md` | `mobile/maestro/asset-list.yaml` | HP |
| SC-LIST-2 | Empty asset list | `T2` | `docs/tasks/s-060-mobile-asset-lifecycle.md` | `mobile/maestro/asset-list.yaml` | EC |
| SC-DETAIL-1 | Open an asset from the list | `T2` | `docs/tasks/s-060-mobile-asset-lifecycle.md` | `mobile/maestro/asset-detail.yaml` | HP |
| SC-INGEST-1 | Upload a new asset (happy path) | `T3a`, `T3b` | `docs/tasks/s-060-mobile-asset-lifecycle.md` | `mobile/maestro/asset-ingestion.yaml` | HP |
| SC-INGEST-2 | Upload rejected without rights | `T3b` | `docs/tasks/s-060-mobile-asset-lifecycle.md` | `mobile/maestro/asset-ingestion-no-rights.yaml` | EC |

## S-120 — Media preparation
Spec: `docs/bdd/s-120-media-preparation.feature`

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| S120_HP1 | Successful preparation produces metadata and HLS outputs | S-120-T2 | `s-120-media-preparation.feature` | — | HP |
| S120_EC1 | Downstream processing is blocked while asset is not prepared | S-120-T3 | `s-120-media-preparation.feature` | — | EC |
| S120_EC2 | Preparation failure leaves the asset not ready and observable | S-120-T4 | `s-120-media-preparation.feature` | — | EC |
| S120_EC3 | Malformed probe/transcode result does not mark the asset prepared | S-120-T5 | `s-120-media-preparation.feature` | — | EC |

## S-125 — HLS playback delivery
Spec: `docs/bdd/s-125-hls-playback-delivery.feature`

> Delivered 2026-06-22. Executable evidence is now certified in the completed
> `S-125` ledger and mapped here to the concrete integration tests that prove the
> grant-issuance and playback-delivery boundary.

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| S125_HP1 | Authorized reviewer obtains a playback grant for a ready asset | S-125-T4 | `apps/api/tests/playback_grant_test.rs::authorized_reviewer_ready_asset_receives_grant_and_audit_row` | — | HP |
| S125_EC1 | Grant issuance is denied for an asset that is not ready | S-125-T4 | `apps/api/tests/playback_grant_test.rs::not_ready_asset_returns_fail_closed_denial_and_writes_no_grant_row` | — | EC |
| S125_EC2 | Unauthorized caller cannot obtain a playback grant | S-125-T4 | `apps/api/tests/playback_grant_test.rs::authenticated_non_member_returns_403_and_writes_no_grant_row`; `apps/api/tests/playback_grant_test.rs::unauthenticated_request_returns_401_and_writes_no_grant_row` | — | EC |
| S125_HP2 | Manifest is returned with short-lived scoped segment references only | S-125-T5b | `apps/api/tests/playback_delivery_test.rs::valid_grant_returns_manifest_with_short_lived_segment_references` | — | HP |
| S125_EC3 | Segment fetched with an expired scoped reference is denied | S-125-T5b | `apps/api/tests/playback_delivery_test.rs::expired_short_lived_segment_reference_is_denied_fail_closed` | — | EC |

## S-127 — Mobile review player surface
Spec: `docs/bdd/s-127-mobile-review-player.feature`

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| SC-PLAYBACK-1 | Review detail loads embedded playback after grant success | `S-127-T3` | `mobile/__tests__/ReviewDetailScreen.test.tsx::HP-1: approve posts a scoped decision, rotates session, and reveals publish`; `mobile/maestro/playback.yaml` | `mobile/maestro/playback.yaml` | HP |
| SC-PLAYBACK-2 | Review detail keeps the decision flow usable when playback is unavailable | `S-127-T3` | `mobile/__tests__/ReviewDetailScreen.test.tsx::EC-5: playback denial shows a not-ready empty state and keeps decision controls usable`; `mobile/__tests__/ReviewDetailScreen.test.tsx::EC-6: playback failure shows an error state and keeps decision controls usable` | `mobile/maestro/playback.yaml` | EC |
| SC-PLAYBACK-3 | Asset detail opens inline playback after an explicit play action | `S-127-T4` | `mobile/__tests__/asset.screens.test.tsx::HP-1: finalized asset shows Play and opens inline playback after an explicit tap`; `mobile/maestro/playback.yaml` | `mobile/maestro/playback.yaml` | HP |
| SC-PLAYBACK-4 | Asset detail denial or failure leaves the rest of the screen usable | `S-127-T4` | `mobile/__tests__/asset.screens.test.tsx::EC-1: playback denial shows a not-ready state and keeps compliance access usable`; `mobile/__tests__/asset.screens.test.tsx::EC-2: playback failure shows an error state and keeps compliance access usable` | `mobile/maestro/playback.yaml` | EC |

## S-210 — Mobile product-experience refresh
Spec: `docs/bdd/s-210-mobile-product-experience.feature`

> **Status:** Implemented (2026-06-28). All tasks T0–T9 complete. Evidence column
> updated from `(planned)` to confirmed executable evidence. Maestro flow edits
> landed with their UI task (D12); final accordion step added in T9.

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| SC-DASH-1 | Home dashboard shows live content on load | S-210-T3 | `mobile/__tests__/HomeScreen.test.tsx::HomeScreen > HP-1` (greeting + review count + recent assets + quick-actions) | — (Maestro home flow deferred: X-S-210-1 data not yet real) | HP |
| SC-DASH-2 | Home dashboard degrades cleanly on error or session expiry | S-210-T3 | `mobile/__tests__/HomeScreen.test.tsx::HomeScreen > EC-1` (fetch error → StateView error); `HomeScreen > EC-2` (session expired → auth.logout) | — | EC |
| SC-DASH-3 | Home quick-actions reach the correct sections (testIDs invariant) | S-210-T3 | `mobile/__tests__/RootNavigator.test.tsx::HP-1` (home-open-assets, home-open-upload, home-open-review, home-open-organizations); existing Maestro flows | existing Maestro flows (testIDs preserved) | HP |
| SC-ACTBAR-1 | Primary action is bottom-anchored on Upload / AssetDetail / ReviewDetail | S-210-T2 | `mobile/__tests__/ActionBar.test.tsx::ActionBar > HP-1/HP-2/EC-1`; Upload/AssetDetail/ReviewDetail screen tests (upload-finalize, asset-play-button, review-approve, review-reject preserved) | — | HP |
| SC-FORM-1 | Incomplete rights form shows a visible validation message | S-210-T6 | `mobile/__tests__/UploadScreen.test.tsx::SC-FORM-1 > HP-1` (all-empty → per-field errors); `SC-FORM-1 > HP-2` (form does not advance); `SC-FORM-1 > EC-1` (error clears on interaction) | — | EC |
| SC-FORM-2 | Rights form reflects a three-step progress indicator | S-210-T6 | `mobile/__tests__/UploadScreen.test.tsx::SC-FORM-2 > HP-1` (indicator visible on rights step); `SC-FORM-2 > HP-2` (advances to File step) | — | HP |
| SC-EMPTY-1 | Empty list screens present a primary CTA | S-210-T7 | `mobile/__tests__/asset.screens.test.tsx::SC-EMPTY-1 > HP-1` (empty + CTA visible); `SC-EMPTY-1 > HP-2` (CTA press calls onOpenUpload); `SC-EMPTY-1 > EC-1` (no CTA when onOpenUpload omitted) | — | HP |
| SC-STATUS-1 | Domain status values render as user-facing labels | S-210-T8 | `mobile/__tests__/format.test.ts::formatStatusLabel > HP-1` (finalized → "Ready"; in_review → "In review"); `HP-2` (consent grant → "Active"); `EC-1` (unknown fallback); badge tone unchanged | — | HP |

### TestID invariants (hard contract — preserved across S-210)

The following testIDs are asserted by existing Maestro flows and Jest suites and were
preserved verbatim across all S-210 tasks:

`home-screen`, `home-open-assets`, `home-open-upload`, `home-open-review`,
`home-open-organizations`, `home-sign-out`, `asset-list-screen`,
`asset-list-empty-state`, `asset-card-{id}`, `asset-detail-screen`,
`asset-play-button`, `asset-open-compliance`, `upload-screen`, `upload-finalize`,
`review-inbox-screen`, `review-task-card-{id}`, `review-detail-screen`,
`review-approve`, `review-reject`.

### Existing scenario assertion deltas (landed — Maestro edits per D12)

| Scenario | Original assertion | S-210 change | Shipped with |
| --- | --- | --- | --- |
| SC-DETAIL-1 | `assertVisible: asset-seed-1`, `e2e-user`, `Finalized` | ids behind "Technical details" accordion (D5); status → "Ready" (D8); accordion expanded via `tapOn: asset-tech-details-toggle` before id assertions | T5 (ids/accordion) + T8 (status) + T9 (Maestro tapOn step) |
| SC-LIST-1 | asserts title + status badge | media placeholder tile added (D4); empty-CTA row added (D7) | T4 (placeholder) + T7 (CTA) |
| SC-LIST-2 | asserts empty state text | primary CTA visible (D7) | T7 |
| SC-INGEST-1 | `tapOn: Pick file` → `upload-finalize` | ActionBar repositions buttons; `upload-finalize` testID preserved | T2 |
| SC-INGEST-2 | asserts error text visible | unaffected by S-210 | — |
| SC-PLAYBACK-3 / SC-PLAYBACK-4 | `asset-play-button` → inline player | ActionBar repositions Play; `asset-play-button` preserved | T2 |
| SC-NAV-1 | `home-screen` auth controls root nav | Home gains dashboard content; all `home-open-*` testIDs preserved | T3 |

## S-200 — Mobile credential login with backend-issued JWT (FenixCRM parity)
Spec: `docs/bdd/s-200-mobile-auth.feature`

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| SC-AUTH-1 | Sign in with valid credentials — backend issues HS256 JWT, mobile lands on home | S-200-T4e, T5a, T6a | `crates/auth/src/service.rs::login_success_issues_token_for_existing_account`; `apps/api` handler integration tests (T4e); `mobile/maestro/login.yaml` | `mobile/maestro/login.yaml` | HP |
| SC-AUTH-2 | Register new account + workspace atomically, user signed in | S-200-T3b, T4d, T6a | `crates/auth/src/service.rs::register_success_hashes_password_persists_and_issues_token`; `apps/api` register handler integration tests (T4d) | — | HP |
| SC-AUTH-3 | Stored token restores session on cold start | S-200-T6a | `mobile/` `AuthProvider` cold-start restore; T6a completion record | — | HP |
| SC-AUTH-4 | Invalid credentials rejected generically (unknown email = wrong password) | S-200-T3c, T4e | `crates/auth/src/service.rs::login_wrong_password_and_unknown_email_return_same_error`; 401 generic mapping in T4e integration tests | — | EC |
| SC-AUTH-5 | Expired / rejected token forces logout (401 → logout) | S-200-T6a | `mobile/` `api/client.ts` 401 handler; T6a completion record | — | EC |
| SC-AUTH-6 | Logout clears stored token | S-200-T6a | `mobile/` `AuthProvider` logout path; secure-store clear; T6a completion record | — | EC |
| SC-AUTH-7 | Registration rejects duplicate email (409 Conflict) | S-200-T2c, T3b, T4d | `crates/db/src/user_account.rs::build_registration_result_conflict_propagates`; `crates/auth/src/service.rs::register_duplicate_email_returns_conflict_and_does_not_issue_token`; 409 mapping in T4d integration tests | — | EC |
| SC-AUTH-8 | Algorithm substitution rejected before signature/claim check | S-200-T1b-ii, T1c-i | `crates/auth/src/issuer.rs::parse_rejects_rs256_algorithm`; `crates/auth/src/verifier.rs::hs256_verifier_rejects_rs256_algorithm`; `parse_rejects_alg_none` | — | EC |
