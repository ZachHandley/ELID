# Building ELID Swift Package from Source

This guide explains how to build the ELID Swift package from source, including generating the XCFramework from the Rust FFI layer.

## Prerequisites

### Required Tools

1. **Rust** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Xcode** (15.0+)
   - Install from the Mac App Store
   - Accept license: `sudo xcodebuild -license accept`
   - Install command-line tools: `xcode-select --install`

3. **UniFFI bindgen** (0.25.0)
   ```bash
   cargo install uniffi-bindgen --version 0.25
   ```

4. **Rust Apple Targets**
   ```bash
   # iOS targets
   rustup target add aarch64-apple-ios        # iOS device (arm64)
   rustup target add x86_64-apple-ios         # iOS simulator (Intel)
   rustup target add aarch64-apple-ios-sim    # iOS simulator (Apple Silicon)

   # macOS targets
   rustup target add aarch64-apple-darwin     # macOS Apple Silicon
   rustup target add x86_64-apple-darwin      # macOS Intel
   ```

## Build Process

### Step 1: Generate Swift Bindings

From the `bindings/elid-ffi` directory:

```bash
cd bindings/elid-ffi

# Generate Swift bindings from UniFFI interface
uniffi-bindgen generate src/elid.udl --language swift --out-dir ../elid-swift/Sources/Elid/
```

This creates:
- `Elid.swift` - Swift wrapper code
- `ElidFFI.h` - C header file
- `ElidFFI.modulemap` - Module map for SPM

### Step 2: Build Rust Libraries

Build the Rust FFI library for all target architectures:

```bash
cd bindings/elid-ffi

# iOS device
cargo build --lib --release --target aarch64-apple-ios

# iOS simulator (both architectures)
cargo build --lib --release --target x86_64-apple-ios
cargo build --lib --release --target aarch64-apple-ios-sim

# macOS universal
cargo build --lib --release --target x86_64-apple-darwin
cargo build --lib --release --target aarch64-apple-darwin
```

Build artifacts are in `target/<architecture>/release/libelid_ffi.a`

### Step 3: Create XCFramework

Use the provided build script:

```bash
cd bindings/elid-swift
./build-xcframework.sh
```

Or manually:

```bash
# Create universal libraries
lipo -create \
    ../elid-ffi/target/x86_64-apple-ios/release/libelid_ffi.a \
    ../elid-ffi/target/aarch64-apple-ios-sim/release/libelid_ffi.a \
    -output libelid_ffi_sim.a

lipo -create \
    ../elid-ffi/target/x86_64-apple-darwin/release/libelid_ffi.a \
    ../elid-ffi/target/aarch64-apple-darwin/release/libelid_ffi.a \
    -output libelid_ffi_macos.a

# Create XCFramework
xcodebuild -create-xcframework \
    -library ../elid-ffi/target/aarch64-apple-ios/release/libelid_ffi.a \
        -headers Sources/ElidFFI/ \
    -library libelid_ffi_sim.a \
        -headers Sources/ElidFFI/ \
    -library libelid_ffi_macos.a \
        -headers Sources/ElidFFI/ \
    -output ElidFFI.xcframework
```

### Step 4: Build Swift Package

```bash
cd bindings/elid-swift

# Verify package structure
swift package describe

# Build package
swift build

# Run tests
swift test
```

## Development Workflow

### Quick Rebuild

After changing Rust code:

```bash
# 1. Rebuild Rust library (only for your current platform)
cd bindings/elid-ffi
cargo build --release

# 2. For macOS development, create a simple framework
cd ../elid-swift
rm -rf ElidFFI.xcframework
xcodebuild -create-xcframework \
    -library ../elid-ffi/target/release/libelid_ffi.a \
        -headers Sources/ElidFFI/ \
    -output ElidFFI.xcframework

# 3. Test
swift test
```

### Regenerate Bindings

After modifying `elid.udl`:

```bash
cd bindings/elid-ffi
uniffi-bindgen generate src/elid.udl --language swift --out-dir ../elid-swift/Sources/Elid/

# Copy generated files if needed
cp generated/Elid.swift ../elid-swift/Sources/Elid/
cp generated/ElidFFI.h ../elid-swift/Sources/ElidFFI/
cp generated/ElidFFI.modulemap ../elid-swift/Sources/ElidFFI/
```

