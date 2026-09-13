use std::collections::BTreeMap;

use dubbridge_domain::haa::{
    ActionSpec, ApprovalChallenge, ApprovalChallengePackage, ApprovalIntent, CanonicalValue,
    DisplayClaim, SignatureEnvelope, HAA_PROTOCOL_V1,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

const ACTION_DIGEST_DOMAIN: &[u8] = b"dubbridge.haa.action.v1\0";
const INTENT_DIGEST_DOMAIN: &[u8] = b"dubbridge.haa.intent.v1\0";
const CHALLENGE_DIGEST_DOMAIN: &[u8] = b"dubbridge.haa.challenge.v1\0";

#[derive(Debug, thiserror::Error)]
pub enum HaaServiceError {
    #[error("HAA canonical serialization failed: {0}")]
    CanonicalSerialization(String),
    #[error("HAA signing failed: {0}")]
    Signing(String),
    #[error("approval expiry must be after issue time")]
    InvalidExpiry,
}

pub trait ChallengeSigner: Send + Sync {
    fn sign_digest(&self, digest_hex: &str) -> Result<SignatureEnvelope, HaaServiceError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestedIntent {
    pub canonical_action: String,
    pub canonical_intent: String,
    pub action_digest: String,
    pub intent_digest: String,
}

#[derive(Serialize)]
struct IntentDigestView<'a> {
    protocol_version: &'static str,
    action_digest: &'a str,
    requester: &'a dubbridge_domain::haa::PrincipalRef,
    audience: &'a dubbridge_domain::haa::PrincipalRef,
    policy: &'a dubbridge_domain::haa::PolicyRef,
    created_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

pub fn digest_intent(intent: &ApprovalIntent) -> Result<DigestedIntent, HaaServiceError> {
    let canonical_action = canonical_json(&intent.action)?;
    let action_digest = hash_domain(ACTION_DIGEST_DOMAIN, canonical_action.as_bytes());
    let canonical_intent = canonical_json(intent)?;
    let digest_view = IntentDigestView {
        protocol_version: HAA_PROTOCOL_V1,
        action_digest: &action_digest,
        requester: &intent.requester,
        audience: &intent.audience,
        policy: &intent.policy,
        created_at: intent.created_at,
        expires_at: intent.expires_at,
    };
    let digest_bytes = canonical_json(&digest_view)?;
    let intent_digest = hash_domain(INTENT_DIGEST_DOMAIN, digest_bytes.as_bytes());
    Ok(DigestedIntent {
        canonical_action,
        canonical_intent,
        action_digest,
        intent_digest,
    })
}

pub fn challenge_digest(challenge: &ApprovalChallenge) -> Result<String, HaaServiceError> {
    let canonical = canonical_json(challenge)?;
    Ok(hash_domain(
        CHALLENGE_DIGEST_DOMAIN,
        canonical.as_bytes(),
    ))
}

pub fn build_challenge_package(
    signer: &dyn ChallengeSigner,
    request_id: Uuid,
    intent: &ApprovalIntent,
    digested: &DigestedIntent,
    approver: dubbridge_domain::haa::PrincipalRef,
    issued_at: OffsetDateTime,
    expires_at: OffsetDateTime,
) -> Result<ApprovalChallengePackage, HaaServiceError> {
    if expires_at <= issued_at || expires_at > intent.expires_at {
        return Err(HaaServiceError::InvalidExpiry);
    }
    let challenge = ApprovalChallenge {
        protocol_version: HAA_PROTOCOL_V1.to_owned(),
        request_id,
        action_digest: digested.action_digest.clone(),
        intent_digest: digested.intent_digest.clone(),
        nonce: fresh_nonce(),
        approver,
        audience: intent.audience.clone(),
        policy: intent.policy.clone(),
        display_claims: display_claims_for_action(&intent.action)?,
        issued_at,
        expires_at,
    };
    let digest = challenge_digest(&challenge)?;
    let haa_signature = signer.sign_digest(&digest)?;
    Ok(ApprovalChallengePackage {
        challenge,
        haa_signature,
    })
}

pub fn display_claims_for_action(action: &ActionSpec) -> Result<Vec<DisplayClaim>, HaaServiceError> {
    let mut claims = vec![
        DisplayClaim {
            label: "Action".to_owned(),
            value: action.action_type.clone(),
        },
        DisplayClaim {
            label: "Resource".to_owned(),
            value: action.resource.clone(),
        },
    ];
    if let Some(environment) = &action.environment {
        claims.push(DisplayClaim {
            label: "Environment".to_owned(),
            value: environment.clone(),
        });
    }
    for (name, value) in &action.parameters {
        claims.push(DisplayClaim {
            label: format!("Parameter: {name}"),
            value: canonical_json(value)?,
        });
    }
    for precondition in &action.preconditions {
        claims.push(DisplayClaim {
            label: format!("Precondition: {}", precondition.name),
            value: canonical_json(&precondition.expected)?,
        });
    }
    Ok(claims)
}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, HaaServiceError> {
    let value = serde_json::to_value(value)
        .map_err(|error| HaaServiceError::CanonicalSerialization(error.to_string()))?;
    let mut output = String::new();
    write_canonical_value(&value, &mut output)?;
    Ok(output)
}

fn write_canonical_value(value: &Value, output: &mut String) -> Result<(), HaaServiceError> {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(flag) => output.push_str(if *flag { "true" } else { "false" }),
        Value::Number(number) => output.push_str(&number.to_string()),
        Value::String(text) => output.push_str(
            &serde_json::to_string(text)
                .map_err(|error| HaaServiceError::CanonicalSerialization(error.to_string()))?,
        ),
        Value::Array(items) => write_canonical_array(items, output)?,
        Value::Object(object) => write_canonical_object(object, output)?,
    }
    Ok(())
}

fn write_canonical_array(items: &[Value], output: &mut String) -> Result<(), HaaServiceError> {
    output.push('[');
    for (index, item) in items.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        write_canonical_value(item, output)?;
    }
    output.push(']');
    Ok(())
}

