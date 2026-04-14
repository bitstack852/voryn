#!/usr/bin/env bash
# Deploy a release to the update server
#
# Prerequisites:
#   - SSH access to the update server as voryn-deploy user
#   - APK file in build/ directory
#
# Usage: ./scripts/deploy-release.sh 0.2.0
#
# This script:
#   1. Uploads the APK and SHA-256 hash
#   2. Updates version.json
#   3. Verifies the upload

set -euo pipefail

VERSION="${1:?Usage: deploy-release.sh <version>}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"

UPDATE_SERVER="updates.voryn.app"
DEPLOY_USER="voryn-deploy"
WEB_ROOT="/var/www/voryn-updates"

APK_FILE="${BUILD_DIR}/voryn-${VERSION}-release.apk"
APK_HASH="${APK_FILE}.sha256"

echo "=== Deploying Voryn v${VERSION} ==="

# ── Verify files exist ───────────────────────────────────────────
if [ ! -f "${APK_FILE}" ]; then
    echo "Error: APK not found: ${APK_FILE}"
    echo "Run scripts/build-release-android.sh first."
    exit 1
fi

if [ ! -f "${APK_HASH}" ]; then
    echo "Generating SHA-256 hash..."
    sha256sum "${APK_FILE}" > "${APK_HASH}"
fi

echo "  APK:  $(basename "${APK_FILE}") ($(du -h "${APK_FILE}" | cut -f1))"
echo "  SHA:  $(cat "${APK_HASH}" | cut -d' ' -f1)"

# ── Upload APK + hash ───────────────────────────────────────────
echo ""
echo "[1/3] Uploading APK..."
scp "${APK_FILE}" "${DEPLOY_USER}@${UPDATE_SERVER}:${WEB_ROOT}/releases/"
scp "${APK_HASH}" "${DEPLOY_USER}@${UPDATE_SERVER}:${WEB_ROOT}/releases/"

# ── Update version.json ─────────────────────────────────────────
echo "[2/3] Updating version.json..."
RELEASED_AT=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

ssh "${DEPLOY_USER}@${UPDATE_SERVER}" "cat > ${WEB_ROOT}/version.json" << VERSIONJSON
{
  "latest": "${VERSION}",
  "minimum": "${VERSION}",
  "android_url": "https://${UPDATE_SERVER}/releases/voryn-${VERSION}-release.apk",
  "ios_url": "",
  "changelog": "Release v${VERSION}",
  "released_at": "${RELEASED_AT}"
}
VERSIONJSON

# ── Verify ───────────────────────────────────────────────────────
echo "[3/3] Verifying deployment..."
REMOTE_VERSION=$(curl -sf "https://${UPDATE_SERVER}/version.json" | grep -o '"latest":"[^"]*"' | cut -d'"' -f4)

if [ "${REMOTE_VERSION}" = "${VERSION}" ]; then
    echo ""
    echo "=== Deployment Successful ==="
    echo "  Version:  ${VERSION}"
    echo "  URL:      https://${UPDATE_SERVER}/releases/voryn-${VERSION}-release.apk"
    echo "  Verified: version.json reports latest=${REMOTE_VERSION}"
else
    echo ""
    echo "=== Deployment Warning ==="
    echo "  version.json may not have updated yet (CDN cache)."
    echo "  Expected: ${VERSION}, Got: ${REMOTE_VERSION:-<empty>}"
    echo "  Check: curl https://${UPDATE_SERVER}/version.json"
fi
