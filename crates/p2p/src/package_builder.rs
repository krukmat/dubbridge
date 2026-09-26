use crate::aad::Aad;
use crate::crypto;
use crate::manifest::{self, Manifest, ManifestFile};
use crate::nonce_tracker::NonceTracker;
use crate::path;

pub struct PackageFileInput {
    pub path: String,
    pub plaintext: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PackageBuildError {
    EmptyPackage,
    DuplicatePath,
    InvalidPath(path::PathError),
    OutOfOrder,
    NonceCollision,
    Encryption(String, String),
}

pub struct SealedPackage {
    pub manifest: Manifest,
    pub manifest_canonical_json: String,
    pub manifest_digest_sha256: String,
    pub files: Vec<SealedFile>,
}

pub struct SealedFile {
    pub path: String,
    pub ciphertext: Vec<u8>,
}

const MANIFEST_SLOT_PATH: &str = "index.m3u8";

pub fn build_package(
    asset_id: &str,
    publication_id: &str,
    lineage_id: &str,
    ck: &[u8; 32],
    inputs: &[PackageFileInput],
) -> Result<SealedPackage, PackageBuildError> {
    build_package_with_nonce_source(
        asset_id,
        publication_id,
        lineage_id,
        ck,
        inputs,
        crypto::generate_nonce,
    )
}

fn build_package_with_nonce_source<F>(
    asset_id: &str,
    publication_id: &str,
    lineage_id: &str,
    ck: &[u8; 32],
    inputs: &[PackageFileInput],
    mut next_nonce: F,
) -> Result<SealedPackage, PackageBuildError>
where
    F: FnMut() -> [u8; 12],
{
    let normalized_paths = validate_and_normalize_paths(inputs)?;

    let mut manifest_files: Vec<ManifestFile> = Vec::with_capacity(inputs.len());
    let mut sealed_files: Vec<SealedFile> = Vec::with_capacity(inputs.len());
    let mut nonce_tracker = NonceTracker::default();

    for (i, input) in inputs.iter().enumerate() {
        let norm_path = &normalized_paths[i];
        let nonce = next_nonce();
        nonce_tracker
            .register(nonce)
            .map_err(|_| PackageBuildError::NonceCollision)?;
        let (mf, sf) = encrypt_one(
            asset_id,
            publication_id,
            lineage_id,
            ck,
            norm_path,
            input,
            nonce,
        )?;
        manifest_files.push(mf);
        sealed_files.push(sf);
    }

    let manifest = Manifest {
        asset_id: asset_id.to_string(),
        cipher: "AES-256-GCM".to_string(),
        digest: "SHA-256".to_string(),
        files: manifest_files,
        lineage_id: lineage_id.to_string(),
        manifest_version: "p2p-manifest-v1".to_string(),
        publication_id: publication_id.to_string(),
    };

    let manifest_canonical_json = manifest::canonical_json(&manifest);
    let manifest_digest_sha256 = manifest::manifest_sha256(&manifest_canonical_json);

    Ok(SealedPackage {
        manifest,
        manifest_canonical_json,
        manifest_digest_sha256,
        files: sealed_files,
    })
}

fn validate_and_normalize_paths(
    inputs: &[PackageFileInput],
) -> Result<Vec<String>, PackageBuildError> {
    // 1. Empty check
    if inputs.is_empty() {
        return Err(PackageBuildError::EmptyPackage);
    }

    // 2. Normalize paths
    let mut normalized_paths: Vec<String> = Vec::with_capacity(inputs.len());
    for input in inputs {
        match path::normalize_path(&input.path) {
            Ok(norm) => normalized_paths.push(norm),
            Err(e) => return Err(PackageBuildError::InvalidPath(e)),
        }
    }

    // 3. Duplicate check
    for i in 0..normalized_paths.len() {
        for j in (i + 1)..normalized_paths.len() {
            if normalized_paths[i] == normalized_paths[j] {
                return Err(PackageBuildError::DuplicatePath);
            }
        }
    }

    // 4. First path check
    if normalized_paths[0] != MANIFEST_SLOT_PATH {
        return Err(PackageBuildError::OutOfOrder);
    }

    // 5. Order check for segments (indices 1..end)
    for i in 1..normalized_paths.len() - 1 {
        if normalized_paths[i] >= normalized_paths[i + 1] {
            return Err(PackageBuildError::OutOfOrder);
        }
    }

    Ok(normalized_paths)
}

fn encrypt_one(
    asset_id: &str,
    publication_id: &str,
    lineage_id: &str,
    ck: &[u8; 32],
    norm_path: &str,
    input: &PackageFileInput,
    nonce: [u8; 12],
) -> Result<(ManifestFile, SealedFile), PackageBuildError> {
    let aad = Aad {
        aad_version: "p2p-aad-v1".to_string(),
        asset_id: asset_id.to_string(),
        lineage_id: lineage_id.to_string(),
        manifest_version: "p2p-manifest-v1".to_string(),
        path: norm_path.to_string(),
        publication_id: publication_id.to_string(),
    };

    match crypto::encrypt_file_with_nonce(ck, &aad, &input.plaintext, nonce) {
        Ok(encrypted) => {
            let nonce_b64u = base64url_encode_no_padding(&encrypted.nonce);
            let ciphertext_sha256 = sha256_hex(&encrypted.ciphertext);

            let mf = ManifestFile {
                path: norm_path.to_string(),
                plaintext_size: input.plaintext.len() as u64,
                ciphertext_size: encrypted.ciphertext.len() as u64,
                nonce_b64u,
                ciphertext_sha256,
            };
            let sf = SealedFile {
                path: norm_path.to_string(),
                ciphertext: encrypted.ciphertext,
            };
            Ok((mf, sf))
        }
        Err(e) => Err(PackageBuildError::Encryption(
            format!("{:?}", e),
            norm_path.to_string(),
        )),
    }
}

fn base64url_encode_no_padding(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::new();
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i];
        let b1 = if i + 1 < input.len() { input[i + 1] } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] } else { 0 };

        let mut bytes = [0u8; 4];
        bytes[0] = b0 >> 2;
        bytes[1] = ((b0 & 0x03) << 4) | ((b1 >> 4) & 0x0f);
        bytes[2] = ((b1 & 0x0f) << 2) | ((b2 >> 6) & 0x03);
        bytes[3] = b2 & 0x3f;

        output.push(ALPHABET[bytes[0] as usize] as char);
        if i + 1 < input.len() {
            output.push(ALPHABET[bytes[1] as usize] as char);
        }
        if i + 2 < input.len() {
            output.push(ALPHABET[bytes[2] as usize] as char);
            output.push(ALPHABET[bytes[3] as usize] as char);
        }
        i += 3;
    }
    output
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aad::canonical_aad_json;
    use aes_gcm::aead::{Aead, KeyInit, Payload};
    use aes_gcm::{Aes256Gcm, Nonce};
    use std::convert::TryInto;

    fn base64url_decode_no_padding(s: &str) -> Vec<u8> {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut lookup = [255u8; 256];
        for (i, c) in ALPHABET.iter().enumerate() {
            lookup[*c as usize] = i as u8;
        }

        let mut output = Vec::new();
        let mut buffer = 0u32;
        let mut bits = 0u32;

        for c in s.bytes() {
            if c == b'=' {
                continue;
            }
            let val = lookup[c as usize];
            if val == 255 {
                continue;
            }
            buffer = (buffer << 6) | val as u32;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                output.push((buffer >> bits) as u8);
            }
        }
        output
    }

    #[test]
    fn hp_t2f_1_valid_package_produces_correct_manifest_and_roundtrips() {
        let ck = [0u8; 32];
        let inputs = vec![
            PackageFileInput {
                path: "index.m3u8".to_string(),
                plaintext: b"#EXTM3U".to_vec(),
            },
            PackageFileInput {
                path: "segments/000001.ts".to_string(),
                plaintext: b"segment1".to_vec(),
            },
            PackageFileInput {
                path: "segments/000002.ts".to_string(),
                plaintext: b"segment2".to_vec(),
            },
        ];

        assert!(build_package("asset1", "pub1", "line1", &ck, &inputs).is_ok());

        let assigned_nonces = [[0x41; 12], [0x42; 12], [0x43; 12]];
        let mut nonce_source = assigned_nonces.into_iter();
        let pkg = build_package_with_nonce_source("asset1", "pub1", "line1", &ck, &inputs, || {
            nonce_source.next().expect("one nonce per input")
        })
        .expect("distinct assigned nonces should build a package");

        assert_eq!(pkg.files.len(), 3);
        assert_eq!(pkg.manifest.files.len(), 3);
        for (manifest_file, assigned_nonce) in pkg.manifest.files.iter().zip(assigned_nonces) {
            assert_eq!(
                nonce_hex_from_b64u_for_test(&manifest_file.nonce_b64u),
                assigned_nonce
            );
        }

        // Verify manifest digest
        let expected_digest = manifest::manifest_sha256(&pkg.manifest_canonical_json);
        assert_eq!(pkg.manifest_digest_sha256, expected_digest);

        // Decrypt each file
        for (i, sf) in pkg.files.iter().enumerate() {
            let mf = &pkg.manifest.files[i];
            let nonce_bytes = base64url_decode_no_padding(&mf.nonce_b64u);
            let nonce: [u8; 12] = nonce_bytes.try_into().unwrap();

            let aad = Aad {
                aad_version: "p2p-aad-v1".to_string(),
                asset_id: "asset1".to_string(),
                lineage_id: "line1".to_string(),
                manifest_version: "p2p-manifest-v1".to_string(),
                path: sf.path.clone(),
                publication_id: "pub1".to_string(),
            };
            let aad_bytes = canonical_aad_json(&aad).into_bytes();

            let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
            #[allow(deprecated)]
            let nonce_ref = Nonce::from_slice(&nonce);
            let decrypted = cipher
                .decrypt(
                    nonce_ref,
                    Payload {
                        msg: &sf.ciphertext,
                        aad: &aad_bytes,
                    },
                )
                .unwrap();

            assert_eq!(decrypted, inputs[i].plaintext);
        }
    }

    #[test]
    fn hp_t2f_2_two_calls_are_each_independently_decryptable_but_not_byte_identical() {
        let ck = [0u8; 32];
        let inputs = vec![
            PackageFileInput {
                path: "index.m3u8".to_string(),
                plaintext: b"#EXTM3U".to_vec(),
            },
            PackageFileInput {
                path: "segments/000001.ts".to_string(),
                plaintext: b"segment1".to_vec(),
            },
        ];

        let pkg1 = build_package("asset1", "pub1", "line1", &ck, &inputs).unwrap();
        let pkg2 = build_package("asset1", "pub1", "line1", &ck, &inputs).unwrap();

        assert_eq!(pkg1.files.len(), 2);
        assert_eq!(pkg2.files.len(), 2);

        // Ciphertexts should differ due to fresh nonces
        assert_ne!(pkg1.files[0].ciphertext, pkg2.files[0].ciphertext);
        assert_ne!(pkg1.files[1].ciphertext, pkg2.files[1].ciphertext);

        // Both should decrypt correctly
        for pkg in [pkg1, pkg2] {
            for (i, sf) in pkg.files.iter().enumerate() {
                let mf = &pkg.manifest.files[i];
                let nonce_bytes = base64url_decode_no_padding(&mf.nonce_b64u);
                let nonce: [u8; 12] = nonce_bytes.try_into().unwrap();

                let aad = Aad {
                    aad_version: "p2p-aad-v1".to_string(),
                    asset_id: "asset1".to_string(),
                    lineage_id: "line1".to_string(),
                    manifest_version: "p2p-manifest-v1".to_string(),
                    path: sf.path.clone(),
                    publication_id: "pub1".to_string(),
                };
                let aad_bytes = canonical_aad_json(&aad).into_bytes();

                let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
                #[allow(deprecated)]
                let nonce_ref = Nonce::from_slice(&nonce);
                let decrypted = cipher
                    .decrypt(
                        nonce_ref,
                        Payload {
                            msg: &sf.ciphertext,
                            aad: &aad_bytes,
                        },
                    )
                    .unwrap();

                assert_eq!(decrypted, inputs[i].plaintext);
            }
        }
    }

    #[test]
    fn ec_t2c_r3c_duplicate_nonce_fails_closed() {
        let ck = [0x31; 32];
        let inputs = vec![
            PackageFileInput {
                path: "index.m3u8".to_string(),
                plaintext: b"#EXTM3U".to_vec(),
            },
            PackageFileInput {
                path: "segments/000001.ts".to_string(),
                plaintext: b"segment1".to_vec(),
            },
        ];
        let mut nonce_source = [[0x41; 12], [0x41; 12]].into_iter();

        let result =
            build_package_with_nonce_source("asset1", "pub1", "line1", &ck, &inputs, || {
                nonce_source.next().expect("one nonce per input")
            });

        assert!(matches!(result, Err(PackageBuildError::NonceCollision)));
    }

    #[test]
    fn ec_t2f_1_empty_inputs_is_rejected() {
        let ck = [0u8; 32];
        let inputs: Vec<PackageFileInput> = vec![];
        let result = build_package("asset1", "pub1", "line1", &ck, &inputs);
        assert!(matches!(result, Err(PackageBuildError::EmptyPackage)));
    }

    #[test]
    fn ec_t2f_2_duplicate_normalized_path_is_rejected() {
        let ck = [0u8; 32];
        let inputs = vec![
            PackageFileInput {
                path: "index.m3u8".to_string(),
                plaintext: b"#EXTM3U".to_vec(),
            },
            PackageFileInput {
                path: "segments/000001.ts".to_string(),
                plaintext: b"segment1".to_vec(),
            },
            PackageFileInput {
                path: "segments/000001.ts".to_string(),
                plaintext: b"segment1".to_vec(),
            },
        ];
        let result = build_package("asset1", "pub1", "line1", &ck, &inputs);
        assert!(matches!(result, Err(PackageBuildError::DuplicatePath)));
    }

    #[test]
    fn ec_t2f_3_invalid_path_is_rejected() {
        let ck = [0u8; 32];
        let inputs = vec![PackageFileInput {
            path: "../escape.ts".to_string(),
            plaintext: b"data".to_vec(),
        }];
        let result = build_package("asset1", "pub1", "line1", &ck, &inputs);
        assert!(matches!(result, Err(PackageBuildError::InvalidPath(_))));
    }

    #[test]
    fn ec_t2f_4_wrong_manifest_slot_path_is_rejected() {
        let ck = [0u8; 32];
        let inputs = vec![PackageFileInput {
            path: "segments/000001.ts".to_string(),
            plaintext: b"segment1".to_vec(),
        }];
        let result = build_package("asset1", "pub1", "line1", &ck, &inputs);
        assert!(matches!(result, Err(PackageBuildError::OutOfOrder)));
    }

    #[test]
    fn ec_t2f_4b_unsorted_segments_are_rejected() {
        let ck = [0u8; 32];
        let inputs = vec![
            PackageFileInput {
                path: "index.m3u8".to_string(),
                plaintext: b"#EXTM3U".to_vec(),
            },
            PackageFileInput {
                path: "segments/000002.ts".to_string(),
                plaintext: b"segment2".to_vec(),
            },
            PackageFileInput {
                path: "segments/000001.ts".to_string(),
                plaintext: b"segment1".to_vec(),
            },
        ];
        let result = build_package("asset1", "pub1", "line1", &ck, &inputs);
        assert!(matches!(result, Err(PackageBuildError::OutOfOrder)));
    }

    fn nonce_hex_from_b64u_for_test(value: &str) -> [u8; 12] {
        base64url_decode_no_padding(value).try_into().unwrap()
    }
}
