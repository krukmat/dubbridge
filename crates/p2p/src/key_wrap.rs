use aes_gcm::aead::{Aead, Generate, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use zeroize::Zeroizing;

pub struct WrappedKey {
    pub kek_id: String,
    pub kek_version: u32,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

#[derive(Debug)]
pub enum KeyWrapError {
    InvalidKeyLength,
    WrapFailed,
    /// Returned when AEAD authentication fails: either the ciphertext/nonce
    /// was tampered with, or the AAD reconstructed from `wrapped.kek_id`/
    /// `wrapped.kek_version` no longer matches the AAD used to seal it
    /// (i.e. `kek_id`/`kek_version` were swapped after wrapping).
    UnwrapFailed,
}

#[derive(serde::Serialize)]
struct KeyWrapAad<'a> {
    aad_version: &'a str,
    kek_id: &'a str,
    kek_version: u32,
}

fn canonical_key_wrap_aad(kek_id: &str, kek_version: u32) -> Vec<u8> {
    serde_json::to_vec(&KeyWrapAad {
        aad_version: "p2p-kek-wrap-v1",
        kek_id,
        kek_version,
    })
    .expect("KeyWrapAad serialization is infallible for this type")
}

pub fn generate_ck() -> Zeroizing<[u8; 32]> {
    let mut ck = [0u8; 32];
    getrandom::fill(&mut ck).expect("OS CSPRNG is expected to be available");
    Zeroizing::new(ck)
}

pub fn wrap_ck(
    ck: &[u8; 32],
    kek: &[u8; 32],
    kek_id: &str,
    kek_version: u32,
) -> Result<WrappedKey, KeyWrapError> {
    let aad_bytes = canonical_key_wrap_aad(kek_id, kek_version);

    let nonce = Nonce::generate();

    let cipher = Aes256Gcm::new_from_slice(kek).map_err(|_| KeyWrapError::InvalidKeyLength)?;

    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: ck.as_slice(),
                aad: &aad_bytes,
            },
        )
        .map_err(|_| KeyWrapError::WrapFailed)?;

    Ok(WrappedKey {
        kek_id: kek_id.to_string(),
        kek_version,
        nonce: nonce.into(),
        ciphertext,
    })
}

