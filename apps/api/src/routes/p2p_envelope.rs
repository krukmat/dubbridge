use std::{env, sync::Arc};

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use dubbridge_audit::emit_governance_audit;
use dubbridge_auth::{AuthenticatedPrincipal, SharedTokenVerifier, authenticate_bearer};
use dubbridge_db::{
    error::DbError,
    p2p_envelope_repo::{P2pEnvelopeReleaseContext, get_envelope_release_context},
};
use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
};
use dubbridge_p2p::{
    Zeroizing,
    device_envelope::{DeviceEnvelopeBinding, seal_ck_for_device},
    key_wrap::{WrappedKey, unwrap_ck},
};
use serde::Serialize;
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::state::AppState;

const KEK_HEX_ENV: &str = "DUBBRIDGE_P2P_KEK_HEX";
const KEK_ID_ENV: &str = "DUBBRIDGE_P2P_KEK_ID";
const KEK_VERSION_ENV: &str = "DUBBRIDGE_P2P_KEK_VERSION";

pub fn router(verifier: SharedTokenVerifier) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/p2p/authorizations/{id}/device-envelope",
            get(get_device_envelope),
        )
        .route_layer(middleware::from_fn_with_state(
            verifier,
            authenticate_bearer,
        ))
}

#[derive(Debug, Serialize)]
struct DeviceEnvelopeResponse {
    profile_version: &'static str,
    key_id: String,
    encapsulated_key_base64: String,
    ciphertext_base64: String,
    binding_json: String,
}

struct KekConfig {
    id: String,
    version: u32,
    key: Zeroizing<[u8; 32]>,
}

async fn get_device_envelope(
    State(state): State<Arc<AppState>>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(authorization_id): Path<Uuid>,
) -> Response {
    let context = match get_envelope_release_context(
        &state.pool,
        authorization_id,
        principal.subject_id,
        OffsetDateTime::now_utc(),
    )
    .await
    {
        Ok(context) => context,
        Err(error) => {
            return release_context_error_response(
                &state,
                authorization_id,
                principal.subject_id,
                error,
            )
            .await;
        }
    };

    let kek = match KekConfig::from_env() {
        Ok(kek) => kek,
        Err(()) => {
            return audited_context_denial(
                &state,
                &context,
                "kek_unavailable",
                StatusCode::SERVICE_UNAVAILABLE,
            )
            .await;
        }
    };

    let response = match build_device_envelope(&context, &kek) {
        Ok(response) => response,
        Err(error) => {
            return audited_context_denial(
                &state,
                &context,
                error.reason_code(),
                StatusCode::CONFLICT,
            )
            .await;
        }
    };

    let event = envelope_audit_event(
        &context,
        AuditEventKind::P2pDeviceEnvelopeReleased,
        Some(
            json!({
                "authorization_id": context.authorization_id,
                "invitation_id": context.invitation_id,
                "viewer_subject_id": context.viewer_subject_id,
                "device_id": context.device_id,
                "device_key_id": context.device_key_id,
            })
            .to_string(),
        ),
    );
    if emit_governance_audit(&state.pool, &event).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Json(response).into_response()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnvelopeBuildError {
    KekMismatch,
    InvalidNonce,
    WrappedKeyInvalid,
    DeviceKeyInvalid,
}

impl EnvelopeBuildError {
    const fn reason_code(self) -> &'static str {
        match self {
            Self::KekMismatch => "kek_mismatch",
            Self::InvalidNonce => "sealed_nonce_invalid",
            Self::WrappedKeyInvalid => "wrapped_ck_invalid",
            Self::DeviceKeyInvalid => "device_key_invalid",
        }
    }
}

