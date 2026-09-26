#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Aad {
    pub aad_version: String,
    pub asset_id: String,
    pub lineage_id: String,
    pub manifest_version: String,
    pub path: String,
    pub publication_id: String,
}

pub fn canonical_aad_json(aad: &Aad) -> String {
    serde_json::to_string(aad).expect("Aad serialization is infallible for this type")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_aad_json() {
        let aad = Aad {
            aad_version: "p2p-aad-v1".to_string(),
            asset_id: "11111111-1111-4111-8111-111111111111".to_string(),
            lineage_id: "33333333-3333-4333-8333-333333333333".to_string(),
            manifest_version: "p2p-manifest-v1".to_string(),
            path: "index.m3u8".to_string(),
            publication_id: "22222222-2222-4222-8222-222222222222".to_string(),
        };

        let canonical = canonical_aad_json(&aad);
        let expected = r#"{"aad_version":"p2p-aad-v1","asset_id":"11111111-1111-4111-8111-111111111111","lineage_id":"33333333-3333-4333-8333-333333333333","manifest_version":"p2p-manifest-v1","path":"index.m3u8","publication_id":"22222222-2222-4222-8222-222222222222"}"#;

        assert_eq!(canonical, expected);
    }

    #[test]
    fn test_canonical_aad_json_sha256() {
        let aad = Aad {
            aad_version: "p2p-aad-v1".to_string(),
            asset_id: "11111111-1111-4111-8111-111111111111".to_string(),
            lineage_id: "33333333-3333-4333-8333-333333333333".to_string(),
            manifest_version: "p2p-manifest-v1".to_string(),
            path: "index.m3u8".to_string(),
            publication_id: "22222222-2222-4222-8222-222222222222".to_string(),
        };

        let canonical = canonical_aad_json(&aad);
        let digest = crate::manifest::manifest_sha256(&canonical);
        let expected_digest = "d6d266884e3e7ba734ac7e325d813830eb85274b9f9e65a09a00c32264a3111a";

        assert_eq!(digest, expected_digest);
    }
}
