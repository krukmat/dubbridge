use crate::aad::Aad;
use aes_gcm::aead::{Aead, Generate, KeyInit, Nonce as AeadNonce, Payload};
use aes_gcm::{Aes256Gcm, Nonce};

pub struct EncryptedFile {
    pub path: String,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

#[derive(Debug)]
pub enum CryptoError {
    EncryptionFailed,
    InvalidKeyLength,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::EncryptionFailed => write!(f, "Encryption failed"),
            CryptoError::InvalidKeyLength => write!(f, "Invalid key length"),
        }
    }
}

impl std::error::Error for CryptoError {}

pub fn encrypt_file(
    ck: &[u8; 32],
    aad: &Aad,
    plaintext: &[u8],
) -> Result<EncryptedFile, CryptoError> {
    encrypt_file_with_nonce(ck, aad, plaintext, generate_nonce())
}

pub(crate) fn generate_nonce() -> [u8; 12] {
    AeadNonce::<Aes256Gcm>::generate().into()
}

pub(crate) fn encrypt_file_with_nonce(
    ck: &[u8; 32],
    aad: &Aad,
    plaintext: &[u8],
    nonce: [u8; 12],
) -> Result<EncryptedFile, CryptoError> {
    let aad_bytes = crate::aad::canonical_aad_json(aad).into_bytes();
    let cipher = Aes256Gcm::new_from_slice(ck).map_err(|_| CryptoError::InvalidKeyLength)?;

    #[allow(deprecated)]
    let nonce_ref = Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(
            nonce_ref,
            Payload {
                msg: plaintext,
                aad: &aad_bytes,
            },
        )
        .map_err(|_| CryptoError::EncryptionFailed)?;

    Ok(EncryptedFile {
        path: aad.path.clone(),
        nonce,
        ciphertext,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_aad(path: &str) -> Aad {
        Aad {
            aad_version: "p2p-aad-v1".to_string(),
            asset_id: "11111111-1111-4111-8111-111111111111".to_string(),
            lineage_id: "33333333-3333-4333-8333-333333333333".to_string(),
            manifest_version: "p2p-manifest-v1".to_string(),
            path: path.to_string(),
            publication_id: "22222222-2222-4222-8222-222222222222".to_string(),
        }
    }

    #[test]
    fn hp_t2c_1_round_trip_recovers_plaintext() {
        let ck = [0u8; 32];
        let aad = make_aad("test/path");
        let plaintext = b"hello world";

        let encrypted = encrypt_file(&ck, &aad, plaintext).expect("Encryption should succeed");

        let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
        let aad_bytes = crate::aad::canonical_aad_json(&aad).into_bytes();
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let decrypted = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &encrypted.ciphertext,
                    aad: &aad_bytes,
                },
            )
            .unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn hp_t2c_2_two_calls_produce_different_nonces_and_ciphertexts() {
        let ck = [0u8; 32];
        let aad = make_aad("test/path");
        let plaintext = b"hello world";

        let enc1 = encrypt_file(&ck, &aad, plaintext).unwrap();
        let enc2 = encrypt_file(&ck, &aad, plaintext).unwrap();

        assert_ne!(enc1.nonce, enc2.nonce);
        assert_ne!(enc1.ciphertext, enc2.ciphertext);
    }

    #[test]
    fn hp_t2c_r1a_fixed_nonce_round_trip_recovers_plaintext() {
        let ck = [0x11; 32];
        let aad = make_aad("test/fixed-nonce");
        let plaintext = b"assigned nonce plaintext";
        let assigned_nonce = [0x22; 12];

        let encrypted = encrypt_file_with_nonce(&ck, &aad, plaintext, assigned_nonce)
            .expect("assigned-nonce encryption should succeed");

        assert_eq!(encrypted.nonce, assigned_nonce);
        let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
        let aad_bytes = crate::aad::canonical_aad_json(&aad).into_bytes();
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(&assigned_nonce);
        let decrypted = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &encrypted.ciphertext,
                    aad: &aad_bytes,
                },
            )
            .expect("ciphertext should authenticate with the assigned nonce and AAD");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn ec_t2c_r1a_tampered_aad_fails_for_assigned_nonce() {
        let ck = [0x11; 32];
        let aad = make_aad("test/fixed-nonce");
        let assigned_nonce = [0x22; 12];
        let encrypted = encrypt_file_with_nonce(&ck, &aad, b"authenticated", assigned_nonce)
            .expect("assigned-nonce encryption should succeed");

        let tampered_aad = make_aad("test/tampered");
        let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
        let aad_bytes = crate::aad::canonical_aad_json(&tampered_aad).into_bytes();
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(&assigned_nonce);

        assert!(
            cipher
                .decrypt(
                    nonce,
                    Payload {
                        msg: &encrypted.ciphertext,
                        aad: &aad_bytes,
                    },
                )
                .is_err()
        );
    }

    // ec_t2c_1_wrong_key_length_is_rejected:
    // This test is unreachable because the function signature enforces a 32-byte key via &[u8; 32].
    // The InvalidKeyLength error is only possible if the underlying AES implementation rejects the key,
    // which is not possible for a valid 32-byte array.

    #[test]
    fn ec_t2c_2_tampered_aad_fails_authentication() {
        let ck = [0u8; 32];
        let aad = make_aad("test/path");
        let plaintext = b"hello world";

        let encrypted = encrypt_file(&ck, &aad, plaintext).unwrap();

        let tampered_aad = make_aad("tampered/path");
        let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
        let aad_bytes = crate::aad::canonical_aad_json(&tampered_aad).into_bytes();
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let result = cipher.decrypt(
            nonce,
            Payload {
                msg: &encrypted.ciphertext,
                aad: &aad_bytes,
            },
        );
        assert!(result.is_err());
    }
}
