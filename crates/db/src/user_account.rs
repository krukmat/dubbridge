use std::error::Error as StdError;

use sqlx::{
    Decode, PgPool, Postgres, Transaction, Type,
    postgres::{PgTypeInfo, PgValueRef},
};
use uuid::Uuid;

use dubbridge_domain::workspace::Organization;

use crate::error::DbError;
use crate::workspace_repo;

const FIND_ACTIVE_BY_EMAIL_SQL: &str = r#"
        SELECT id, email, password_hash, workspace_id, status
        FROM user_account
        WHERE email = $1 AND status = 'active'
        LIMIT 1
        "#;

const INSERT_ACCOUNT_SQL: &str = r#"
        INSERT INTO user_account (id, email, password_hash, workspace_id)
        VALUES ($1, $2, $3, $4)
        "#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountStatus {
    Active,
    Unknown(String),
}

impl TryFrom<String> for AccountStatus {
    type Error = DbError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(match value.as_str() {
            "active" => Self::Active,
            _ => Self::Unknown(value),
        })
    }
}

impl Type<Postgres> for AccountStatus {
    fn type_info() -> PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <String as Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for AccountStatus {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn StdError + Send + Sync + 'static>> {
        let raw = <String as Decode<Postgres>>::decode(value)?;
        Ok(match raw.as_str() {
            "active" => Self::Active,
            _ => Self::Unknown(raw),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct UserAccount {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub workspace_id: Uuid,
    pub status: AccountStatus,
}

fn require_known_status(status: AccountStatus) -> Result<AccountStatus, DbError> {
    match status {
        AccountStatus::Active => Ok(AccountStatus::Active),
        AccountStatus::Unknown(value) => Err(DbError::UnknownStoredValue {
            field: "user_account.status",
            value,
        }),
    }
}

fn ensure_active_account(account: UserAccount) -> Result<UserAccount, DbError> {
    let status = require_known_status(account.status)?;
    Ok(UserAccount { status, ..account })
}

fn map_lookup_result(row: Option<UserAccount>) -> Result<Option<UserAccount>, DbError> {
    row.map(ensure_active_account).transpose()
}

fn build_registration_result(
    workspace_id: Uuid,
    account_result: Result<Uuid, DbError>,
) -> Result<(Uuid, Uuid), DbError> {
    let account_id = account_result?;
    Ok((account_id, workspace_id))
}

fn should_commit_registration(result: &Result<(Uuid, Uuid), DbError>) -> bool {
    result.is_ok()
}

pub async fn find_active_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserAccount>, DbError> {
    let row = sqlx::query_as::<_, UserAccount>(FIND_ACTIVE_BY_EMAIL_SQL)
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(DbError::QueryFailed)?;

    map_lookup_result(row)
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .as_deref()
        == Some("23505")
}

fn map_insert_account_error(error: sqlx::Error) -> DbError {
    if is_unique_violation(&error) {
        DbError::Conflict
    } else {
        DbError::QueryFailed(error)
    }
}

pub async fn insert_workspace(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
) -> Result<Uuid, DbError> {
    let organization = Organization::new(name.to_string());
    workspace_repo::insert_org_tx(tx, &organization).await?;
    Ok(organization.id.0)
}

pub async fn insert_account(
    tx: &mut Transaction<'_, Postgres>,
    email: &str,
    password_hash: &str,
    workspace_id: Uuid,
) -> Result<Uuid, DbError> {
    let account_id = Uuid::new_v4();
    sqlx::query(INSERT_ACCOUNT_SQL)
        .bind(account_id)
        .bind(email)
        .bind(password_hash)
        .bind(workspace_id)
        .execute(&mut **tx)
        .await
        .map_err(map_insert_account_error)?;
    Ok(account_id)
}

pub async fn register(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    workspace_name: &str,
) -> Result<(Uuid, Uuid), DbError> {
    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    let workspace_id = insert_workspace(&mut tx, workspace_name).await?;
    let registration = build_registration_result(
        workspace_id,
        insert_account(&mut tx, email, password_hash, workspace_id).await,
    );
    if should_commit_registration(&registration) {
        tx.commit().await.map_err(DbError::QueryFailed)?;
    }
    registration
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use sqlx::migrate::Migrator;

    static MIGRATOR: Migrator = sqlx::migrate!("../../infra/migrations");

    async fn test_pool() -> Option<PgPool> {
        let database_url = match env::var("DUBBRIDGE_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => return None,
        };

        let pool = match PgPool::connect(&database_url).await {
            Ok(pool) => pool,
            Err(_) => return None,
        };

        MIGRATOR.run(&pool).await.expect("migrations");
        sqlx::query("TRUNCATE TABLE user_account, organizations RESTART IDENTITY CASCADE")
            .execute(&pool)
            .await
            .expect("truncate auth tables");

        Some(pool)
    }

    async fn insert_org(pool: &PgPool, org_id: Uuid) {
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, created_at, updated_at)
            VALUES ($1, $2, now(), now())
            "#,
        )
        .bind(org_id)
        .bind(format!("Org-{org_id}"))
        .execute(pool)
        .await
        .expect("insert organization");
    }

    async fn insert_account(
        pool: &PgPool,
        account_id: Uuid,
        email: &str,
        workspace_id: Uuid,
        status: &str,
    ) {
        sqlx::query(
            r#"
            INSERT INTO user_account (id, email, password_hash, workspace_id, status)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(account_id)
        .bind(email)
        .bind("bcrypt$demo")
        .bind(workspace_id)
        .bind(status)
        .execute(pool)
        .await
        .expect("insert account");
    }

    async fn count_organizations(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM organizations")
            .fetch_one(pool)
            .await
            .expect("count organizations")
    }

    async fn count_accounts(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM user_account")
            .fetch_one(pool)
            .await
            .expect("count accounts")
    }

    #[test]
    fn account_status_known_variant_maps_to_active() {
        assert_eq!(
            AccountStatus::try_from("active".to_string()).unwrap(),
            AccountStatus::Active
        );
    }

    #[test]
    fn account_status_unknown_variant_is_retained_for_fail_closed_handling() {
        assert_eq!(
            AccountStatus::try_from("suspended".to_string()).unwrap(),
            AccountStatus::Unknown("suspended".to_string())
        );
    }

    #[test]
    fn require_known_status_unknown_value_fails_closed() {
        let err =
            require_known_status(AccountStatus::Unknown("suspended".to_string())).unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "user_account.status",
                ..
            }
        ));
    }

