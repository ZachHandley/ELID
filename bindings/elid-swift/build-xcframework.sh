#!/bin/bash
set -e

# Build script for creating XCFramework from Rust cdylib
# This builds elid-ffi for iOS and macOS, then packages into an XCFramework

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
FFI_DIR="$REPO_ROOT/bindings/elid-ffi"
OUTPUT_DIR="$SCRIPT_DIR"

echo "🔨 Building ELID XCFramework for iOS and macOS..."

# Add required Rust targets if not already installed
echo "📦 Installing Rust targets..."
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# Build iOS device (arm64)
echo "🍎 Building for iOS device (arm64)..."
cd "$FFI_DIR"
cargo build --lib --release --target aarch64-apple-ios

# Build iOS simulator (x86_64 and arm64)
echo "🍎 Building for iOS simulator (x86_64 and arm64)..."
cargo build --lib --release --target x86_64-apple-ios
cargo build --lib --release --target aarch64-apple-ios-sim

# Build macOS (x86_64 and arm64)
echo "🍏 Building for macOS (x86_64 and arm64)..."
cargo build --lib --release --target x86_64-apple-darwin
cargo build --lib --release --target aarch64-apple-darwin

# Create directories for frameworks
echo "📁 Creating framework structure..."
rm -rf "$OUTPUT_DIR/build"
mkdir -p "$OUTPUT_DIR/build"

# Create iOS device framework
IOS_DEVICE_DIR="$OUTPUT_DIR/build/ios-arm64"
mkdir -p "$IOS_DEVICE_DIR/Headers"
cp "$SCRIPT_DIR/Sources/ElidFFI/ElidFFI.h" "$IOS_DEVICE_DIR/Headers/"
cp "$SCRIPT_DIR/Sources/ElidFFI/ElidFFI.modulemap" "$IOS_DEVICE_DIR/Modules/module.modulemap"
mkdir -p "$IOS_DEVICE_DIR/Modules"
cp "$SCRIPT_DIR/Sources/ElidFFI/ElidFFI.modulemap" "$IOS_DEVICE_DIR/Modules/module.modulemap"
cp "$FFI_DIR/target/aarch64-apple-ios/release/libelid_ffi.a" "$IOS_DEVICE_DIR/libelid_ffi.a"

# Create iOS simulator fat library (x86_64 + arm64)
IOS_SIM_DIR="$OUTPUT_DIR/build/ios-simulator"
mkdir -p "$IOS_SIM_DIR/Headers"
mkdir -p "$IOS_SIM_DIR/Modules"
cp "$SCRIPT_DIR/Sources/ElidFFI/ElidFFI.h" "$IOS_SIM_DIR/Headers/"
cp "$SCRIPT_DIR/Sources/ElidFFI/ElidFFI.modulemap" "$IOS_SIM_DIR/Modules/module.modulemap"

echo "🔗 Creating universal simulator library..."
lipo -create \
    "$FFI_DIR/target/x86_64-apple-ios/release/libelid_ffi.a" \
    "$FFI_DIR/target/aarch64-apple-ios-sim/release/libelid_ffi.a" \
    -output "$IOS_SIM_DIR/libelid_ffi.a"

# Create macOS fat library (x86_64 + arm64)
MACOS_DIR="$OUTPUT_DIR/build/macos"
mkdir -p "$MACOS_DIR/Headers"
mkdir -p "$MACOS_DIR/Modules"
cp "$SCRIPT_DIR/Sources/ElidFFI/ElidFFI.h" "$MACOS_DIR/Headers/"
cp "$SCRIPT_DIR/Sources/ElidFFI/ElidFFI.modulemap" "$MACOS_DIR/Modules/module.modulemap"

echo "🔗 Creating universal macOS library..."
lipo -create \
    "$FFI_DIR/target/x86_64-apple-darwin/release/libelid_ffi.a" \
    "$FFI_DIR/target/aarch64-apple-darwin/release/libelid_ffi.a" \
    -output "$MACOS_DIR/libelid_ffi.a"

# Create XCFramework
echo "📦 Creating XCFramework..."
rm -rf "$OUTPUT_DIR/ElidFFI.xcframework"
xcodebuild -create-xcframework \
    -library "$IOS_DEVICE_DIR/libelid_ffi.a" -headers "$IOS_DEVICE_DIR/Headers" \
    -library "$IOS_SIM_DIR/libelid_ffi.a" -headers "$IOS_SIM_DIR/Headers" \
    -library "$MACOS_DIR/libelid_ffi.a" -headers "$MACOS_DIR/Headers" \
    -output "$OUTPUT_DIR/ElidFFI.xcframework"

# Verify the XCFramework
echo "✅ Verifying XCFramework..."
xcodebuild -checkFirstLaunchStatus

echo ""
echo "✨ XCFramework created successfully at:"
echo "   $OUTPUT_DIR/ElidFFI.xcframework"
echo ""
echo "📊 Framework info:"
xcodebuild -version
echo ""
echo "🎯 Supported platforms:"
ls -la "$OUTPUT_DIR/ElidFFI.xcframework"

# Clean up build directory
echo "🧹 Cleaning up temporary build files..."
rm -rf "$OUTPUT_DIR/build"

echo ""
echo "✅ Build complete!"