fn build_device_envelope(
    context: &P2pEnvelopeReleaseContext,
    kek: &KekConfig,
) -> Result<DeviceEnvelopeResponse, EnvelopeBuildError> {
    if context.sealed_kek_id != kek.id
        || context.sealed_kek_version <= 0
        || u32::try_from(context.sealed_kek_version).ok() != Some(kek.version)
    {
        return Err(EnvelopeBuildError::KekMismatch);
    }

    let nonce: [u8; 12] = context
        .sealed_nonce
        .as_slice()
        .try_into()
        .map_err(|_| EnvelopeBuildError::InvalidNonce)?;
    let wrapped = WrappedKey {
        kek_id: context.sealed_kek_id.clone(),
        kek_version: kek.version,
        nonce,
        ciphertext: context.sealed_wrapped_ck.clone(),
    };
    let ck = unwrap_ck(&wrapped, &kek.key).map_err(|_| EnvelopeBuildError::WrappedKeyInvalid)?;

    let invitation_id = context.invitation_id.to_string();
    let viewer_id = context.viewer_subject_id.to_string();
    let asset_id = context.asset_id.to_string();
    let publication_id = context.publication_id.to_string();
    let lineage_id = context.lineage_id.to_string();
    let authorization_id = context.authorization_id.to_string();
    let binding = DeviceEnvelopeBinding::k1(
        &context.device_key_id,
        &invitation_id,
        &viewer_id,
        &asset_id,
        &publication_id,
        &lineage_id,
        &authorization_id,
        context.expires_at.unix_timestamp(),
    );
    let envelope = seal_ck_for_device(&ck, &context.device_public_key_spki, &binding)
        .map_err(|_| EnvelopeBuildError::DeviceKeyInvalid)?;

    Ok(DeviceEnvelopeResponse {
        profile_version: envelope.profile_version,
        key_id: context.device_key_id.clone(),
        encapsulated_key_base64: BASE64_STANDARD.encode(envelope.encapsulated_key),
        ciphertext_base64: BASE64_STANDARD.encode(envelope.ciphertext),
        binding_json: envelope.binding_json,
    })
}

async fn release_context_error_response(
    state: &AppState,
    authorization_id: Uuid,
    viewer_subject_id: Uuid,
    error: DbError,
) -> Response {
    if matches!(&error, DbError::NotFound | DbError::Conflict) {
        let event = AuditEvent::new_p3_event(
            None,
            AuditEventKind::P2pAudienceAccessDenied,
            authorization_id,
            None,
            None,
            Some(
                json!({
                    "operation": "device_envelope",
                    "reason": "authorization_unavailable",
                    "authorization_id": authorization_id,
                    "viewer_subject_id": viewer_subject_id,
                })
                .to_string(),
            ),
        );
        if emit_governance_audit(&state.pool, &event).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    db_error_response(error)
}

async fn audited_context_denial(
    state: &AppState,
    context: &P2pEnvelopeReleaseContext,
    reason: &'static str,
    status: StatusCode,
) -> Response {
    let event = envelope_audit_event(
        context,
        AuditEventKind::P2pAudienceAccessDenied,
        Some(
            json!({
                "operation": "device_envelope",
                "reason": reason,
                "authorization_id": context.authorization_id,
                "device_id": context.device_id,
            })
            .to_string(),
        ),
    );
    if emit_governance_audit(&state.pool, &event).await.is_err() {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    } else {
        status.into_response()
    }
}

fn envelope_audit_event(
    context: &P2pEnvelopeReleaseContext,
    event_kind: AuditEventKind,
    detail: Option<String>,
) -> AuditEvent {
    AuditEvent::new_p3_event(
        Some(AssetId(context.asset_id)),
        event_kind,
        context.authorization_id,
        Some(context.publication_id),
        Some(context.lineage_id),
        detail,
    )
}

impl KekConfig {
    fn from_env() -> Result<Self, ()> {
        let id = required_env(KEK_ID_ENV)?;
        let version = required_env(KEK_VERSION_ENV)?
            .parse::<u32>()
            .map_err(|_| ())?;
        if version == 0 || version > i32::MAX as u32 {
            return Err(());
        }
        let key_hex = Zeroizing::new(required_env(KEK_HEX_ENV)?);
        let key = decode_32_byte_hex(&key_hex)?;
        Ok(Self { id, version, key })
    }
}

fn required_env(name: &str) -> Result<String, ()> {
    let value = env::var(name).map_err(|_| ())?;
    if value.trim().is_empty() {
        Err(())
    } else {
        Ok(value)
    }
}

fn decode_32_byte_hex(value: &str) -> Result<Zeroizing<[u8; 32]>, ()> {
    if value.len() != 64 {
        return Err(());
    }
    let mut output = Zeroizing::new([0u8; 32]);
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|_| ())?;
    }
    Ok(output)
}

