use async_trait::async_trait;
use dubbridge_db::{
    error::DbError,
    user_account::{self, UserAccount},
};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    CredentialError, Hs256Issuer, IssueError, hash_password, normalize_required,
    validate_password_for_register, verify_password,
};

pub const DEFAULT_AUTH_SCOPES: &[&str] = &[
    "assets:read",
    "assets:ingest",
    "workspaces:read",
    "workspaces:write",
];
const DUMMY_PASSWORD_HASH: &str = "$2y$12$B7W778qFaQ2lGUidArlIH.FT38HrF/Xks/0xuvj.h8vWN1r1CkFlG";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSuccess {
    pub token: String,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
}

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error(transparent)]
    Validation(#[from] CredentialError),
    #[error("account already exists")]
    Conflict,
    #[error("failed to issue access token: {0}")]
    TokenIssue(#[source] IssueError),
    #[error("database error: {0}")]
    Db(#[source] DbError),
    #[error("invalid credentials")]
    InvalidCredentials,
}

#[async_trait]
pub trait AccountStore: Send + Sync {
    async fn find_active_by_email(&self, email: &str) -> Result<Option<UserAccount>, DbError>;

    async fn register(
        &self,
        email: &str,
        password_hash: &str,
        workspace_name: &str,
    ) -> Result<(Uuid, Uuid), DbError>;
}

pub trait AccessTokenIssuer: Send + Sync {
    fn issue_access_token(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        scopes: &[String],
    ) -> Result<String, IssueError>;
}

#[derive(Clone)]
pub struct PgAccountStore {
    pool: PgPool,
}

impl PgAccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountStore for PgAccountStore {
    async fn find_active_by_email(&self, email: &str) -> Result<Option<UserAccount>, DbError> {
        user_account::find_active_by_email(&self.pool, email).await
    }

    async fn register(
        &self,
        email: &str,
        password_hash: &str,
        workspace_name: &str,
    ) -> Result<(Uuid, Uuid), DbError> {
        user_account::register(&self.pool, email, password_hash, workspace_name).await
    }
}

impl AccessTokenIssuer for Hs256Issuer {
    fn issue_access_token(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        scopes: &[String],
    ) -> Result<String, IssueError> {
        self.generate_jwt(user_id, workspace_id, scopes)
    }
}

#[derive(Clone)]
pub struct AuthService<S, I> {
    account_store: S,
    token_issuer: I,
    scopes: Vec<String>,
}

impl<S, I> AuthService<S, I> {
    pub fn new(account_store: S, token_issuer: I) -> Self {
        Self::with_scopes(
            account_store,
            token_issuer,
            DEFAULT_AUTH_SCOPES.iter().map(|scope| (*scope).to_string()),
        )
    }

    pub fn with_scopes(
        account_store: S,
        token_issuer: I,
        scopes: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            account_store,
            token_issuer,
            scopes: scopes.into_iter().collect(),
        }
    }
}

impl<S, I> AuthService<S, I>
where
    S: AccountStore,
    I: AccessTokenIssuer,
{
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        workspace_name: &str,
    ) -> Result<AuthSuccess, AuthServiceError> {
        let email = normalize_required(email, "email")?;
        let workspace_name = normalize_required(workspace_name, "workspace_name")?;
        require_password(password)?;
        validate_password_for_register(password)?;

        let password_hash = hash_password(password)?;
        let (user_id, workspace_id) = self
            .account_store
            .register(&email, &password_hash, &workspace_name)
            .await
            .map_err(map_register_db_error)?;

        let token = self.issue_token(user_id, workspace_id)?;

        Ok(AuthSuccess {
            token,
            user_id,
            workspace_id,
        })
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthSuccess, AuthServiceError> {
        let email = normalize_required(email, "email")?;
        require_password(password)?;

        let Some(account) = self
            .account_store
            .find_active_by_email(&email)
            .await
            .map_err(AuthServiceError::Db)?
        else {
            compare_against_dummy_hash(password)?;
            return Err(AuthServiceError::InvalidCredentials);
        };

        if !verify_password(password, &account.password_hash)? {
            return Err(AuthServiceError::InvalidCredentials);
        }

        let token = self.issue_token(account.id, account.workspace_id)?;

        Ok(AuthSuccess {
            token,
            user_id: account.id,
            workspace_id: account.workspace_id,
        })
    }

    fn issue_token(&self, user_id: Uuid, workspace_id: Uuid) -> Result<String, AuthServiceError> {
        self.token_issuer
            .issue_access_token(user_id, workspace_id, &self.scopes)
            .map_err(AuthServiceError::TokenIssue)
    }
}

fn require_password(password: &str) -> Result<(), CredentialError> {
    if password.trim().is_empty() {
        return Err(CredentialError::MissingField { field: "password" });
    }
    Ok(())
}

fn map_register_db_error(error: DbError) -> AuthServiceError {
    match error {
        DbError::Conflict => AuthServiceError::Conflict,
        other => AuthServiceError::Db(other),
    }
}

fn compare_against_dummy_hash(password: &str) -> Result<(), CredentialError> {
    let _ = verify_password(password, DUMMY_PASSWORD_HASH)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::*;
    use crate::parse_jwt;

    const TEST_SECRET: &str = "register-service-secret-0123456789";

    #[derive(Clone)]
    struct FakeAccountStore {
        state: Arc<Mutex<FakeAccountStoreState>>,
    }

    #[derive(Default)]
    struct FakeAccountStoreState {
        accounts_by_email: HashMap<String, UserAccount>,
        lookup_calls: Vec<String>,
        register_calls: Vec<(String, String, String)>,
        register_result: FakeRegisterResult,
    }

    #[derive(Clone)]
    enum FakeRegisterResult {
        Success(Uuid, Uuid),
        Conflict,
    }

    impl Default for FakeRegisterResult {
        fn default() -> Self {
            Self::Success(Uuid::new_v4(), Uuid::new_v4())
        }
    }

    impl FakeAccountStore {
        fn with_register_result(result: Result<(Uuid, Uuid), DbError>) -> Self {
            let register_result = match result {
                Ok((user_id, workspace_id)) => FakeRegisterResult::Success(user_id, workspace_id),
                Err(DbError::Conflict) => FakeRegisterResult::Conflict,
                Err(other) => panic!("unsupported fake register result: {other}"),
            };

            Self {
                state: Arc::new(Mutex::new(FakeAccountStoreState {
                    register_result,
                    ..FakeAccountStoreState::default()
                })),
            }
        }

        fn with_account(
            email: &str,
            password_hash: &str,
            user_id: Uuid,
            workspace_id: Uuid,
        ) -> Self {
            let store = Self::with_register_result(Ok((Uuid::new_v4(), Uuid::new_v4())));
            store.insert_existing_account(email, password_hash, user_id, workspace_id);
            store
        }

        fn insert_existing_account(
            &self,
            email: &str,
            password_hash: &str,
            user_id: Uuid,
            workspace_id: Uuid,
        ) {
            self.state.lock().expect("state").accounts_by_email.insert(
                email.to_string(),
                UserAccount {
                    id: user_id,
                    email: email.to_string(),
                    password_hash: password_hash.to_string(),
                    workspace_id,
                    status: user_account::AccountStatus::Active,
                },
            );
        }

        fn register_call_count(&self) -> usize {
            self.state.lock().expect("state").register_calls.len()
        }

        fn lookup_call_count(&self) -> usize {
            self.state.lock().expect("state").lookup_calls.len()
        }
    }

    #[async_trait]
    impl AccountStore for FakeAccountStore {
        async fn find_active_by_email(&self, email: &str) -> Result<Option<UserAccount>, DbError> {
            let mut state = self.state.lock().expect("state");
            state.lookup_calls.push(email.to_string());
            Ok(state.accounts_by_email.get(email).cloned())
        }

        async fn register(
            &self,
            email: &str,
            password_hash: &str,
            workspace_name: &str,
        ) -> Result<(Uuid, Uuid), DbError> {
            let mut state = self.state.lock().expect("state");
            state.register_calls.push((
                email.to_string(),
                password_hash.to_string(),
                workspace_name.to_string(),
            ));
            match state.register_result.clone() {
                FakeRegisterResult::Success(user_id, workspace_id) => {
                    state.accounts_by_email.insert(
                        email.to_string(),
                        UserAccount {
                            id: user_id,
                            email: email.to_string(),
                            password_hash: password_hash.to_string(),
                            workspace_id,
                            status: user_account::AccountStatus::Active,
                        },
                    );
                    Ok((user_id, workspace_id))
                }
                FakeRegisterResult::Conflict => Err(DbError::Conflict),
            }
        }
    }

    type IssueCall = (Uuid, Uuid, Vec<String>);

    #[derive(Clone)]
    struct RecordingIssuer {
        calls: Arc<Mutex<Vec<IssueCall>>>,
        result: Arc<Mutex<FakeIssueResult>>,
    }

    #[derive(Clone)]
    enum FakeIssueResult {
        Success(String),
        Error(IssueError),
    }

    impl RecordingIssuer {
        fn with_result(result: Result<String, IssueError>) -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
                result: Arc::new(Mutex::new(match result {
                    Ok(token) => FakeIssueResult::Success(token),
                    Err(error) => FakeIssueResult::Error(error),
                })),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.lock().expect("calls").len()
        }
    }

    impl AccessTokenIssuer for RecordingIssuer {
        fn issue_access_token(
            &self,
            user_id: Uuid,
            workspace_id: Uuid,
            scopes: &[String],
        ) -> Result<String, IssueError> {
            self.calls
                .lock()
                .expect("calls")
                .push((user_id, workspace_id, scopes.to_vec()));
            match self.result.lock().expect("issuer result").clone() {
                FakeIssueResult::Success(token) => Ok(token),
                FakeIssueResult::Error(error) => Err(error),
            }
        }
    }

    #[tokio::test]
    async fn register_success_hashes_password_persists_and_issues_token() {
        let user_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let store = FakeAccountStore::with_register_result(Ok((user_id, workspace_id)));
        let issuer = Hs256Issuer::new(TEST_SECRET, Duration::from_secs(3600)).expect("issuer");
        let service = AuthService::new(store.clone(), issuer);

        let result = service
            .register(
                " owner@example.com ",
                "correct horse battery staple",
                " DubBridge ",
            )
            .await
            .expect("register succeeds");

        let state = store.state.lock().expect("state");
        let calls = &state.register_calls;
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "owner@example.com");
        assert_eq!(calls[0].2, "DubBridge");
        assert_ne!(calls[0].1, "correct horse battery staple");
        drop(state);

        let claims = parse_jwt(&result.token, TEST_SECRET).expect("parse token");
        assert_eq!(result.user_id, user_id);
        assert_eq!(result.workspace_id, workspace_id);
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.workspace_id, workspace_id.to_string());
        assert_eq!(claims.scope, DEFAULT_AUTH_SCOPES.join(" "));
    }

    #[tokio::test]
    async fn register_duplicate_email_returns_conflict_and_does_not_issue_token() {
        let store = FakeAccountStore::with_register_result(Err(DbError::Conflict));
        let issuer = RecordingIssuer::with_result(Ok("token".to_string()));
        let service = AuthService::new(store.clone(), issuer.clone());

        let error = service
            .register(
                "owner@example.com",
                "correct horse battery staple",
                "DubBridge",
            )
            .await
            .expect_err("duplicate email");

        assert!(matches!(error, AuthServiceError::Conflict));
        assert_eq!(store.register_call_count(), 1);
        assert_eq!(issuer.call_count(), 0);
    }

    #[tokio::test]
    async fn register_validation_error_short_circuits_before_db_and_issuer() {
        let store = FakeAccountStore::with_register_result(Ok((Uuid::new_v4(), Uuid::new_v4())));
        let issuer = RecordingIssuer::with_result(Ok("token".to_string()));
        let service = AuthService::new(store.clone(), issuer.clone());

        let error = service
            .register("owner@example.com", "short", "DubBridge")
            .await
            .expect_err("short password");

        assert!(matches!(
            error,
            AuthServiceError::Validation(CredentialError::PasswordTooShort { .. })
        ));
        assert_eq!(store.register_call_count(), 0);
        assert_eq!(issuer.call_count(), 0);
    }

    #[tokio::test]
    async fn register_validation_error_for_empty_fields_short_circuits_before_db() {
        let store = FakeAccountStore::with_register_result(Ok((Uuid::new_v4(), Uuid::new_v4())));
        let issuer = RecordingIssuer::with_result(Ok("token".to_string()));
        let service = AuthService::new(store.clone(), issuer.clone());

        let error = service
            .register("   ", "correct horse battery staple", "DubBridge")
            .await
            .expect_err("missing email");

        assert!(matches!(
            error,
            AuthServiceError::Validation(CredentialError::MissingField { field: "email" })
        ));
        assert_eq!(store.register_call_count(), 0);
        assert_eq!(issuer.call_count(), 0);
    }

    #[tokio::test]
    async fn login_success_issues_token_for_existing_account() {
        let user_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let password_hash = hash_password("correct horse battery staple").expect("hash");
        let store = FakeAccountStore::with_account(
            "owner@example.com",
            &password_hash,
            user_id,
            workspace_id,
        );
        let issuer = Hs256Issuer::new(TEST_SECRET, Duration::from_secs(3600)).expect("issuer");
        let service = AuthService::new(store.clone(), issuer);

        let result = service
            .login(" owner@example.com ", "correct horse battery staple")
            .await
            .expect("login succeeds");

        let claims = parse_jwt(&result.token, TEST_SECRET).expect("parse token");
        assert_eq!(store.lookup_call_count(), 1);
        assert_eq!(result.user_id, user_id);
        assert_eq!(result.workspace_id, workspace_id);
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.workspace_id, workspace_id.to_string());
        assert_eq!(claims.scope, DEFAULT_AUTH_SCOPES.join(" "));
    }

    #[tokio::test]
    async fn register_creates_account_immediately_usable_for_login() {
        let user_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let store = FakeAccountStore::with_register_result(Ok((user_id, workspace_id)));
        let issuer = Hs256Issuer::new(TEST_SECRET, Duration::from_secs(3600)).expect("issuer");
        let service = AuthService::new(store.clone(), issuer);

        let registered = service
            .register(
                "owner@example.com",
                "correct horse battery staple",
                "DubBridge",
            )
            .await
            .expect("register succeeds");
        let logged_in = service
            .login("owner@example.com", "correct horse battery staple")
            .await
            .expect("login succeeds");

        assert_eq!(registered.user_id, user_id);
        assert_eq!(registered.workspace_id, workspace_id);
        assert_eq!(logged_in.user_id, user_id);
        assert_eq!(logged_in.workspace_id, workspace_id);
        assert_eq!(store.register_call_count(), 1);
        assert_eq!(store.lookup_call_count(), 1);
    }

    #[tokio::test]
    async fn login_wrong_password_and_unknown_email_return_same_error() {
        let password_hash = hash_password("correct horse battery staple").expect("hash");
        let store = FakeAccountStore::with_account(
            "owner@example.com",
            &password_hash,
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        let issuer = RecordingIssuer::with_result(Ok("token".to_string()));
        let service = AuthService::new(store.clone(), issuer.clone());

        let wrong_password = service
            .login("owner@example.com", "definitely the wrong password")
            .await
            .expect_err("wrong password");
        let unknown_email = service
            .login("missing@example.com", "definitely the wrong password")
            .await
            .expect_err("unknown email");

        assert!(matches!(
            wrong_password,
            AuthServiceError::InvalidCredentials
        ));
        assert!(matches!(
            unknown_email,
            AuthServiceError::InvalidCredentials
        ));
        assert_eq!(store.lookup_call_count(), 2);
        assert_eq!(issuer.call_count(), 0);
    }

    #[tokio::test]
    async fn login_validation_error_short_circuits_before_db_and_issuer() {
        let store = FakeAccountStore::with_register_result(Ok((Uuid::new_v4(), Uuid::new_v4())));
        let issuer = RecordingIssuer::with_result(Ok("token".to_string()));
        let service = AuthService::new(store.clone(), issuer.clone());

        let error = service
            .login("owner@example.com", "   ")
            .await
            .expect_err("missing password");

        assert!(matches!(
            error,
            AuthServiceError::Validation(CredentialError::MissingField { field: "password" })
        ));
        assert_eq!(store.lookup_call_count(), 0);
        assert_eq!(issuer.call_count(), 0);
    }
}
