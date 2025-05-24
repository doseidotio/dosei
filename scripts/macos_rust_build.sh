#!/bin/bash
set -e

RESOURCES_PATH="macos/Dosei/Contents/Resources"

mkdir -p "$RESOURCES_PATH"

echo "Building for Apple Silicon (aarch64)..."
cargo build --bin macos-rust --release --target aarch64-apple-darwin
cargo build --bin dosei --release --target aarch64-apple-darwin

echo "Building for Intel (x86_64)..."
cargo build --bin macos-rust --release --target x86_64-apple-darwin
cargo build --bin dosei --release --target x86_64-apple-darwin

echo "Creating universal binary..."
lipo -create \
  "target/aarch64-apple-darwin/release/macos-rust" \
  "target/x86_64-apple-darwin/release/macos-rust" \
  -output "$RESOURCES_PATH/macos-rust"
lipo -create \
  "target/aarch64-apple-darwin/release/dosei" \
  "target/x86_64-apple-darwin/release/dosei" \
  -output "$RESOURCES_PATH/dosei"

chmod +x "$RESOURCES_PATH/macos-rust"
chmod +x "$RESOURCES_PATH/dosei"

cp ./scripts/post_install.sh "$RESOURCES_PATH/post_install.sh"
