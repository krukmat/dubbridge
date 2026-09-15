use aws_lc_rs::{
    aead::{Aad, AES_256_GCM, LessSafeKey, Nonce, UnboundKey},
    agreement::{self, ECDH_P256, EphemeralPrivateKey, UnparsedPublicKey},
    error::Unspecified,
    hkdf::{self, KeyType, Prk},
    hmac,
    rand::SystemRandom,
};
use serde::Serialize;
use zeroize::Zeroizing;

const HPKE_VERSION_LABEL: &[u8] = b"HPKE-v1";
const KEM_SUITE_ID: &[u8] = b"KEM\x00\x10";
const HPKE_SUITE_ID: &[u8] = b"HPKE\x00\x10\x00\x01\x00\x02";
const HPKE_INFO: &[u8] = b"dubbridge:p2p:k1:hpke-base:v1";
const PROFILE_VERSION: &str = "p2p-k1-hpke-v1";
const P256_SPKI_PREFIX: &[u8] = &[
    0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06,
    0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceEnvelopeBinding<'a> {
    pub profile_version: &'a str,
    pub device_key_id: &'a str,
    pub invitation_id: &'a str,
    pub viewer_id: &'a str,
    pub asset_id: &'a str,
    pub publication_id: &'a str,
    pub lineage_id: &'a str,
    pub authorization_id: &'a str,
    pub expires_at_unix: i64,
}

