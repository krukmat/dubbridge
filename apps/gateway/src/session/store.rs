// P1-T3: SessionStore implementations — InMemory (tests) + Redis (runtime)

use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use redis::AsyncCommands;

use super::{SessionError, SessionId, SessionStore, StoredSession};

// ── InMemorySessionStore ──────────────────────────────────────────────────────

/// Deterministic in-memory store for unit and integration tests.
/// Expiry is checked lazily on resolve — no background sweep needed for tests.
pub struct InMemorySessionStore {
    sessions: Mutex<HashMap<SessionId, StoredSession>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create(
        &self,
        session: StoredSession,
        _absolute_ttl_secs: u64,
    ) -> Result<SessionId, SessionError> {
        let id = SessionId::generate();
        self.sessions.lock().unwrap().insert(id.clone(), session);
        Ok(id)
    }

    async fn resolve(
        &self,
        id: &SessionId,
        absolute_ttl_secs: u64,
        idle_ttl_secs: u64,
    ) -> Result<Option<StoredSession>, SessionError> {
        let mut map = self.sessions.lock().unwrap();
        match map.get(id) {
            None => Ok(None),
            Some(s) if s.is_expired(absolute_ttl_secs, idle_ttl_secs) => {
                map.remove(id);
                Ok(None)
            }
            Some(s) => Ok(Some(s.clone())),
        }
    }

    async fn touch(&self, id: &SessionId, _idle_ttl_secs: u64) -> Result<(), SessionError> {
        let mut map = self.sessions.lock().unwrap();
        if let Some(s) = map.get_mut(id) {
            s.touch();
        }
        Ok(())
    }

    async fn delete(&self, id: &SessionId) -> Result<(), SessionError> {
        self.sessions.lock().unwrap().remove(id);
        Ok(())
    }
}

// ── RedisSessionStore ─────────────────────────────────────────────────────────

/// Redis-backed session store for runtime use.
/// Sessions are stored as JSON under `dubbridge:session:{id}` with a Redis TTL
/// equal to `absolute_ttl_secs`. Idle expiry is enforced in Rust on resolve.
pub struct RedisSessionStore {
    conn: redis::aio::ConnectionManager,
}

impl RedisSessionStore {
    pub async fn new(redis_url: &str) -> Result<Self, SessionError> {
        let client =
            redis::Client::open(redis_url).map_err(|e| SessionError::Backend(e.to_string()))?;
        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;
        Ok(Self { conn })
    }

    fn key(id: &SessionId) -> String {
        format!("dubbridge:session:{}", id.as_str())
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn create(
        &self,
        session: StoredSession,
        absolute_ttl_secs: u64,
    ) -> Result<SessionId, SessionError> {
        let id = SessionId::generate();
        let key = Self::key(&id);
        let json = serde_json::to_string(&session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(&key, json, absolute_ttl_secs)
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;

        Ok(id)
    }

    async fn resolve(
        &self,
        id: &SessionId,
        absolute_ttl_secs: u64,
        idle_ttl_secs: u64,
    ) -> Result<Option<StoredSession>, SessionError> {
        let key = Self::key(id);
        let mut conn = self.conn.clone();

        let raw: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;

        let json = match raw {
            None => return Ok(None),
            Some(j) => j,
        };

        let session: StoredSession =
            serde_json::from_str(&json).map_err(|e| SessionError::Serialization(e.to_string()))?;

        if session.is_expired(absolute_ttl_secs, idle_ttl_secs) {
            conn.del::<_, ()>(&key)
                .await
                .map_err(|e| SessionError::Backend(e.to_string()))?;
            return Ok(None);
        }

        Ok(Some(session))
    }

    async fn touch(&self, id: &SessionId, idle_ttl_secs: u64) -> Result<(), SessionError> {
        let key = Self::key(id);
        let mut conn = self.conn.clone();

        // GET current value, update last_accessed, re-SET preserving Redis TTL
        let raw: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;

        let json = match raw {
            None => return Ok(()), // session already expired or missing — noop
            Some(j) => j,
        };

        let mut session: StoredSession =
            serde_json::from_str(&json).map_err(|e| SessionError::Serialization(e.to_string()))?;

        session.touch();

        let updated_json = serde_json::to_string(&session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        // Preserve the absolute-TTL Redis expiry; also ensure idle TTL is respected
        // by keeping whichever is smaller (the KEEPTTL option preserves Redis expiry)
        redis::cmd("SET")
            .arg(&key)
            .arg(&updated_json)
            .arg("KEEPTTL")
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;

        // Additionally cap at idle TTL: EXPIRE key idle_ttl_secs only if remaining > idle
        // This is a best-effort cap; exact idle enforcement happens on resolve() in Rust.
        let remaining_ttl: i64 = conn
            .ttl(&key)
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;

        if remaining_ttl > idle_ttl_secs as i64 {
            conn.expire::<_, ()>(&key, idle_ttl_secs as i64)
                .await
                .map_err(|e| SessionError::Backend(e.to_string()))?;
        }

        Ok(())
    }

    async fn delete(&self, id: &SessionId) -> Result<(), SessionError> {
        let key = Self::key(id);
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| SessionError::Backend(e.to_string()))?;
        Ok(())
    }
}
