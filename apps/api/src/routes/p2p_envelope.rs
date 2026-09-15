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
use dubbridge_auth::{AuthenticatedPrincipal, SharedTokenVerifier, authenticate_bearer};
use dubbridge_db::{error::DbError, p2p_envelope_repo::get_envelope_release_context};
use dubbridge_p2p::{
    Zeroizing,
    device_envelope::{DeviceEnvelopeBinding, seal_ck_for_device},
    key_wrap::{WrappedKey, unwrap_ck},
};
use serde::Serialize;
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
        Err(error) => return db_error_response(error),
    };
    let kek = match KekConfig::from_env() {
        Ok(kek) => kek,
        Err(()) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if context.sealed_kek_id != kek.id
        || context.sealed_kek_version <= 0
        || u32::try_from(context.sealed_kek_version).ok() != Some(kek.version)
    {
        return StatusCode::CONFLICT.into_response();
    }
    let nonce: [u8; 12] = match context.sealed_nonce.as_slice().try_into() {
        Ok(nonce) => nonce,
        Err(_) => return StatusCode::CONFLICT.into_response(),
    };
    let wrapped = WrappedKey {
        kek_id: context.sealed_kek_id.clone(),
        kek_version: kek.version,
        nonce,
        ciphertext: context.sealed_wrapped_ck.clone(),
    };
    let ck = match unwrap_ck(&wrapped, &kek.key) {
        Ok(ck) => ck,
        Err(_) => return StatusCode::CONFLICT.into_response(),
    };

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
    let envelope = match seal_ck_for_device(&ck, &context.device_public_key_spki, &binding) {
        Ok(envelope) => envelope,
        Err(_) => return StatusCode::CONFLICT.into_response(),
    };

    Json(DeviceEnvelopeResponse {
        profile_version: envelope.profile_version,
        key_id: context.device_key_id,
        encapsulated_key_base64: BASE64_STANDARD.encode(envelope.encapsulated_key),
        ciphertext_base64: BASE64_STANDARD.encode(envelope.ciphertext),
        binding_json: envelope.binding_json,
    })
    .into_response()
}

impl KekConfig {
    fn from_env() -> Result<Self, ()> {
        let id = required_env(KEK_ID_ENV)?;
        let version = required_env(KEK_VERSION_ENV)?.parse::<u32>().map_err(|_| ())?;
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
    use super::decode_32_byte_hex;

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
}
