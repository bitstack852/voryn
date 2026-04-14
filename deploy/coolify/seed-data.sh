#!/usr/bin/env bash
# Seed the update server with initial version.json
#
# Run this ONCE after deploying, to populate the update-data volume.
#
# Usage (from Coolify server or via docker exec):
#   docker exec -it <update-server-container> sh -c "$(cat seed-data.sh)"
#
# Or run locally and copy into the volume:
#   ./seed-data.sh

set -euo pipefail

DATA_DIR="${1:-/usr/share/nginx/html}"

mkdir -p "${DATA_DIR}/releases"

cat > "${DATA_DIR}/version.json" << 'EOF'
{
  "latest": "0.1.0",
  "minimum": "0.1.0",
  "android_url": "https://updates.voryn.bitstack.website/releases/voryn-0.1.0-release.apk",
  "ios_url": "",
  "changelog": "Initial release — infrastructure setup",
  "released_at": "2026-04-14T00:00:00Z"
}
EOF

echo "Seeded version.json at ${DATA_DIR}/version.json"
echo "Ready for APK uploads to ${DATA_DIR}/releases/"
