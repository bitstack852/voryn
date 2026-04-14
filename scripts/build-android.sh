#!/usr/bin/env bash
set -euo pipefail

echo "Building Voryn Rust core for Android targets..."

if ! command -v cargo-ndk &> /dev/null; then
    echo "Error: cargo-ndk not found. Install with: cargo install cargo-ndk"
    exit 1
fi

cargo ndk \
    -t aarch64-linux-android \
    -t armv7-linux-androideabi \
    -t x86_64-linux-android \
    -t i686-linux-android \
    build --release -p voryn-core

echo "Android builds complete."