fn db_error_response(error: DbError) -> Response {
    match error {
        DbError::NotFound => StatusCode::NOT_FOUND.into_response(),
        DbError::Conflict => StatusCode::CONFLICT.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_p2p::key_wrap::wrap_ck;

    fn valid_device_spki() -> Vec<u8> {
        BASE64_STANDARD
            .decode("MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEaxfR8uEsQkf4vOblY6RA8ncDfYEt6zOg9KE5RdiYwpZP40Li/hp/m47n60p8D54WK84zV2sxXs7LtkBoN79R9Q==")
            .expect("valid P-256 SPKI fixture")
    }

    fn release_context(kek: &[u8; 32]) -> P2pEnvelopeReleaseContext {
        let wrapped = wrap_ck(&[7_u8; 32], kek, "kek-test", 1).expect("wrap CK");
        P2pEnvelopeReleaseContext {
            authorization_id: Uuid::new_v4(),
            invitation_id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            publication_id: Uuid::new_v4(),
            lineage_id: Uuid::new_v4(),
            viewer_subject_id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            device_key_id: "dubbridge-p2p-k1-v1".to_owned(),
            device_public_key_spki: valid_device_spki(),
            expires_at: OffsetDateTime::now_utc() + time::Duration::minutes(5),
            sealed_kek_id: wrapped.kek_id,
            sealed_kek_version: i32::try_from(wrapped.kek_version).expect("version fits"),
            sealed_nonce: wrapped.nonce.to_vec(),
            sealed_wrapped_ck: wrapped.ciphertext,
        }
    }

    fn kek_config(key: [u8; 32]) -> KekConfig {
        KekConfig {
            id: "kek-test".to_owned(),
            version: 1,
            key: Zeroizing::new(key),
        }
    }

    #[test]
    fn envelope_kek_decoder_accepts_exact_256_bit_hex() {
        let decoded = decode_32_byte_hex(&"ab".repeat(32)).expect("decode");
        assert_eq!(*decoded, [0xab; 32]);
    }

    #[test]
    fn envelope_kek_decoder_rejects_wrong_length_or_non_hex() {
        assert!(decode_32_byte_hex("ab").is_err());
        assert!(decode_32_byte_hex(&"zz".repeat(32)).is_err());
    }

    #[test]
    fn envelope_builder_binds_exact_release_identity() {
        let key = [3_u8; 32];
        let context = release_context(&key);
        let response = build_device_envelope(&context, &kek_config(key)).expect("build envelope");
        let binding: serde_json::Value =
            serde_json::from_str(&response.binding_json).expect("parse binding");

        assert_eq!(response.profile_version, "p2p-k1-hpke-v1");
        assert_eq!(response.key_id, context.device_key_id);
        assert_eq!(binding["device_key_id"], context.device_key_id);
        assert_eq!(binding["invitation_id"], context.invitation_id.to_string());
        assert_eq!(binding["viewer_id"], context.viewer_subject_id.to_string());
        assert_eq!(binding["asset_id"], context.asset_id.to_string());
        assert_eq!(
            binding["publication_id"],
            context.publication_id.to_string()
        );
        assert_eq!(binding["lineage_id"], context.lineage_id.to_string());
        assert_eq!(
            binding["authorization_id"],
            context.authorization_id.to_string()
        );
        assert_eq!(
            binding["expires_at_unix"],
            context.expires_at.unix_timestamp()
        );
    }

    #[test]
    fn envelope_builder_fails_closed_on_kek_nonce_wrapped_ck_or_device_key_drift() {
        let key = [3_u8; 32];

        let mut kek_mismatch = release_context(&key);
        kek_mismatch.sealed_kek_id = "other-kek".to_owned();
        assert_eq!(
            build_device_envelope(&kek_mismatch, &kek_config(key)).unwrap_err(),
            EnvelopeBuildError::KekMismatch
        );

        let mut bad_nonce = release_context(&key);
        bad_nonce.sealed_nonce = vec![0_u8; 11];
        assert_eq!(
            build_device_envelope(&bad_nonce, &kek_config(key)).unwrap_err(),
            EnvelopeBuildError::InvalidNonce
        );

        let mut bad_wrapped = release_context(&key);
        bad_wrapped.sealed_wrapped_ck[0] ^= 0xff;
        assert_eq!(
            build_device_envelope(&bad_wrapped, &kek_config(key)).unwrap_err(),
            EnvelopeBuildError::WrappedKeyInvalid
        );

        let mut bad_device = release_context(&key);
        bad_device.device_public_key_spki = vec![1, 2, 3];
        assert_eq!(
            build_device_envelope(&bad_device, &kek_config(key)).unwrap_err(),
            EnvelopeBuildError::DeviceKeyInvalid
        );
    }

    #[test]
    fn envelope_audit_event_contains_only_bounded_release_identifiers() {
        let context = release_context(&[3_u8; 32]);
        let event = envelope_audit_event(
            &context,
            AuditEventKind::P2pDeviceEnvelopeReleased,
            Some(
                json!({
                    "authorization_id": context.authorization_id,
                    "device_id": context.device_id,
                })
                .to_string(),
            ),
        );

        assert!(event.has_valid_p3_correlation());
        let detail = event.detail.expect("detail");
        assert!(!detail.contains("wrapped"));
        assert!(!detail.contains("nonce"));
        assert!(!detail.contains("kek"));
        assert!(!detail.contains("ck"));
    }
}
