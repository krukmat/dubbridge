---
type: Audit
title: "MVP0-P2P P2.T2c-r integrated code review packet"
task: P2.T2c-r
phase: code-solution
status: complete
date: 2026-09-08
---

# P2.T2c-r — integrated code-solution review packet

## Acceptance

- A new lineage build must allocate one 96-bit CSPRNG nonce per file and reject a duplicate under the same CK before encrypting the colliding file.
- Duplicate rejection must return PackageBuildError::NonceCollision and no SealedPackage.
- The public encrypt_file and build_package signatures and normal CSPRNG behavior stay unchanged.
- The caller-assigned nonce primitive and deterministic nonce-source seam remain crate-private.
- Fixed assigned nonces round-trip with correct AAD and fail authentication with tampered AAD.
- Distinct deterministic nonces produce a decryptable multi-file package with distinct manifest nonces.
- No persistence, CK/KEK, storage, network, or retry behavior changes.

## Verification already passed

- cargo fmt --check
- cargo test -p dubbridge-p2p --all-targets: 39 unit tests + 3 Rust/Node contract tests passed
- cargo clippy -p dubbridge-p2p --all-targets --all-features -- -D warnings

## Diff

```diff
diff --git a/crates/p2p/src/crypto.rs b/crates/p2p/src/crypto.rs
index 45cb2d5..ac4eb60 100644
--- a/crates/p2p/src/crypto.rs
+++ b/crates/p2p/src/crypto.rs
@@ -1,5 +1,5 @@
 use crate::aad::Aad;
-use aes_gcm::aead::{Aead, Generate, KeyInit, Payload};
+use aes_gcm::aead::{Aead, Generate, KeyInit, Nonce as AeadNonce, Payload};
 use aes_gcm::{Aes256Gcm, Nonce};
 
 pub struct EncryptedFile {
@@ -30,15 +30,27 @@ pub fn encrypt_file(
     aad: &Aad,
     plaintext: &[u8],
 ) -> Result<EncryptedFile, CryptoError> {
-    let aad_bytes = crate::aad::canonical_aad_json(aad).into_bytes();
+    encrypt_file_with_nonce(ck, aad, plaintext, generate_nonce())
+}
 
-    let nonce = Nonce::generate();
+pub(crate) fn generate_nonce() -> [u8; 12] {
+    AeadNonce::<Aes256Gcm>::generate().into()
+}
 
+pub(crate) fn encrypt_file_with_nonce(
+    ck: &[u8; 32],
+    aad: &Aad,
+    plaintext: &[u8],
+    nonce: [u8; 12],
+) -> Result<EncryptedFile, CryptoError> {
+    let aad_bytes = crate::aad::canonical_aad_json(aad).into_bytes();
     let cipher = Aes256Gcm::new_from_slice(ck).map_err(|_| CryptoError::InvalidKeyLength)?;
 
+    #[allow(deprecated)]
+    let nonce_ref = Nonce::from_slice(&nonce);
     let ciphertext = cipher
         .encrypt(
-            &nonce,
+            nonce_ref,
             Payload {
                 msg: plaintext,
                 aad: &aad_bytes,
@@ -48,7 +60,7 @@ pub fn encrypt_file(
 
     Ok(EncryptedFile {
         path: aad.path.clone(),
-        nonce: nonce.into(),
+        nonce,
         ciphertext,
     })
 }
@@ -106,6 +118,61 @@ mod tests {
         assert_ne!(enc1.ciphertext, enc2.ciphertext);
     }
 
+    #[test]
+    fn hp_t2c_r1a_fixed_nonce_round_trip_recovers_plaintext() {
+        let ck = [0x11; 32];
+        let aad = make_aad("test/fixed-nonce");
+        let plaintext = b"assigned nonce plaintext";
+        let assigned_nonce = [0x22; 12];
+
+        let encrypted = encrypt_file_with_nonce(&ck, &aad, plaintext, assigned_nonce)
+            .expect("assigned-nonce encryption should succeed");
+
+        assert_eq!(encrypted.nonce, assigned_nonce);
+        let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
+        let aad_bytes = crate::aad::canonical_aad_json(&aad).into_bytes();
+        #[allow(deprecated)]
+        let nonce = Nonce::from_slice(&assigned_nonce);
+        let decrypted = cipher
+            .decrypt(
+                nonce,
+                Payload {
+                    msg: &encrypted.ciphertext,
+                    aad: &aad_bytes,
+                },
+            )
+            .expect("ciphertext should authenticate with the assigned nonce and AAD");
+
+        assert_eq!(decrypted, plaintext);
+    }
+
+    #[test]
+    fn ec_t2c_r1a_tampered_aad_fails_for_assigned_nonce() {
+        let ck = [0x11; 32];
+        let aad = make_aad("test/fixed-nonce");
+        let assigned_nonce = [0x22; 12];
+        let encrypted = encrypt_file_with_nonce(&ck, &aad, b"authenticated", assigned_nonce)
+            .expect("assigned-nonce encryption should succeed");
+
+        let tampered_aad = make_aad("test/tampered");
+        let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
+        let aad_bytes = crate::aad::canonical_aad_json(&tampered_aad).into_bytes();
+        #[allow(deprecated)]
+        let nonce = Nonce::from_slice(&assigned_nonce);
+
+        assert!(
+            cipher
+                .decrypt(
+                    nonce,
+                    Payload {
+                        msg: &encrypted.ciphertext,
+                        aad: &aad_bytes,
+                    },
+                )
+                .is_err()
+        );
+    }
+
     // ec_t2c_1_wrong_key_length_is_rejected:
     // This test is unreachable because the function signature enforces a 32-byte key via &[u8; 32].
     // The InvalidKeyLength error is only possible if the underlying AES implementation rejects the key,
diff --git a/crates/p2p/src/lib.rs b/crates/p2p/src/lib.rs
index 4f1a2f8..547e55c 100644
--- a/crates/p2p/src/lib.rs
+++ b/crates/p2p/src/lib.rs
@@ -2,6 +2,7 @@ pub mod aad;
 pub mod crypto;
 pub mod key_wrap;
 pub mod manifest;
+mod nonce_tracker;
 pub mod package_builder;
 pub mod path;
 pub mod source;
diff --git a/crates/p2p/src/package_builder.rs b/crates/p2p/src/package_builder.rs
index 8827114..bf40582 100644
--- a/crates/p2p/src/package_builder.rs
+++ b/crates/p2p/src/package_builder.rs
@@ -1,6 +1,7 @@
 use crate::aad::Aad;
 use crate::crypto;
 use crate::manifest::{self, Manifest, ManifestFile};
+use crate::nonce_tracker::NonceTracker;
 use crate::path;
 
 pub struct PackageFileInput {
@@ -14,6 +15,7 @@ pub enum PackageBuildError {
     DuplicatePath,
     InvalidPath(path::PathError),
     OutOfOrder,
+    NonceCollision,
     Encryption(String, String),
 }
 
@@ -38,14 +40,48 @@ pub fn build_package(
     ck: &[u8; 32],
     inputs: &[PackageFileInput],
 ) -> Result<SealedPackage, PackageBuildError> {
+    build_package_with_nonce_source(
+        asset_id,
+        publication_id,
+        lineage_id,
+        ck,
+        inputs,
+        crypto::generate_nonce,
+    )
+}
+
+fn build_package_with_nonce_source<F>(
+    asset_id: &str,
+    publication_id: &str,
+    lineage_id: &str,
+    ck: &[u8; 32],
+    inputs: &[PackageFileInput],
+    mut next_nonce: F,
+) -> Result<SealedPackage, PackageBuildError>
+where
+    F: FnMut() -> [u8; 12],
+{
     let normalized_paths = validate_and_normalize_paths(inputs)?;
 
     let mut manifest_files: Vec<ManifestFile> = Vec::with_capacity(inputs.len());
     let mut sealed_files: Vec<SealedFile> = Vec::with_capacity(inputs.len());
+    let mut nonce_tracker = NonceTracker::default();
 
     for (i, input) in inputs.iter().enumerate() {
         let norm_path = &normalized_paths[i];
-        let (mf, sf) = encrypt_one(asset_id, publication_id, lineage_id, ck, norm_path, input)?;
+        let nonce = next_nonce();
+        nonce_tracker
+            .register(nonce)
+            .map_err(|_| PackageBuildError::NonceCollision)?;
+        let (mf, sf) = encrypt_one(
+            asset_id,
+            publication_id,
+            lineage_id,
+            ck,
+            norm_path,
+            input,
+            nonce,
+        )?;
         manifest_files.push(mf);
         sealed_files.push(sf);
     }
@@ -119,6 +155,7 @@ fn encrypt_one(
     ck: &[u8; 32],
     norm_path: &str,
     input: &PackageFileInput,
+    nonce: [u8; 12],
 ) -> Result<(ManifestFile, SealedFile), PackageBuildError> {
     let aad = Aad {
         aad_version: "p2p-aad-v1".to_string(),
@@ -129,7 +166,7 @@ fn encrypt_one(
         publication_id: publication_id.to_string(),
     };
 
-    match crypto::encrypt_file(ck, &aad, &input.plaintext) {
+    match crypto::encrypt_file_with_nonce(ck, &aad, &input.plaintext, nonce) {
         Ok(encrypted) => {
             let nonce_b64u = base64url_encode_no_padding(&encrypted.nonce);
             let ciphertext_sha256 = sha256_hex(&encrypted.ciphertext);
@@ -349,6 +386,84 @@ mod tests {
         }
     }
 
+    #[test]
+    fn hp_t2c_r3c_distinct_assigned_nonces_build_decryptable_package() {
+        let ck = [0x31; 32];
+        let inputs = vec![
+            PackageFileInput {
+                path: "index.m3u8".to_string(),
+                plaintext: b"#EXTM3U".to_vec(),
+            },
+            PackageFileInput {
+                path: "segments/000001.ts".to_string(),
+                plaintext: b"segment1".to_vec(),
+            },
+        ];
+        let assigned_nonces = [[0x41; 12], [0x42; 12]];
+        let mut nonce_source = assigned_nonces.into_iter();
+
+        let package =
+            build_package_with_nonce_source("asset1", "pub1", "line1", &ck, &inputs, || {
+                nonce_source.next().expect("one nonce per input")
+            })
+            .expect("distinct nonces should build a package");
+
+        assert_eq!(package.files.len(), inputs.len());
+        for (index, sealed) in package.files.iter().enumerate() {
+            let manifest_file = &package.manifest.files[index];
+            assert_eq!(
+                nonce_hex_from_b64u_for_test(&manifest_file.nonce_b64u),
+                assigned_nonces[index]
+            );
+
+            let aad = Aad {
+                aad_version: "p2p-aad-v1".to_string(),
+                asset_id: "asset1".to_string(),
+                lineage_id: "line1".to_string(),
+                manifest_version: "p2p-manifest-v1".to_string(),
+                path: sealed.path.clone(),
+                publication_id: "pub1".to_string(),
+            };
+            let aad_bytes = canonical_aad_json(&aad).into_bytes();
+            let cipher = Aes256Gcm::new_from_slice(&ck).unwrap();
+            #[allow(deprecated)]
+            let nonce = Nonce::from_slice(&assigned_nonces[index]);
+            let plaintext = cipher
+                .decrypt(
+                    nonce,
+                    Payload {
+                        msg: &sealed.ciphertext,
+                        aad: &aad_bytes,
+                    },
+                )
+                .expect("sealed file should decrypt");
+            assert_eq!(plaintext, inputs[index].plaintext);
+        }
+    }
+
+    #[test]
+    fn ec_t2c_r3c_duplicate_nonce_fails_closed() {
+        let ck = [0x31; 32];
+        let inputs = vec![
+            PackageFileInput {
+                path: "index.m3u8".to_string(),
+                plaintext: b"#EXTM3U".to_vec(),
+            },
+            PackageFileInput {
+                path: "segments/000001.ts".to_string(),
+                plaintext: b"segment1".to_vec(),
+            },
+        ];
+        let mut nonce_source = [[0x41; 12], [0x41; 12]].into_iter();
+
+        let result =
+            build_package_with_nonce_source("asset1", "pub1", "line1", &ck, &inputs, || {
+                nonce_source.next().expect("one nonce per input")
+            });
+
+        assert!(matches!(result, Err(PackageBuildError::NonceCollision)));
+    }
+
     #[test]
     fn ec_t2f_1_empty_inputs_is_rejected() {
         let ck = [0u8; 32];
@@ -420,4 +535,8 @@ mod tests {
         let result = build_package("asset1", "pub1", "line1", &ck, &inputs);
         assert!(matches!(result, Err(PackageBuildError::OutOfOrder)));
     }
+
+    fn nonce_hex_from_b64u_for_test(value: &str) -> [u8; 12] {
+        base64url_decode_no_padding(value).try_into().unwrap()
+    }
 }
diff --git a/crates/p2p/src/nonce_tracker.rs b/crates/p2p/src/nonce_tracker.rs
new file mode 100644
index 0000000..8e30f0a
--- /dev/null
+++ b/crates/p2p/src/nonce_tracker.rs
@@ -0,0 +1,41 @@
+use std::collections::HashSet;
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub(crate) struct NonceCollision;
+
+#[derive(Debug, Default)]
+pub(crate) struct NonceTracker {
+    seen: HashSet<[u8; 12]>,
+}
+
+impl NonceTracker {
+    pub(crate) fn register(&mut self, nonce: [u8; 12]) -> Result<(), NonceCollision> {
+        if self.seen.insert(nonce) {
+            Ok(())
+        } else {
+            Err(NonceCollision)
+        }
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn hp_t2c_r2_distinct_nonces_register() {
+        let mut tracker = NonceTracker::default();
+
+        assert_eq!(tracker.register([1; 12]), Ok(()));
+        assert_eq!(tracker.register([2; 12]), Ok(()));
+    }
+
+    #[test]
+    fn ec_t2c_r2_duplicate_nonce_is_rejected() {
+        let mut tracker = NonceTracker::default();
+        let nonce = [7; 12];
+
+        assert_eq!(tracker.register(nonce), Ok(()));
+        assert_eq!(tracker.register(nonce), Err(NonceCollision));
+    }
+}

```

