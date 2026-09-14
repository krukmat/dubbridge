use dubbridge_connectors::p2p_availability::AvailabilityPublicationRequest;
use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId},
    p2p_ready_descriptor::{P2pReadyDescriptor, P2pReadyDescriptorInput},
};
use dubbridge_p2p::package_builder::{PackageFileInput, build_package};
use serde_json::Value;
use uuid::Uuid;

const PLAINTEXT_MARKER: &[u8] = b"T6-PLAINTEXT-MUST-NOT-CROSS-PUBLICATION-BOUNDARY";
const RAW_KEK: [u8; 32] = [0xa5; 32];

fn forbidden_keys() -> [&'static str; 11] {
    [
        "plaintext_ck",
        "ck_bytes",
        "wrapped_ck",
        "wrapped_ck_bytes",
        "kek_bytes",
        "viewer",
        "invite",
        "device",
        "database_url",
        "jwt_signing_material",
        "service_private_key",
    ]
}

fn assert_no_forbidden_keys(value: &Value) {
    match value {
        Value::Object(map) => {
            for key in map.keys() {
                assert!(
                    !forbidden_keys().contains(&key.as_str()),
                    "forbidden publication-boundary key leaked: {key}"
                );
            }
            for child in map.values() {
                assert_no_forbidden_keys(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                assert_no_forbidden_keys(child);
            }
        }
        _ => {}
    }
}

#[test]
fn sealed_package_contains_ciphertext_not_source_plaintext() {
    let asset_id = Uuid::new_v4().to_string();
    let publication_id = Uuid::new_v4().to_string();
    let lineage_id = Uuid::new_v4().to_string();
    let ck = [0x42_u8; 32];
    let inputs = vec![
        PackageFileInput {
            path: "index.m3u8".to_string(),
            plaintext: PLAINTEXT_MARKER.to_vec(),
        },
        PackageFileInput {
            path: "segment-00001.ts".to_string(),
            plaintext: PLAINTEXT_MARKER.to_vec(),
        },
    ];

    let sealed = build_package(&asset_id, &publication_id, &lineage_id, &ck, &inputs)
        .expect("seal package");

    assert_eq!(sealed.files.len(), inputs.len());
    for (sealed_file, input) in sealed.files.iter().zip(&inputs) {
        assert_ne!(sealed_file.ciphertext, input.plaintext);
        assert!(sealed_file.ciphertext.len() > input.plaintext.len());
    }
    assert!(!sealed.manifest_canonical_json.contains("T6-PLAINTEXT"));
}

#[test]
fn availability_request_is_metadata_only_and_secret_deny_clean() {
    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();
    let request = AvailabilityPublicationRequest::new(
        publication_id,
        lineage_id,
        "b753ba52473d8b9f1ddc8444d43d6166c6b46eeb1214018e3a33503f56a021b4",
        format!("packages/{publication_id}/{lineage_id}"),
    )
    .expect("valid publication request");

    let value = serde_json::to_value(&request).expect("serialize request");
    assert_no_forbidden_keys(&value);
    let object = value.as_object().expect("request object");
    assert_eq!(object.len(), 6);
    assert_eq!(
        object.keys().cloned().collect::<std::collections::BTreeSet<_>>(),
        [
            "contract_version",
            "lineage_id",
            "manifest_digest_sha256",
            "manifest_version",
            "package_ref",
            "publication_id",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    );
}

#[test]
fn ready_descriptor_exposes_only_opaque_wrap_reference() {
    let asset_id = AssetId::new();
    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();
    let descriptor = P2pReadyDescriptor::try_from(P2pReadyDescriptorInput {
        asset_id,
        publication_id,
        lineage_id,
        manifest_digest_sha256:
            "b753ba52473d8b9f1ddc8444d43d6166c6b46eeb1214018e3a33503f56a021b4".to_string(),
        external_publication_id: "hyperdrive:stable-id".to_string(),
        kek_id: "server-kek-v1".to_string(),
        kek_version: 1,
        ready_at: "2026-09-14T10:00:00Z".to_string(),
    })
    .expect("valid ready descriptor");

    let value = serde_json::to_value(&descriptor).expect("serialize descriptor");
    assert_no_forbidden_keys(&value);
    let serialized = serde_json::to_string(&value).expect("descriptor json");
    assert!(!serialized.contains(&hex::encode(RAW_KEK)));
    assert!(!serialized.contains("T6-PLAINTEXT"));
    assert!(descriptor.ck_wrap_ref.starts_with("p2p-k1-wrap/"));
}