    #[test]
    fn ensure_active_account_accepts_active_status() {
        let account = UserAccount {
            id: Uuid::new_v4(),
            email: "owner@example.com".to_string(),
            password_hash: "bcrypt$demo".to_string(),
            workspace_id: Uuid::new_v4(),
            status: AccountStatus::Active,
        };

        assert_eq!(ensure_active_account(account.clone()).unwrap(), account);
    }

    #[test]
    fn map_lookup_result_none_returns_none() {
        assert!(map_lookup_result(None).unwrap().is_none());
    }

    #[test]
    fn build_registration_result_success_preserves_ids() {
        let workspace_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        assert_eq!(
            build_registration_result(workspace_id, Ok(account_id)).unwrap(),
            (account_id, workspace_id)
        );
    }

    #[test]
    fn build_registration_result_conflict_propagates() {
        let error = build_registration_result(Uuid::new_v4(), Err(DbError::Conflict)).unwrap_err();
        assert!(matches!(error, DbError::Conflict));
    }

    #[test]
    fn should_commit_registration_only_on_success() {
        assert!(should_commit_registration(&Ok((
            Uuid::new_v4(),
            Uuid::new_v4()
        ))));
        assert!(!should_commit_registration(&Err(DbError::Conflict)));
    }

    #[test]
    fn find_active_by_email_query_filters_to_active_status() {
        assert!(FIND_ACTIVE_BY_EMAIL_SQL.contains("status = 'active'"));
    }

    #[test]
    fn insert_account_query_uses_workspace_linkage() {
        assert!(INSERT_ACCOUNT_SQL.contains("workspace_id"));
    }

    #[test]
    fn map_insert_account_error_non_unique_stays_query_failed() {
        let error = sqlx::Error::RowNotFound;
        assert!(matches!(
            map_insert_account_error(error),
            DbError::QueryFailed(sqlx::Error::RowNotFound)
        ));
    }

    #[tokio::test]
    async fn find_active_by_email_returns_active_account() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let workspace_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        insert_org(&pool, workspace_id).await;
        insert_account(
            &pool,
            account_id,
            "owner@example.com",
            workspace_id,
            "active",
        )
        .await;

        let account = find_active_by_email(&pool, "owner@example.com")
            .await
            .expect("query succeeds")
            .expect("account exists");

        assert_eq!(account.id, account_id);
        assert_eq!(account.email, "owner@example.com");
        assert_eq!(account.password_hash, "bcrypt$demo");
        assert_eq!(account.workspace_id, workspace_id);
        assert_eq!(account.status, AccountStatus::Active);
    }

    #[tokio::test]
    async fn find_active_by_email_returns_none_for_suspended_account() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let workspace_id = Uuid::new_v4();
        insert_org(&pool, workspace_id).await;
        insert_account(
            &pool,
            Uuid::new_v4(),
            "suspended@example.com",
            workspace_id,
            "suspended",
        )
        .await;

        let account = find_active_by_email(&pool, "suspended@example.com")
            .await
            .expect("query succeeds");

        assert!(account.is_none());
    }

    #[tokio::test]
    async fn find_active_by_email_returns_none_for_unknown_email() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let account = find_active_by_email(&pool, "missing@example.com")
            .await
            .expect("query succeeds");

        assert!(account.is_none());
    }

    #[tokio::test]
    async fn register_creates_workspace_and_account() {
        let Some(pool) = test_pool().await else {
            return;
        };

        let (account_id, workspace_id) =
            register(&pool, "owner@example.com", "bcrypt$demo", "DubBridge")
                .await
                .expect("register succeeds");

        let account = find_active_by_email(&pool, "owner@example.com")
            .await
            .expect("query succeeds")
            .expect("account exists");

        assert_eq!(account.id, account_id);
        assert_eq!(account.workspace_id, workspace_id);
        assert_eq!(count_organizations(&pool).await, 1);
        assert_eq!(count_accounts(&pool).await, 1);
    }

    #[tokio::test]
    async fn register_duplicate_email_returns_conflict_and_rolls_back_workspace() {
        let Some(pool) = test_pool().await else {
            return;
        };

        register(&pool, "owner@example.com", "bcrypt$demo", "First Workspace")
            .await
            .expect("initial register succeeds");
        let organizations_before = count_organizations(&pool).await;
        let accounts_before = count_accounts(&pool).await;

        let error = register(
            &pool,
            "owner@example.com",
            "bcrypt$demo-2",
            "Orphaned Workspace",
        )
        .await
        .expect_err("duplicate email must conflict");

        assert!(matches!(error, DbError::Conflict));
        assert_eq!(count_organizations(&pool).await, organizations_before);
        assert_eq!(count_accounts(&pool).await, accounts_before);
    }
}
