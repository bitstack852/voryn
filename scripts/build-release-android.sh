#!/usr/bin/env bash
# Build a signed release APK for Android
#
# Prerequisites:
#   - Android SDK + NDK installed
#   - cargo-ndk installed
#   - VORYN_KEYSTORE_PATH and VORYN_KEYSTORE_PASSWORD set (for signing)
#
# Usage: ./scripts/build-release-android.sh
# Output: build/voryn-<version>-release.apk

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"

# Read version from Cargo.toml
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "=== Building Voryn v${VERSION} for Android ==="

mkdir -p "${BUILD_DIR}"

# ── Step 1: Build Rust native libraries ──────────────────────────
echo "[1/4] Building Rust for Android targets..."
cd "${PROJECT_ROOT}"
cargo ndk \
    -t aarch64-linux-android \
    -t armv7-linux-androideabi \
    -t x86_64-linux-android \
    -t i686-linux-android \
    build --release -p voryn-core

echo "  Rust build complete."

# ── Step 2: Install JS dependencies ──────────────────────────────
echo "[2/4] Installing dependencies..."
cd "${PROJECT_ROOT}"
yarn install

# ── Step 3: Build React Native Android ───────────────────────────
echo "[3/4] Building React Native Android release..."
cd "${PROJECT_ROOT}/apps/mobile/android"

if [ -f gradlew ]; then
    ./gradlew assembleRelease
else
    echo "  Warning: Gradle wrapper not found. Skipping RN build."
    echo "  This will be available after React Native project init."
fi

# ── Step 4: Sign APK ────────────────────────────────────────────
echo "[4/4] Signing APK..."
APK_NAME="voryn-${VERSION}-release.apk"
OUTPUT_APK="${BUILD_DIR}/${APK_NAME}"

if [ -n "${VORYN_KEYSTORE_PATH:-}" ] && [ -f "${VORYN_KEYSTORE_PATH}" ]; then
    UNSIGNED_APK="apps/mobile/android/app/build/outputs/apk/release/app-release-unsigned.apk"
    if [ -f "${UNSIGNED_APK}" ]; then
        # Align
        zipalign -v -p 4 "${UNSIGNED_APK}" "${BUILD_DIR}/aligned.apk"
        # Sign
        apksigner sign \
            --ks "${VORYN_KEYSTORE_PATH}" \
            --ks-pass "env:VORYN_KEYSTORE_PASSWORD" \
            --out "${OUTPUT_APK}" \
            "${BUILD_DIR}/aligned.apk"
        rm "${BUILD_DIR}/aligned.apk"
    fi
else
    echo "  Warning: VORYN_KEYSTORE_PATH not set. APK not signed."
    echo "  Set VORYN_KEYSTORE_PATH and VORYN_KEYSTORE_PASSWORD for release builds."
fi

# ── Generate SHA-256 hash ────────────────────────────────────────
if [ -f "${OUTPUT_APK}" ]; then
    sha256sum "${OUTPUT_APK}" > "${OUTPUT_APK}.sha256"
    echo ""
    echo "=== Build Complete ==="
    echo "  APK: ${OUTPUT_APK}"
    echo "  SHA: $(cat "${OUTPUT_APK}.sha256")"
else
    echo ""
    echo "=== Build Complete (Rust only) ==="
    echo "  Rust libraries built successfully."
    echo "  Full APK build requires Android project setup."
fi
