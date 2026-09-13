//! Test-only fixture binary that builds and materializes a real P2P
//! ciphertext package.
//!
//! Invokes the real, unmodified `dubbridge_p2p::package_builder::build_package`
//! and `dubbridge_p2p::package_writer::materialize` production functions
//! against CLI-supplied inputs, then prints one line of JSON to stdout. It is
//! never invoked by `apps/api`, `apps/worker-runner`, or any shipped surface
//! — its only caller is the Node integration test in
//! `apps/availability-node/test/package-publication-integration.test.js`,
//! via `execFileSync("cargo", ["run", ..., "--bin",
//! "package_build_and_materialize_fixture", ...])`. This exists so that
//! integration test can exercise the actual Rust package-build/materialize
//! pipeline instead of a hand-crafted JS fixture reproducing its on-disk
//! shape.
//!
//! Exit code is always 0; the caller must inspect the JSON `ok` field, not
//! the process exit code, since an intentionally-invalid input (an EC case)
//! must still let the caller parse the structured error from stdout without
//! `execFileSync` throwing on a non-zero exit.

use std::env;
use std::path::PathBuf;

use dubbridge_p2p::package_builder::{PackageBuildError, PackageFileInput, build_package};
use dubbridge_p2p::package_writer::{MaterializeError, materialize};

struct Args {
    root: PathBuf,
    asset_id: String,
    publication_id: String,
    lineage_id: String,
    ck: [u8; 32],
    files: Vec<PackageFileInput>,
}

fn fail_usage(message: &str) -> ! {
    eprintln!("usage error: {message}");
    std::process::exit(2);
}

fn decode_hex_ck(hex: &str) -> [u8; 32] {
    if hex.len() != 64 {
        fail_usage("--ck-hex must be exactly 64 hex characters (32 bytes)");
    }
    let mut ck = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex[i * 2..i * 2 + 2];
        ck[i] = u8::from_str_radix(byte_str, 16).unwrap_or_else(|_| {
            fail_usage("--ck-hex must contain only hex characters");
        });
    }
    ck
}

fn parse_args() -> Args {
    let mut root: Option<PathBuf> = None;
    let mut asset_id: Option<String> = None;
    let mut publication_id: Option<String> = None;
    let mut lineage_id: Option<String> = None;
    let mut ck: Option<[u8; 32]> = None;
    let mut files: Vec<PackageFileInput> = Vec::new();

    let mut argv = env::args().skip(1);
    while let Some(flag) = argv.next() {
        match flag.as_str() {
            "--root" => {
                let value = argv
                    .next()
                    .unwrap_or_else(|| fail_usage("--root needs a value"));
                root = Some(PathBuf::from(value));
            }
            "--asset-id" => {
                asset_id = Some(
                    argv.next()
                        .unwrap_or_else(|| fail_usage("--asset-id needs a value")),
                );
            }
            "--publication-id" => {
                publication_id = Some(
                    argv.next()
                        .unwrap_or_else(|| fail_usage("--publication-id needs a value")),
                );
            }
            "--lineage-id" => {
                lineage_id = Some(
                    argv.next()
                        .unwrap_or_else(|| fail_usage("--lineage-id needs a value")),
                );
            }
            "--ck-hex" => {
                let value = argv
                    .next()
                    .unwrap_or_else(|| fail_usage("--ck-hex needs a value"));
                ck = Some(decode_hex_ck(&value));
            }
            "--file" => {
                let value = argv
                    .next()
                    .unwrap_or_else(|| fail_usage("--file needs a value"));
                let (path, content) = value
                    .split_once('=')
                    .unwrap_or_else(|| fail_usage("--file must be <relpath>=<content>"));
                files.push(PackageFileInput {
                    path: path.to_string(),
                    plaintext: content.as_bytes().to_vec(),
                });
            }
            other => fail_usage(&format!("unrecognized flag: {other}")),
        }
    }

    Args {
        root: root.unwrap_or_else(|| fail_usage("--root is required")),
        asset_id: asset_id.unwrap_or_else(|| fail_usage("--asset-id is required")),
        publication_id: publication_id
            .unwrap_or_else(|| fail_usage("--publication-id is required")),
        lineage_id: lineage_id.unwrap_or_else(|| fail_usage("--lineage-id is required")),
        ck: ck.unwrap_or_else(|| fail_usage("--ck-hex is required")),
        files,
    }
}

fn build_error_debug_string(err: &PackageBuildError) -> String {
    format!("{err:?}")
}

fn materialize_error_debug_string(err: &MaterializeError) -> String {
    format!("{err:?}")
}

fn main() {
    let args = parse_args();

    let package = match build_package(
        &args.asset_id,
        &args.publication_id,
        &args.lineage_id,
        &args.ck,
        &args.files,
    ) {
        Ok(package) => package,
        Err(err) => {
            let output = serde_json::json!({
                "ok": false,
                "stage": "build",
                "error": build_error_debug_string(&err),
            });
            println!("{output}");
            std::process::exit(0);
        }
    };

    let manifest_digest_sha256 = package.manifest_digest_sha256.clone();

    let package_ref = match materialize(&args.root, &package) {
        Ok(package_ref) => package_ref,
        Err(err) => {
            let output = serde_json::json!({
                "ok": false,
                "stage": "materialize",
                "error": materialize_error_debug_string(&err),
            });
            println!("{output}");
            std::process::exit(0);
        }
    };

    let output = serde_json::json!({
        "ok": true,
        "publication_id": args.publication_id,
        "lineage_id": args.lineage_id,
        "manifest_digest_sha256": manifest_digest_sha256,
        "contract_version": "availability-publication-v1",
        "manifest_version": "p2p-manifest-v1",
        "package_dir": package_ref.root.to_string_lossy().to_string(),
    });
    println!("{output}");
}