fn write_canonical_object(
    object: &serde_json::Map<String, Value>,
    output: &mut String,
) -> Result<(), HaaServiceError> {
    output.push('{');
    let sorted: BTreeMap<&str, &Value> = object
        .iter()
        .map(|(key, value)| (key.as_str(), value))
        .collect();
    for (index, (key, value)) in sorted.into_iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        output.push_str(
            &serde_json::to_string(key)
                .map_err(|error| HaaServiceError::CanonicalSerialization(error.to_string()))?,
        );
        output.push(':');
        write_canonical_value(value, output)?;
    }
    output.push('}');
    Ok(())
}

fn hash_domain(domain: &[u8], payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(payload);
    hex_lower(&hasher.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn fresh_nonce() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_domain::haa::{
        ActionPrecondition, PolicyRef, PrincipalRef, SignatureEnvelope,
    };
    use time::macros::datetime;

    struct TestSigner;

    impl ChallengeSigner for TestSigner {
        fn sign_digest(&self, digest_hex: &str) -> Result<SignatureEnvelope, HaaServiceError> {
            Ok(SignatureEnvelope {
                algorithm: "test-only".to_owned(),
                key_id: "test-key".to_owned(),
                signature_b64: digest_hex.to_owned(),
            })
        }
    }

    fn principal(kind: &str, id: &str) -> PrincipalRef {
        PrincipalRef {
            kind: kind.to_owned(),
            id: id.to_owned(),
        }
    }

    fn action(replicas: i64) -> ActionSpec {
        ActionSpec {
            schema: "deployment.execute.v1".to_owned(),
            action_type: "deployment.execute".to_owned(),
            resource: "project/localdevengine".to_owned(),
            environment: Some("production".to_owned()),
            parameters: BTreeMap::from([
                ("region".to_owned(), CanonicalValue::from("eu")),
                ("replicas".to_owned(), CanonicalValue::from(replicas)),
            ]),
            preconditions: vec![ActionPrecondition {
                name: "commit".to_owned(),
                expected: CanonicalValue::from("17ac839"),
            }],
        }
    }

    fn intent(action: ActionSpec) -> ApprovalIntent {
        ApprovalIntent {
            action,
            requester: principal("agent", "release-manager"),
            audience: principal("executor", "deploy-service"),
            policy: PolicyRef {
                id: "production-deploy".to_owned(),
                version: "1".to_owned(),
            },
            created_at: datetime!(2026-09-13 00:00 UTC),
            expires_at: datetime!(2026-09-13 00:05 UTC),
        }
    }

    #[test]
    fn canonical_json_sorts_nested_object_keys() {
        let first = CanonicalValue::Object(BTreeMap::from([
            ("z".to_owned(), CanonicalValue::from(1_i64)),
            ("a".to_owned(), CanonicalValue::from(2_i64)),
        ]));
        assert_eq!(canonical_json(&first).expect("canonical"), r#"{"a":2,"z":1}"#);
    }

    #[test]
    fn semantic_action_change_changes_action_and_intent_digest() {
        let first = digest_intent(&intent(action(2))).expect("digest");
        let second = digest_intent(&intent(action(3))).expect("digest");
        assert_ne!(first.action_digest, second.action_digest);
        assert_ne!(first.intent_digest, second.intent_digest);
    }

    #[test]
    fn audience_change_changes_intent_digest_without_changing_action_digest() {
        let first_intent = intent(action(2));
        let mut second_intent = first_intent.clone();
        second_intent.audience = principal("executor", "other-service");
        let first = digest_intent(&first_intent).expect("digest");
        let second = digest_intent(&second_intent).expect("digest");
        assert_eq!(first.action_digest, second.action_digest);
        assert_ne!(first.intent_digest, second.intent_digest);
    }

    #[test]
    fn display_claims_cover_parameters_and_preconditions() {
        let claims = display_claims_for_action(&action(2)).expect("claims");
        assert!(claims.iter().any(|claim| claim.label == "Parameter: replicas"));
        assert!(claims.iter().any(|claim| claim.label == "Parameter: region"));
        assert!(claims.iter().any(|claim| claim.label == "Precondition: commit"));
    }

    #[test]
    fn challenge_nonce_changes_challenge_digest() {
        let intent = intent(action(2));
        let digested = digest_intent(&intent).expect("digest");
        let first = build_challenge_package(
            &TestSigner,
            Uuid::nil(),
            &intent,
            &digested,
            principal("human", "matias"),
            datetime!(2026-09-13 00:01 UTC),
            datetime!(2026-09-13 00:02 UTC),
        )
        .expect("first challenge");
        let second = build_challenge_package(
            &TestSigner,
            Uuid::nil(),
            &intent,
            &digested,
            principal("human", "matias"),
            datetime!(2026-09-13 00:01 UTC),
            datetime!(2026-09-13 00:02 UTC),
        )
        .expect("second challenge");
        assert_ne!(first.challenge.nonce, second.challenge.nonce);
        assert_ne!(
            challenge_digest(&first.challenge).expect("digest"),
            challenge_digest(&second.challenge).expect("digest")
        );
    }

    #[test]
    fn challenge_expiry_cannot_outlive_intent() {
        let intent = intent(action(2));
        let digested = digest_intent(&intent).expect("digest");
        let result = build_challenge_package(
            &TestSigner,
            Uuid::nil(),
            &intent,
            &digested,
            principal("human", "matias"),
            datetime!(2026-09-13 00:01 UTC),
            datetime!(2026-09-13 00:06 UTC),
        );
        assert!(matches!(result, Err(HaaServiceError::InvalidExpiry)));
    }
}