## Troubleshooting

### Issue: `uniffi-bindgen: command not found`

**Solution**: Install UniFFI bindgen:
```bash
cargo install uniffi-bindgen --version 0.25
```

### Issue: `error: unable to spawn target`

**Solution**: Install missing Rust target:
```bash
rustup target add <target-triple>
```

### Issue: `module 'ElidFFI' not found`

**Solution**: Ensure XCFramework is built:
```bash
ls -la ElidFFI.xcframework
# If missing:
./build-xcframework.sh
```

### Issue: `xcodebuild: error: SDK "iphonesimulator" cannot be located`

**Solution**: Install Xcode command-line tools:
```bash
xcode-select --install
sudo xcode-select --reset
```

### Issue: Permission denied when running build script

**Solution**: Make script executable:
```bash
chmod +x build-xcframework.sh
```

### Issue: Tests fail with "test vectors not found"

**Solution**: Ensure test vectors are copied:
```bash
cp ../test-vectors.json Tests/ElidTests/
```

## CI/CD Integration

The GitHub Actions workflow (`.github/workflows/bindings-swift.yml`) automates:

1. **Linting**: Rust fmt and clippy
2. **Building**: XCFramework for all platforms
3. **Testing**: Swift tests on macOS
4. **Cross-language validation**: Compare outputs with Python/Node
5. **Benchmarking**: Performance tests
6. **Release packaging**: Zipped XCFramework with checksum

## Platform-Specific Notes

### macOS Development

For local macOS development, you can use a simpler workflow:

```bash
# Build only for current platform
cd bindings/elid-ffi
cargo build --release

# Create minimal XCFramework
cd ../elid-swift
rm -rf ElidFFI.xcframework
xcodebuild -create-xcframework \
    -library ../elid-ffi/target/release/libelid_ffi.a \
    -headers Sources/ElidFFI/ \
    -output ElidFFI.xcframework

swift test
```

### iOS Simulator Development

For testing on iOS simulator:

```bash
# Build for simulator architecture
cd bindings/elid-ffi
cargo build --release --target aarch64-apple-ios-sim  # Apple Silicon Mac
# OR
cargo build --release --target x86_64-apple-ios       # Intel Mac

# Create XCFramework with simulator lib
cd ../elid-swift
xcodebuild -create-xcframework \
    -library ../elid-ffi/target/aarch64-apple-ios-sim/release/libelid_ffi.a \
    -headers Sources/ElidFFI/ \
    -output ElidFFI.xcframework

# Run on simulator
xcodebuild test \
    -scheme Elid \
    -sdk iphonesimulator \
    -destination 'platform=iOS Simulator,name=iPhone 15'
```

### Cross-Compilation from Linux

Cross-compiling for Apple platforms from Linux requires additional setup and is not officially supported. Use macOS or GitHub Actions for building iOS/macOS binaries.

## Distribution

### For Internal Use

Commit the XCFramework to your repository:

```swift
// Package.swift
.binaryTarget(
    name: "ElidFFI",
    path: "ElidFFI.xcframework"
)
```

### For Public Distribution

Host the XCFramework on GitHub releases:

```swift
// Package.swift
.binaryTarget(
    name: "ElidFFI",
    url: "https://github.com/your-org/ELID/releases/download/v0.1.0/ElidFFI.xcframework.zip",
    checksum: "sha256-checksum-here"
)
```

Generate checksum:
```bash
swift package compute-checksum ElidFFI.xcframework.zip
```

## Performance Tips

1. **Always use `--release` builds**: Debug builds are 10-100x slower
2. **Enable LTO**: Add to `elid-ffi/Cargo.toml`:
   ```toml
   [profile.release]
   lto = true
   codegen-units = 1
   ```
3. **Use batch encoding**: Process multiple embeddings at once
4. **Profile with Instruments**: Use Xcode's Time Profiler for optimization

## Next Steps

- See [README.md](README.md) for usage examples
- See [Examples/CoreMLIntegration.swift](Examples/CoreMLIntegration.swift) for integration patterns
- Run tests: `swift test`
- Run benchmarks: `swift test --filter Performance`

## Support

- Report issues: [GitHub Issues](https://github.com/your-org/ELID/issues)
- Ask questions: [GitHub Discussions](https://github.com/your-org/ELID/discussions)
