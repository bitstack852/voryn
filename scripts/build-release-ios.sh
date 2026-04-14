#!/usr/bin/env bash
# Build a release IPA for iOS (requires macOS + Xcode)
#
# Prerequisites:
#   - macOS with Xcode 15+
#   - Apple Developer account configured
#   - Provisioning profile installed
#
# Usage: ./scripts/build-release-ios.sh
# Output: build/voryn-<version>-release.ipa

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"

VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "=== Building Voryn v${VERSION} for iOS ==="

mkdir -p "${BUILD_DIR}"

# ── Step 1: Build Rust native libraries ──────────────────────────
echo "[1/4] Building Rust for iOS targets..."
cd "${PROJECT_ROOT}"

TARGETS=(
    "aarch64-apple-ios"
    "aarch64-apple-ios-sim"
)

for target in "${TARGETS[@]}"; do
    echo "  Building for ${target}..."
    cargo build --release --target "${target}" -p voryn-core
done

# Create universal library for simulator (if building for sim)
echo "  Creating universal library..."
# lipo -create \
#     target/aarch64-apple-ios-sim/release/libvoryn_core.a \
#     -output "${BUILD_DIR}/libvoryn_core_sim.a" 2>/dev/null || true

echo "  Rust build complete."

# ── Step 2: Install JS dependencies ──────────────────────────────
echo "[2/4] Installing dependencies..."
cd "${PROJECT_ROOT}"
yarn install

cd "${PROJECT_ROOT}/apps/mobile/ios"
if [ -f Podfile ]; then
    pod install
fi

# ── Step 3: Build Xcode project ─────────────────────────────────
echo "[3/4] Building Xcode project..."
cd "${PROJECT_ROOT}/apps/mobile/ios"

if [ -f "*.xcworkspace" ] 2>/dev/null || [ -d "Voryn.xcworkspace" ]; then
    xcodebuild \
        -workspace Voryn.xcworkspace \
        -scheme Voryn \
        -configuration Release \
        -sdk iphoneos \
        -archivePath "${BUILD_DIR}/Voryn.xcarchive" \
        archive
else
    echo "  Warning: Xcode workspace not found. Skipping build."
    echo "  This will be available after React Native project init."
fi

# ── Step 4: Export IPA ───────────────────────────────────────────
echo "[4/4] Exporting IPA..."

if [ -d "${BUILD_DIR}/Voryn.xcarchive" ]; then
    cat > "${BUILD_DIR}/ExportOptions.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>app-store</string>
    <key>destination</key>
    <string>upload</string>
</dict>
</plist>
PLIST

    xcodebuild \
        -exportArchive \
        -archivePath "${BUILD_DIR}/Voryn.xcarchive" \
        -exportOptionsPlist "${BUILD_DIR}/ExportOptions.plist" \
        -exportPath "${BUILD_DIR}"

    IPA_NAME="voryn-${VERSION}-release.ipa"
    mv "${BUILD_DIR}/Voryn.ipa" "${BUILD_DIR}/${IPA_NAME}" 2>/dev/null || true

    echo ""
    echo "=== Build Complete ==="
    echo "  IPA: ${BUILD_DIR}/${IPA_NAME}"
else
    echo ""
    echo "=== Build Complete (Rust only) ==="
    echo "  Rust libraries built successfully."
    echo "  Full IPA build requires Xcode project setup."
fi
