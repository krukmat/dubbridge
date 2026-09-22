#!/usr/bin/env python3
"""Static guards for the P3.T3c mobile/native/logging secret boundary."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

P3_API = [
    ROOT / "apps/api/src/routes/p2p_audience.rs",
    ROOT / "apps/api/src/routes/p2p_envelope.rs",
]
DEVICE_TS_ROOT = ROOT / "mobile/src/p2p/device"
NATIVE_ROOT = (
    ROOT
    / "mobile/modules/dubbridge-p2p-keystore/android/src/main/java/com/dubbridge/p2pkeystore"
)
AUDIT_SOURCE = ROOT / "crates/audit/src/lib.rs"

RUST_LOGGING = (
    "dbg!(",
    "println!(",
    "eprintln!(",
    "tracing::trace!(",
    "tracing::debug!(",
    "tracing::info!(",
    "tracing::warn!(",
    "tracing::error!(",
)
TS_LOGGING_OR_STORAGE = (
    "console.",
    "AsyncStorage",
    "SecureStore",
    "localStorage",
)
KOTLIN_LOGGING_OR_STORAGE = (
    "android.util.Log",
    "Log.",
    "println(",
    "print(",
    "SharedPreferences",
    "EncryptedSharedPreferences",
    "FileOutputStream",
    "openFileOutput(",
)
REQUIRED_NATIVE_GUARDS = (
    'KeyStore.getInstance("AndroidKeyStore")',
    "privateKey.encoded == null",
    "sharedSecret.fill(0)",
    "dh.fill(0)",
    "ciphertext.fill(0)",
    "plaintext.fill(0)",
    "key.fill(0)",
    "nonce.fill(0)",
)


def read(path: Path) -> str:
    if not path.is_file():
        raise SystemExit(f"P3_T3C_SECRET_BOUNDARY=FAIL missing file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def reject_tokens(path: Path, tokens: tuple[str, ...]) -> None:
    source = read(path)
    for token in tokens:
        if token in source:
            raise SystemExit(
                "P3_T3C_SECRET_BOUNDARY=FAIL "
                f"{path.relative_to(ROOT)} contains forbidden boundary token: {token}"
            )


def main() -> None:
    for path in P3_API:
        reject_tokens(path, RUST_LOGGING)

    device_files = sorted(DEVICE_TS_ROOT.glob("*.ts"))
    if not device_files:
        raise SystemExit("P3_T3C_SECRET_BOUNDARY=FAIL no device TypeScript boundary files")
    for path in device_files:
        reject_tokens(path, TS_LOGGING_OR_STORAGE)

    native_files = sorted(NATIVE_ROOT.glob("*.kt"))
    if not native_files:
        raise SystemExit("P3_T3C_SECRET_BOUNDARY=FAIL no native Keystore boundary files")
    for path in native_files:
        reject_tokens(path, KOTLIN_LOGGING_OR_STORAGE)

    native_module = read(NATIVE_ROOT / "DubBridgeP2PKeyStoreModule.kt")
    for required in REQUIRED_NATIVE_GUARDS:
        if required not in native_module:
            raise SystemExit(
                "P3_T3C_SECRET_BOUNDARY=FAIL "
                f"native transient-material guard missing: {required}"
            )

    audit_source = read(AUDIT_SOURCE)
    for forbidden in (
        "detail = event.detail",
        "detail = ?event.detail",
        "event = ?event",
        "event = %event",
    ):
        if forbidden in audit_source:
            raise SystemExit(
                "P3_T3C_SECRET_BOUNDARY=FAIL audit trace may expose event detail"
            )

    for required in (
        "correlation_id = event.correlation_id",
        "publication_id = event.publication_id",
        "lineage_id = event.lineage_id",
        "event_kind = %event.event_kind",
    ):
        if required not in audit_source:
            raise SystemExit(
                "P3_T3C_SECRET_BOUNDARY=FAIL safe audit correlation field missing"
            )

    print("P3_T3C_SECRET_BOUNDARY=PASS")


if __name__ == "__main__":
    main()
