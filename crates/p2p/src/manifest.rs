use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFile {
    pub ciphertext_sha256: String,
    pub ciphertext_size: u64,
    pub nonce_b64u: String,
    pub path: String,
    pub plaintext_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub asset_id: String,
    pub cipher: String,
    pub digest: String,
    pub files: Vec<ManifestFile>,
    pub lineage_id: String,
    pub manifest_version: String,
    pub publication_id: String,
}

pub fn canonical_json(manifest: &Manifest) -> String {
    serde_json::to_string(manifest).expect("Manifest serialization is infallible for this type")
}

pub fn manifest_sha256(canonical: &str) -> String {
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_json_and_digest() {
        let manifest = Manifest {
            manifest_version: "p2p-manifest-v1".to_string(),
            asset_id: "11111111-1111-4111-8111-111111111111".to_string(),
            publication_id: "22222222-2222-4222-8222-222222222222".to_string(),
            lineage_id: "33333333-3333-4333-8333-333333333333".to_string(),
            cipher: "AES-256-GCM".to_string(),
            digest: "SHA-256".to_string(),
            files: vec![
                ManifestFile {
                    path: "index.m3u8".to_string(),
                    plaintext_size: 120,
                    ciphertext_size: 136,
                    nonce_b64u: "AAECAwQFBgcICQoL".to_string(),
                    ciphertext_sha256:
                        "0000000000000000000000000000000000000000000000000000000000000000"
                            .to_string(),
                },
                ManifestFile {
                    path: "segments/000001.ts".to_string(),
                    plaintext_size: 1024,
                    ciphertext_size: 1040,
                    nonce_b64u: "DA0ODxAREhMUFRYX".to_string(),
                    ciphertext_sha256:
                        "1111111111111111111111111111111111111111111111111111111111111111"
                            .to_string(),
                },
            ],
        };

        let canonical = canonical_json(&manifest);
        let expected_json = r#"{"asset_id":"11111111-1111-4111-8111-111111111111","cipher":"AES-256-GCM","digest":"SHA-256","files":[{"ciphertext_sha256":"0000000000000000000000000000000000000000000000000000000000000000","ciphertext_size":136,"nonce_b64u":"AAECAwQFBgcICQoL","path":"index.m3u8","plaintext_size":120},{"ciphertext_sha256":"1111111111111111111111111111111111111111111111111111111111111111","ciphertext_size":1040,"nonce_b64u":"DA0ODxAREhMUFRYX","path":"segments/000001.ts","plaintext_size":1024}],"lineage_id":"33333333-3333-4333-8333-333333333333","manifest_version":"p2p-manifest-v1","publication_id":"22222222-2222-4222-8222-222222222222"}"#;
        assert_eq!(canonical, expected_json);

        let digest = manifest_sha256(&canonical);
        let expected_digest = "b753ba52473d8b9f1ddc8444d43d6166c6b46eeb1214018e3a33503f56a021b4";
        assert_eq!(digest, expected_digest);
    }
}