pub fn unwrap_ck(
    wrapped: &WrappedKey,
    kek: &[u8; 32],
) -> Result<Zeroizing<[u8; 32]>, KeyWrapError> {
    let aad_bytes = canonical_key_wrap_aad(&wrapped.kek_id, wrapped.kek_version);

    let cipher = Aes256Gcm::new_from_slice(kek).map_err(|_| KeyWrapError::InvalidKeyLength)?;

    #[allow(deprecated)]
    let nonce = Nonce::from_slice(&wrapped.nonce);

    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: &wrapped.ciphertext,
                aad: &aad_bytes,
            },
        )
        .map_err(|_| KeyWrapError::UnwrapFailed)?;

    let ck: [u8; 32] = plaintext
        .as_slice()
        .try_into()
        .map_err(|_| KeyWrapError::UnwrapFailed)?;

    Ok(Zeroizing::new(ck))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hp_t2d_1_generate_ck_returns_distinct_csprng_material() {
        let ck1 = generate_ck();
        let ck2 = generate_ck();

        assert_eq!(ck1.len(), 32);
        assert_ne!(*ck1, *ck2);
    }

    #[test]
    fn hp_t2d_2_wrap_unwrap_round_trip_recovers_original_ck() {
        let ck = [7u8; 32];
        let kek = [1u8; 32];

        let wrapped = wrap_ck(&ck, &kek, "kek-a", 1).expect("wrap should succeed");
        let recovered = unwrap_ck(&wrapped, &kek).expect("unwrap should succeed");

        assert_eq!(*recovered, ck);
    }

    #[test]
    fn hp_t2d_3_two_wraps_produce_different_nonces_and_ciphertexts() {
        let ck = [7u8; 32];
        let kek = [1u8; 32];

        let wrapped1 = wrap_ck(&ck, &kek, "kek-a", 1).unwrap();
        let wrapped2 = wrap_ck(&ck, &kek, "kek-a", 1).unwrap();

        assert_ne!(wrapped1.nonce, wrapped2.nonce);
        assert_ne!(wrapped1.ciphertext, wrapped2.ciphertext);
    }

    // ec_t2d_1_wrong_key_length_is_rejected:
    // This test is unreachable because the function signature enforces a 32-byte key via &[u8; 32].
    // The InvalidKeyLength error is only possible if the underlying AES implementation rejects the key,
    // which is not possible for a valid 32-byte array.

    #[test]
    fn ec_t2d_2_unwrap_with_wrong_kek_fails_authentication() {
        let ck = [7u8; 32];
        let kek = [1u8; 32];
        let wrong_kek = [2u8; 32];

        let wrapped = wrap_ck(&ck, &kek, "kek-a", 1).unwrap();
        let result = unwrap_ck(&wrapped, &wrong_kek);

        assert!(matches!(result, Err(KeyWrapError::UnwrapFailed)));
    }

    #[test]
    fn ec_t2d_3_unwrap_with_tampered_ciphertext_fails_authentication() {
        let ck = [7u8; 32];
        let kek = [1u8; 32];

        let mut wrapped = wrap_ck(&ck, &kek, "kek-a", 1).unwrap();
        wrapped.ciphertext[0] ^= 0xFF;

        let result = unwrap_ck(&wrapped, &kek);

        assert!(matches!(result, Err(KeyWrapError::UnwrapFailed)));
    }

    #[test]
    fn ec_t2d_4_tampered_kek_id_or_version_fails_authentication() {
        let ck = [7u8; 32];
        let kek = [1u8; 32];

        let wrapped_orig = wrap_ck(&ck, &kek, "kek-a", 1).unwrap();

        let mut wrapped_tampered_id = WrappedKey {
            kek_id: "kek-b".to_string(),
            kek_version: wrapped_orig.kek_version,
            nonce: wrapped_orig.nonce,
            ciphertext: wrapped_orig.ciphertext.clone(),
        };
        assert!(matches!(
            unwrap_ck(&wrapped_tampered_id, &kek),
            Err(KeyWrapError::UnwrapFailed)
        ));

        wrapped_tampered_id.kek_id = "kek-a".to_string();
        wrapped_tampered_id.kek_version = 2;
        assert!(matches!(
            unwrap_ck(&wrapped_tampered_id, &kek),
            Err(KeyWrapError::UnwrapFailed)
        ));
    }

    #[test]
    fn ec_t2d_5_delimiter_colliding_metadata_pairs_do_not_collide() {
        let ck = [7u8; 32];
        let kek = [1u8; 32];

        // Under a naive "{kek_id}|{kek_version}" concatenation, these two
        // pairs would serialize identically: "A|1|2" vs "A|1|2".
        let wrapped_pair1 = wrap_ck(&ck, &kek, "A|1", 2).unwrap();
        let wrapped_pair2 = wrap_ck(&ck, &kek, "A", 12).unwrap();

        // Each unwraps correctly under its own metadata.
        assert!(unwrap_ck(&wrapped_pair1, &kek).is_ok());
        assert!(unwrap_ck(&wrapped_pair2, &kek).is_ok());

        // Swapping pair1's kek_id/kek_version to pair2's values (keeping
        // pair1's ciphertext/nonce) must fail — the AADs must not collide.
        let cross_wrapped = WrappedKey {
            kek_id: "A".to_string(),
            kek_version: 12,
            nonce: wrapped_pair1.nonce,
            ciphertext: wrapped_pair1.ciphertext.clone(),
        };
        assert!(matches!(
            unwrap_ck(&cross_wrapped, &kek),
            Err(KeyWrapError::UnwrapFailed)
        ));

        // And the reverse direction too.
        let cross_wrapped_reverse = WrappedKey {
            kek_id: "A|1".to_string(),
            kek_version: 2,
            nonce: wrapped_pair2.nonce,
            ciphertext: wrapped_pair2.ciphertext.clone(),
        };
        assert!(matches!(
            unwrap_ck(&cross_wrapped_reverse, &kek),
            Err(KeyWrapError::UnwrapFailed)
        ));
    }
}
