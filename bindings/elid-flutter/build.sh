#!/bin/bash
set -e

echo "Building ELID Flutter bindings..."

# Check for required tools
command -v flutter >/dev/null 2>&1 || { echo "Flutter not found. Please install Flutter."; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Cargo not found. Please install Rust."; exit 1; }

# Install flutter_rust_bridge_codegen if not already installed
if ! command -v flutter_rust_bridge_codegen >/dev/null 2>&1; then
    echo "Installing flutter_rust_bridge_codegen..."
    cargo install flutter_rust_bridge_codegen --version 2.11.1
fi

# Get Flutter dependencies
echo "Getting Flutter dependencies..."
flutter pub get

# Generate bridge code
echo "Generating bridge code..."
flutter_rust_bridge_codegen generate

# Format generated Dart code
echo "Formatting Dart code..."
dart format lib/

# Build Rust library
echo "Building Rust library..."
cd rust
cargo build --release
cd ..

echo "Build complete!"
