//! Stable, capability-neutral DubBridge work-item identity for Fenix S2S.
//! Both durable PostgreSQL intent records and the HTTPS connector MUST use
//! exactly the same versioned UUIDv5 namespaces. Never regenerate a fresh
//! random operation ID for a retry or reinterpret this as an agent run.

use uuid::Uuid;

const FENIX_OPERATION_NAMESPACE: Uuid =
    Uuid::from_u128(0x8a06362792aa4fe291199ce58636f012);
const FENIX_TRACE_NAMESPACE: Uuid =
    Uuid::from_u128(0xcac39fc79892481ab9fed8c12b0b0a7e);

pub fn derive_fenix_operation_identity(work_item_id: Uuid, capability_key: &str) -> (Uuid, Uuid) {
    let name = format!("{work_item_id}:{capability_key}");
    (
        Uuid::new_v5(&FENIX_OPERATION_NAMESPACE, name.as_bytes()),
        Uuid::new_v5(&FENIX_TRACE_NAMESPACE, name.as_bytes()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_across_reloads_and_distinct_across_capabilities() {
        let id = Uuid::parse_str("123e4567-e89b-42d3-a456-426614174000").unwrap();
        let first = derive_fenix_operation_identity(id, "integration.diagnostics");
        let replay = derive_fenix_operation_identity(id, "integration.diagnostics");
        let another = derive_fenix_operation_identity(id, "integration.audit");
        assert_eq!(first, replay);
        assert_ne!(first.0, another.0);
        assert_ne!(first.0, first.1);
        assert_eq!(first.0.get_version_num(), 5);
        assert_eq!(first.1.get_version_num(), 5);
    }
}