impl<'a> DeviceEnvelopeBinding<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn k1(
        device_key_id: &'a str,
        invitation_id: &'a str,
        viewer_id: &'a str,
        asset_id: &'a str,
        publication_id: &'a str,
        lineage_id: &'a str,
        authorization_id: &'a str,
        expires_at_unix: i64,
    ) -> Self {
        Self {
            profile_version: PROFILE_VERSION,
            device_key_id,
            invitation_id,
            viewer_id,
            asset_id,
            publication_id,
            lineage_id,
            authorization_id,
            expires_at_unix,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedDeviceEnvelope {
    pub profile_version: &'static str,
    pub encapsulated_key: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub binding_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceEnvelopeError {
    InvalidPublicKey,
    InvalidBinding,
    KeyAgreement,
    KeyDerivation,
    Encryption,
}

#[derive(Clone, Copy)]
struct OutputLen(usize);

impl KeyType for OutputLen {
    fn len(&self) -> usize {
        self.0
    }
}

pub fn validate_p256_spki(public_key_spki: &[u8]) -> Result<(), DeviceEnvelopeError> {
    let public_key = p256_point_from_spki(public_key_spki)?;
    let unparsed = UnparsedPublicKey::new(&ECDH_P256, public_key);
    agreement::ParsedPublicKey::try_from(unparsed)
        .map(|_| ())
        .map_err(|_| DeviceEnvelopeError::InvalidPublicKey)
}

pub fn seal_ck_for_device(
    ck: &[u8; 32],
    public_key_spki: &[u8],
    binding: &DeviceEnvelopeBinding<'_>,
) -> Result<SealedDeviceEnvelope, DeviceEnvelopeError> {
    validate_binding(binding)?;
    let binding_json =
        serde_json::to_string(binding).map_err(|_| DeviceEnvelopeError::InvalidBinding)?;
    let recipient_point = p256_point_from_spki(public_key_spki)?;
    let rng = SystemRandom::new();
    let ephemeral = EphemeralPrivateKey::generate(&ECDH_P256, &rng)
        .map_err(|_| DeviceEnvelopeError::KeyAgreement)?;
    let encapsulated_key = ephemeral
        .compute_public_key()
        .map_err(|_| DeviceEnvelopeError::KeyAgreement)?
        .as_ref()
        .to_vec();
    let peer = UnparsedPublicKey::new(&ECDH_P256, recipient_point);
    let dh = agreement::agree_ephemeral(ephemeral, &peer, Unspecified, |secret| {
        Ok::<_, Unspecified>(Zeroizing::new(secret.to_vec()))
    })
    .map_err(|_| DeviceEnvelopeError::KeyAgreement)?;

    let recipient_point = p256_point_from_spki(public_key_spki)?;
    let mut kem_context = Vec::with_capacity(encapsulated_key.len() + recipient_point.len());
    kem_context.extend_from_slice(&encapsulated_key);
    kem_context.extend_from_slice(recipient_point);
    let shared_secret = extract_and_expand_kem(&dh, &kem_context)?;
    let (key, nonce) = key_schedule(&shared_secret)?;
    let unbound = UnboundKey::new(&AES_256_GCM, key.as_slice())
        .map_err(|_| DeviceEnvelopeError::Encryption)?;
    let sealing_key = LessSafeKey::new(unbound);
    let nonce = Nonce::assume_unique_for_key(nonce);
    let mut ciphertext = ck.to_vec();
    sealing_key
        .seal_in_place_append_tag(nonce, Aad::from(binding_json.as_bytes()), &mut ciphertext)
        .map_err(|_| DeviceEnvelopeError::Encryption)?;

    Ok(SealedDeviceEnvelope {
        profile_version: PROFILE_VERSION,
        encapsulated_key,
        ciphertext,
        binding_json,
    })
}

fn validate_binding(binding: &DeviceEnvelopeBinding<'_>) -> Result<(), DeviceEnvelopeError> {
    let values = [
        binding.device_key_id,
        binding.invitation_id,
        binding.viewer_id,
        binding.asset_id,
        binding.publication_id,
        binding.lineage_id,
        binding.authorization_id,
    ];
    if binding.profile_version != PROFILE_VERSION
        || binding.expires_at_unix <= 0
        || values.iter().any(|value| value.trim().is_empty())
    {
        return Err(DeviceEnvelopeError::InvalidBinding);
    }
    Ok(())
}

fn p256_point_from_spki(public_key_spki: &[u8]) -> Result<&[u8], DeviceEnvelopeError> {
    let expected_len = P256_SPKI_PREFIX.len() + 65;
    if public_key_spki.len() != expected_len
        || !public_key_spki.starts_with(P256_SPKI_PREFIX)
        || public_key_spki[P256_SPKI_PREFIX.len()] != 0x04
    {
        return Err(DeviceEnvelopeError::InvalidPublicKey);
    }
    Ok(&public_key_spki[P256_SPKI_PREFIX.len()..])
}

fn extract_and_expand_kem(
    dh: &[u8],
    kem_context: &[u8],
) -> Result<Zeroizing<[u8; 32]>, DeviceEnvelopeError> {
    let eae_prk = labeled_extract(&[], KEM_SUITE_ID, b"eae_prk", dh);
    labeled_expand_32(&eae_prk, KEM_SUITE_ID, b"shared_secret", kem_context)
}

fn key_schedule(
    shared_secret: &[u8; 32],
) -> Result<(Zeroizing<[u8; 32]>, [u8; 12]), DeviceEnvelopeError> {
    let psk_id_hash = labeled_extract(&[], HPKE_SUITE_ID, b"psk_id_hash", &[]);
    let info_hash = labeled_extract(&[], HPKE_SUITE_ID, b"info_hash", HPKE_INFO);
    let mut context = Vec::with_capacity(65);
    context.push(0);
    context.extend_from_slice(psk_id_hash.as_slice());
    context.extend_from_slice(info_hash.as_slice());
    let secret = labeled_extract(shared_secret, HPKE_SUITE_ID, b"secret", &[]);
    let key = labeled_expand_32(&secret, HPKE_SUITE_ID, b"key", &context)?;
    let nonce_bytes = labeled_expand(
        secret.as_slice(),
        HPKE_SUITE_ID,
        b"base_nonce",
        &context,
        12,
    )?;
    let nonce: [u8; 12] = nonce_bytes
        .as_slice()
        .try_into()
        .map_err(|_| DeviceEnvelopeError::KeyDerivation)?;
    Ok((key, nonce))
}

fn labeled_extract(
    salt: &[u8],
    suite_id: &[u8],
    label: &[u8],
    ikm: &[u8],
) -> Zeroizing<[u8; 32]> {
    let mut labeled_ikm =
        Vec::with_capacity(HPKE_VERSION_LABEL.len() + suite_id.len() + label.len() + ikm.len());
    labeled_ikm.extend_from_slice(HPKE_VERSION_LABEL);
    labeled_ikm.extend_from_slice(suite_id);
    labeled_ikm.extend_from_slice(label);
    labeled_ikm.extend_from_slice(ikm);
    let zero_salt = [0u8; 32];
    let key = hmac::Key::new(
        hmac::HMAC_SHA256,
        if salt.is_empty() { &zero_salt } else { salt },
    );
    let tag = hmac::sign(&key, &labeled_ikm);
    let mut output = [0u8; 32];
    output.copy_from_slice(tag.as_ref());
    Zeroizing::new(output)
}

fn labeled_expand_32(
    prk: &[u8; 32],
    suite_id: &[u8],
    label: &[u8],
    info: &[u8],
) -> Result<Zeroizing<[u8; 32]>, DeviceEnvelopeError> {
    let bytes = labeled_expand(prk, suite_id, label, info, 32)?;
    let output: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| DeviceEnvelopeError::KeyDerivation)?;
    Ok(Zeroizing::new(output))
}

fn labeled_expand(
    prk: &[u8],
    suite_id: &[u8],
    label: &[u8],
    info: &[u8],
    length: usize,
) -> Result<Zeroizing<Vec<u8>>, DeviceEnvelopeError> {
    let length_u16 = u16::try_from(length).map_err(|_| DeviceEnvelopeError::KeyDerivation)?;
    let length_bytes = length_u16.to_be_bytes();
    let mut labeled_info = Vec::with_capacity(
        2 + HPKE_VERSION_LABEL.len() + suite_id.len() + label.len() + info.len(),
    );
    labeled_info.extend_from_slice(&length_bytes);
    labeled_info.extend_from_slice(HPKE_VERSION_LABEL);
    labeled_info.extend_from_slice(suite_id);
    labeled_info.extend_from_slice(label);
    labeled_info.extend_from_slice(info);
    let prk = Prk::new_less_safe(hkdf::HKDF_SHA256, prk);
    let info_parts = [labeled_info.as_slice()];
    let okm = prk
        .expand(&info_parts, OutputLen(length))
        .map_err(|_| DeviceEnvelopeError::KeyDerivation)?;
    let mut output = Zeroizing::new(vec![0u8; length]);
    okm.fill(output.as_mut_slice())
        .map_err(|_| DeviceEnvelopeError::KeyDerivation)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_spki(point: &[u8]) -> Vec<u8> {
        let mut spki = P256_SPKI_PREFIX.to_vec();
        spki.extend_from_slice(point);
        spki
    }

    #[test]
    fn rejects_non_p256_spki() {
        assert_eq!(
            validate_p256_spki(&[1, 2, 3]),
            Err(DeviceEnvelopeError::InvalidPublicKey)
        );
    }

    #[test]
    fn seals_ck_with_bound_profile_and_non_deterministic_encapsulation() {
        let recipient = agreement::PrivateKey::generate(&ECDH_P256).expect("recipient key");
        let public = recipient.compute_public_key().expect("recipient public key");
        let spki = valid_spki(public.as_ref());
        let binding = DeviceEnvelopeBinding::k1(
            "device-key-1",
            "invite-1",
            "viewer-1",
            "asset-1",
            "publication-1",
            "lineage-1",
            "authorization-1",
            1_800_000_000,
        );
        let ck = [7u8; 32];

        let left = seal_ck_for_device(&ck, &spki, &binding).expect("seal left");
        let right = seal_ck_for_device(&ck, &spki, &binding).expect("seal right");

        assert_eq!(left.profile_version, PROFILE_VERSION);
        assert!(left.binding_json.contains("device-key-1"));
        assert_eq!(left.encapsulated_key.len(), 65);
        assert_eq!(left.ciphertext.len(), ck.len() + 16);
        assert_ne!(left.encapsulated_key, right.encapsulated_key);
        assert_ne!(left.ciphertext, right.ciphertext);
    }

    #[test]
    fn rejects_empty_binding_fields() {
        let binding = DeviceEnvelopeBinding::k1(
            "",
            "invite-1",
            "viewer-1",
            "asset-1",
            "publication-1",
            "lineage-1",
            "authorization-1",
            1_800_000_000,
        );
        assert_eq!(
            validate_binding(&binding),
            Err(DeviceEnvelopeError::InvalidBinding)
        );
    }
}
