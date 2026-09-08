use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use dubbridge_p2p::aad::{Aad, canonical_aad_json};
use dubbridge_p2p::key_wrap::wrap_ck;
use dubbridge_p2p::package_builder::{PackageFileInput, build_package};
use serde_json::{Value, json};
use std::process::{Command, Stdio};

const NODE_CONTRACT: &str = r#"
const crypto = require('node:crypto');
const fs = require('node:fs');

const input = JSON.parse(fs.readFileSync(0, 'utf8'));
const hex = (value) => Buffer.from(value, 'hex');

function decrypt(keyHex, nonceHex, ciphertextHex, aad) {
  const ciphertextWithTag = hex(ciphertextHex);
  const tag = ciphertextWithTag.subarray(ciphertextWithTag.length - 16);
  const ciphertext = ciphertextWithTag.subarray(0, ciphertextWithTag.length - 16);
  const decipher = crypto.createDecipheriv('aes-256-gcm', hex(keyHex), hex(nonceHex));
  decipher.setAAD(Buffer.from(aad, 'utf8'));
  decipher.setAuthTag(tag);
  return Buffer.concat([decipher.update(ciphertext), decipher.final()]);
}

function expectReject(action, label) {
  try {
    action();
  } catch (_) {
    return;
  }
  throw new Error(`${label} was accepted`);
}

if (input.mode === 'positive') {
  const vectorPlaintext = decrypt(
    input.vector.key_hex,
    input.vector.nonce_hex,
    input.vector.ciphertext_hex,
    input.vector.aad,
  );
  if (vectorPlaintext.toString('hex') !== input.vector.plaintext_hex) {
    throw new Error('NIST AES-256-GCM vector did not decrypt');
  }

  const wrappedCk = decrypt(
    input.wrap.kek_hex,
    input.wrap.nonce_hex,
    input.wrap.ciphertext_hex,
    input.wrap.aad,
  );
  if (wrappedCk.toString('hex') !== input.package.ck_hex) {
    throw new Error('wrapped CK did not decrypt');
  }

  for (const file of input.package.files) {
    const plaintext = decrypt(
      input.package.ck_hex,
      file.nonce_hex,
      file.ciphertext_hex,
      file.aad,
    );
    if (plaintext.toString('hex') !== file.plaintext_hex) {
      throw new Error(`package plaintext mismatch for ${file.path}`);
    }
    const digest = crypto.createHash('sha256').update(hex(file.ciphertext_hex)).digest('hex');
    if (digest !== file.ciphertext_sha256) {
      throw new Error(`ciphertext digest mismatch for ${file.path}`);
    }
  }

  const manifestDigest = crypto
    .createHash('sha256')
    .update(input.package.manifest_canonical_json, 'utf8')
    .digest('hex');
  if (manifestDigest !== input.package.manifest_digest_sha256) {
    throw new Error('canonical manifest digest mismatch');
  }
} else if (input.mode === 'negative') {
  for (const file of input.package.files) {
    expectReject(
      () => decrypt(input.package.ck_hex, file.nonce_hex, file.tampered_ciphertext_hex, file.aad),
      `tampered ciphertext for ${file.path}`,
    );
    expectReject(
      () => decrypt(input.package.ck_hex, file.nonce_hex, file.ciphertext_hex, file.tampered_aad),
      `tampered AAD for ${file.path}`,
    );
  }
  expectReject(
    () => decrypt(input.wrap.kek_hex, input.wrap.nonce_hex, input.wrap.tampered_ciphertext_hex, input.wrap.aad),
    'tampered wrapped CK',
  );
} else {
  throw new Error('unknown contract mode');
}
process.stdout.write('ok');
"#;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn nonce_hex_from_b64u(nonce_b64u: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut values = [255u8; 256];
    for (index, byte) in ALPHABET.iter().enumerate() {
        values[*byte as usize] = index as u8;
    }

    let mut decoded = Vec::new();
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    for byte in nonce_b64u.bytes() {
        accumulator = (accumulator << 6) | values[byte as usize] as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            decoded.push((accumulator >> bits) as u8);
        }
    }
    hex(&decoded)
}

fn flip_last_byte(mut hex_value: String) -> String {
    let index = hex_value.len() - 1;
    let replacement = if &hex_value[index..] == "0" { "1" } else { "0" };
    hex_value.replace_range(index.., replacement);
    hex_value
}

fn wrap_contract(ck: &[u8; 32], kek: &[u8; 32]) -> Value {
    let wrapped = wrap_ck(ck, kek, "server-kek", 7).expect("CK must wrap");
    let aad = json!({
        "aad_version": "p2p-kek-wrap-v1",
        "kek_id": wrapped.kek_id,
        "kek_version": wrapped.kek_version,
    })
    .to_string();

    json!({
        "kek_hex": hex(kek),
        "nonce_hex": hex(&wrapped.nonce),
        "ciphertext_hex": hex(&wrapped.ciphertext),
        "tampered_ciphertext_hex": flip_last_byte(hex(&wrapped.ciphertext)),
        "aad": aad,
    })
}

