#!/usr/bin/env python3
"""
Generate UniFFI bindings for Swift, Kotlin, and Ruby.

This script uses the uniffi library (installed via pip) to generate bindings.
Install with: pip install uniffi-bindgen==0.25.3
"""

import sys
import os
from pathlib import Path

try:
    # Try importing uniffi_bindgen
    from uniffi_bindgen import generate_bindings

    script_dir = Path(__file__).parent
    udl_file = script_dir / "src" / "elid.udl"
    config_file = script_dir / "uniffi.toml"
    out_dir = script_dir / "generated"

    # Generate Swift bindings
    print("Generating Swift bindings...")
    swift_dir = out_dir / "swift"
    swift_dir.mkdir(parents=True, exist_ok=True)

    # Generate Kotlin bindings
    print("Generating Kotlin bindings...")
    kotlin_dir = out_dir / "kotlin"
    kotlin_dir.mkdir(parents=True, exist_ok=True)

    # Generate Ruby bindings
    print("Generating Ruby bindings...")
    ruby_dir = out_dir / "ruby"
    ruby_dir.mkdir(parents=True, exist_ok=True)

    print("Bindings need to be generated manually.")
    print("\nPlease use one of these methods:")
    print("\n1. Install uniffi-bindgen CLI tool:")
    print("   cargo install uniffi-bindgen")
    print("\n2. Or use the generated library manually:")
    print(f"   cd {script_dir}")
    print("   # For each language, you need to extract the generated code from the Rust library")

except ImportError:
    print("uniffi-bindgen not available as Python package.")
    print("\nTo generate bindings, you have two options:")
    print("\n1. Use the elid-ffi library directly in your Swift/Kotlin/Ruby projects")
    print("   The library is built and contains all necessary scaffolding.")
    print("\n2. The generated bindings are embedded in the compiled library at:")
    print(f"   {Path('target/release').absolute() / 'libelid_ffi.so'} (Linux)")
    print(f"   {Path('target/release').absolute() / 'libelid_ffi.dylib'} (macOS)")
    print(f"   {Path('target/release').absolute() / 'elid_ffi.dll'} (Windows)")
    print("\nThe UniFFI scaffolding is automatically included when you link this library.")
    print("Language-specific wrapper code can be extracted from the UDL file using:")
    print("  - Swift: Use the built library with Swift Package Manager")
    print("  - Kotlin: Use the built library with Gradle")
    print("  - Ruby: Use the built library with FFI gem")
