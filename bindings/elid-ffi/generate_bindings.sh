#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

echo "Building elid-ffi first..."
cargo build --release

echo "Attempting to generate bindings using uniffi-bindgen from uniffi crate..."

# Try to find and use uniffi-bindgen from the build dependencies
UNIFFI_BINDGEN=$(find ../../target/release/build -name "uniffi-bindgen*" -type f -executable 2>/dev/null | head -1)

if [ -z "$UNIFFI_BINDGEN" ]; then
    echo "uniffi-bindgen executable not found in build artifacts"
    echo "Trying to use Python approach instead..."

    # Alternative: use Python to call uniffi_bindgen
    python3 << 'EOF'
import sys
import os
sys.path.insert(0, "../../target/release/deps")

try:
    from uniffi_bindgen import generate_bindings

    generate_bindings(
        udl_file="src/elid.udl",
        config_file_override="uniffi.toml",
        out_dir="generated",
    )
    print("Bindings generated successfully!")
except ImportError as e:
    print(f"Could not import uniffi_bindgen: {e}")
    print("\nManual generation required. Use these commands:")
    print("  For Swift:  uniffi-bindgen generate src/elid.udl --language swift --out-dir generated/swift")
    print("  For Kotlin: uniffi-bindgen generate src/elid.udl --language kotlin --out-dir generated/kotlin")
    print("  For Ruby:   uniffi-bindgen generate src/elid.udl --language ruby --out-dir generated/ruby")
    sys.exit(1)
EOF
else
    echo "Found uniffi-bindgen at: $UNIFFI_BINDGEN"

    echo "Generating Swift bindings..."
    "$UNIFFI_BINDGEN" generate src/elid.udl --language swift --out-dir generated/swift

    echo "Generating Kotlin bindings..."
    "$UNIFFI_BINDGEN" generate src/elid.udl --language kotlin --out-dir generated/kotlin

    echo "Generating Ruby bindings..."
    "$UNIFFI_BINDGEN" generate src/elid.udl --language ruby --out-dir generated/ruby

    echo "All bindings generated successfully!"
fi