fn node_contract_input() -> Value {
    let ck = [0x11; 32];
    let kek = [0x22; 32];
    let inputs = vec![
        PackageFileInput {
            path: "index.m3u8".to_string(),
            plaintext: b"#EXTM3U\n#EXT-X-VERSION:7\n".to_vec(),
        },
        PackageFileInput {
            path: "segments/000001.ts".to_string(),
            plaintext: b"first encrypted segment".to_vec(),
        },
    ];
    let package = build_package("asset-k1", "publication-k1", "lineage-k1", &ck, &inputs)
        .expect("valid package input must seal");

    let files: Vec<Value> = package
        .files
        .iter()
        .zip(&package.manifest.files)
        .zip(&inputs)
        .map(|((sealed, manifest), input)| {
            let aad = Aad {
                aad_version: "p2p-aad-v1".to_string(),
                asset_id: "asset-k1".to_string(),
                lineage_id: "lineage-k1".to_string(),
                manifest_version: "p2p-manifest-v1".to_string(),
                path: sealed.path.clone(),
                publication_id: "publication-k1".to_string(),
            };
            let aad = canonical_aad_json(&aad);
            let tampered_aad = aad.replace(&sealed.path, "segments/tampered.ts");
            let ciphertext_hex = hex(&sealed.ciphertext);
            json!({
                "path": sealed.path,
                "plaintext_hex": hex(&input.plaintext),
                "ciphertext_hex": ciphertext_hex,
                "tampered_ciphertext_hex": flip_last_byte(hex(&sealed.ciphertext)),
                "ciphertext_sha256": manifest.ciphertext_sha256,
                "nonce_hex": nonce_hex_from_b64u(&manifest.nonce_b64u),
                "aad": aad,
                "tampered_aad": tampered_aad,
            })
        })
        .collect();

    json!({
        "mode": "positive",
        "vector": {
            "key_hex": "0000000000000000000000000000000000000000000000000000000000000000",
            "nonce_hex": "000000000000000000000000",
            "plaintext_hex": "00000000000000000000000000000000",
            "ciphertext_hex": "cea7403d4d606b6e074ec5d3baf39d18d0d1c8a799996bf0265b98b5d48ab919",
            "aad": "",
        },
        "package": {
            "ck_hex": hex(&ck),
            "files": files,
            "manifest_canonical_json": package.manifest_canonical_json,
            "manifest_digest_sha256": package.manifest_digest_sha256,
        },
        "wrap": wrap_contract(&ck, &kek),
    })
}

fn run_node_contract(input: &Value) {
    let mut node = Command::new("node")
        .args(["--input-type=commonjs", "-e", NODE_CONTRACT])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node.js is required for the K1 cross-runtime contract");
    let stdin = node.stdin.as_mut().expect("Node stdin must be available");
    serde_json::to_writer(stdin, input).expect("contract input must serialize");
    let output = node.wait_with_output().expect("Node process must complete");
    assert!(
        output.status.success(),
        "Node K1 contract failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"ok");
}

#[test]
fn hp_t2g_1_nist_aes_256_gcm_vector_decrypts_in_rust() {
    let key = [0u8; 32];
    let nonce = [0u8; 12];
    let ciphertext_and_tag = [
        0xce, 0xa7, 0x40, 0x3d, 0x4d, 0x60, 0x6b, 0x6e, 0x07, 0x4e, 0xc5, 0xd3, 0xba, 0xf3, 0x9d,
        0x18, 0xd0, 0xd1, 0xc8, 0xa7, 0x99, 0x99, 0x6b, 0xf0, 0x26, 0x5b, 0x98, 0xb5, 0xd4, 0x8a,
        0xb9, 0x19,
    ];
    let cipher = Aes256Gcm::new_from_slice(&key).expect("32-byte key is valid");
    #[allow(deprecated)]
    let nonce = Nonce::from_slice(&nonce);
    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: &ciphertext_and_tag,
                aad: b"",
            },
        )
        .expect("NIST AES-256-GCM vector must decrypt");
    assert_eq!(plaintext, [0u8; 16]);
}

#[test]
fn hp_t2g_2_node_decrypts_rust_package_and_wrapped_ck() {
    run_node_contract(&node_contract_input());
}

#[test]
fn ec_t2g_1_node_rejects_tampered_ciphertext_aad_and_wrapped_ck() {
    let mut input = node_contract_input();
    input["mode"] = json!("negative");
    run_node_contract(&input);
}
