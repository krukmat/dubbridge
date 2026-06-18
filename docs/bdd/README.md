# BDD Mapping

... (existing content) ...

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

## S-120 — Media preparation
Spec: `docs/bdd/s-120-media-preparation.feature`

| Scenario ID | Description | Task | Executable Evidence | Mobile Flow | HP / EC |
| --- | --- | --- | --- | --- | --- |
| S120_HP1 | Successful preparation produces metadata and HLS outputs | S-120-T2 | `s-120-media-preparation.feature` | — | HP |
| S120_EC1 | Downstream processing is blocked while asset is not prepared | S-120-T3 | `s-120-media-preparation.feature` | — | EC |
| S120_EC2 | Preparation failure leaves the asset not ready and observable | S-120-T4 | `s-120-media-preparation.feature` | — | EC |
| S120_EC3 | Malformed probe/transcode result does not mark the asset prepared | S-120-T5 | `s-120-media-preparation.feature` | — | EC |