#!/usr/bin/env bash
set -euo pipefail

echo "Building Voryn Rust core for iOS targets..."

TARGETS=(
    "aarch64-apple-ios"
    "aarch64-apple-ios-sim"
    "x86_64-apple-ios"
)

for target in "${TARGETS[@]}"; do
    echo "  Building for $target..."
    cargo build --release --target "$target" -p voryn-core
done

echo "iOS builds complete."
