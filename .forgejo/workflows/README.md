# Forgejo/Gitea Workflows

This directory contains CI/CD workflows for ELID. These workflows are compatible with both Forgejo and Gitea.

## Workflows

### 🧪 `ci.yml` - Continuous Integration

**Triggers:** Push to main/develop, Pull requests
**Purpose:** Runs comprehensive tests across all platforms

**Jobs:**
- **rust-test** - Run all Rust core tests (38 tests)
- **c-ffi-test** - Test C FFI bindings (7 Rust + 10 C tests)
- **wasm-test** - Build and test all WASM targets (Node.js, Bundler, Web)
- **python-test** - Test Python bindings across Python 3.9, 3.10, 3.11
- **fmt-clippy** - Check code formatting and linting
- **docs** - Build documentation and verify C header generation
- **ci-success** - Summary job (requires all others to pass)

**Caching:** Cargo registry, build artifacts, npm packages

**Duration:** ~5-10 minutes

---

### 🚀 `release.yml` - Release & Publishing

**Triggers:** Tag push (v*.*.*), Manual dispatch
**Purpose:** Build release artifacts and publish to package registries

**Jobs:**

**Build Jobs:**
- **build-rust** - Cross-compile for multiple targets (Linux x64/ARM64, macOS Intel/ARM)
- **build-c-ffi** - Package C FFI library with headers (Linux, macOS)
- **build-wasm** - Build all WASM targets (Node.js, Bundler, Web)
- **build-python** - Build Python wheels for multiple OS and Python versions

**Release Jobs:**
- **create-release** - Create GitHub/Forgejo release with artifacts
- **publish-crates** - Publish to crates.io
- **publish-npm** - Publish WASM package to npm
- **publish-pypi** - Publish Python wheels to PyPI

**Artifacts Generated:**
- Rust libraries (`.so`, `.dylib`)
- C FFI packages with headers
- WASM packages (3 targets)
- Python wheels (multiple platforms)

**Requirements:**
- Secrets: `CARGO_TOKEN`, `NPM_TOKEN`, `PYPI_TOKEN`

---

### ⚡ `benchmark.yml` - Performance Benchmarks

**Triggers:** Push to main, Pull requests, Manual dispatch
**Purpose:** Track performance over time

**Jobs:**
- **benchmark** - Run Criterion.rs benchmarks
- **performance-test** - Quick Python performance test (ensures >500K ops/sec)
- **wasm-performance** - WASM performance test (ensures >300K ops/sec)
- **size-check** - Verify binary sizes (WASM <150KB)

**Benchmarks:**
- Levenshtein distance
- Jaro-Winkler similarity
- Hamming distance
- SimHash operations

**Artifacts:** Criterion benchmark results

---

### 🔒 `security.yml` - Security Audit

**Triggers:** Push to main/develop, Pull requests, Weekly schedule (Monday 00:00 UTC), Manual dispatch
**Purpose:** Security scanning and vulnerability detection

**Jobs:**
- **cargo-audit** - Check for known vulnerabilities in dependencies
- **unsafe-check** - Scan for unsafe code usage (using cargo-geiger)
- **dependency-review** - Check for outdated dependencies
- **wasm-security** - WASM binary analysis
- **python-security** - Check Python dependency security
- **license-check** - Verify license compliance
- **coverage** - Generate code coverage report
- **docs-security** - Scan documentation for exposed secrets

**Security Checks:**
- Known CVEs in dependencies
- Unsafe code usage patterns
- License compatibility
- Potential secret exposure
- Code coverage tracking

---

### 📚 `docs.yml` - Documentation

**Triggers:** Push to main, Manual dispatch
**Purpose:** Build and validate documentation

**Jobs:**
- **build-deploy-docs** - Build Rust documentation with rustdoc
- **validate-docs** - Verify all required markdown files exist
- **build-c-docs** - Generate C header file (elid.h)
- **build-ts-defs** - Generate TypeScript definitions
- **generate-badges** - Create shields.io badges

**Artifacts:**
- Rust API documentation
- C header file
- TypeScript definitions
- Markdown badges

**Validated Files:**
- README.md
- WASM_README.md
- PYTHON_README.md
- C_FFI_README.md
- APPWRITE.md
- SIMHASH_GUIDE.md
- PRODUCTION.md
- TEST_REPORT.md

---

## GitHub Actions Compatibility

These workflows use standard GitHub Actions syntax and are compatible with:
- ✅ Forgejo Actions
- ✅ Gitea Actions
- ✅ GitHub Actions

## Required Secrets

For full functionality, configure these secrets in your repository settings:

### Publishing (Optional)
- `CARGO_TOKEN` - crates.io API token for publishing Rust crate
- `NPM_TOKEN` - npm authentication token for publishing WASM package
- `PYPI_TOKEN` - PyPI API token for publishing Python package

### Coverage (Optional)
- `CODECOV_TOKEN` - Codecov token for coverage reporting

## Workflow Status Badges

Add to your README.md:

```markdown
![CI](https://img.shields.io/github/actions/workflow/status/ZachHandley/ELID/ci.yml?branch=main&label=CI)
![Security](https://img.shields.io/github/actions/workflow/status/ZachHandley/ELID/security.yml?branch=main&label=Security)
![Tests](https://img.shields.io/badge/tests-93%2B%20passing-brightgreen)
```

## Local Testing

You can test workflows locally using [act](https://github.com/nektos/act):

```bash
# Test CI workflow
act -j rust-test

# Test all CI jobs
act push

# Test release workflow (dry run)
act -n release
```

## Caching Strategy

Workflows use GitHub Actions cache to speed up builds:

- **Cargo registry** - Cached by Cargo.lock hash
- **Cargo build** - Cached by Cargo.lock hash
- **npm packages** - Cached by package-lock.json hash

Cache is automatically invalidated when dependencies change.

## Performance Targets

| Binding | Minimum Performance | Typical Performance |
|---------|-------------------|-------------------|
| Python | 500,000 ops/sec | 1,400,000 ops/sec |
| WASM | 300,000 ops/sec | 800,000 ops/sec |
| Rust | N/A (native) | 5,000,000+ ops/sec |

Workflows will fail if performance drops below minimum thresholds.

## Size Limits

| Artifact | Maximum Size | Typical Size |
|----------|-------------|-------------|
| WASM (Node) | 150 KB | 96 KB |
| WASM (Bundler) | 150 KB | 96 KB |
| WASM (Web) | 150 KB | 96 KB |

## Troubleshooting

### Workflow fails with "No space left on device"

**Solution:** Clear GitHub Actions cache or increase runner disk space.

### Python wheel build fails

**Solution:** Ensure maturin is installed and PyO3 version matches Cargo.toml.

### WASM build produces small file (472 bytes)

**Solution:** Ensure `--features wasm` flag is included in build command.

### Cargo audit fails

**Solution:** Update dependencies with `cargo update` or add advisories to ignore list.

## Contributing

When adding new workflows:
1. Test locally with `act` if possible
2. Use caching for faster builds
3. Add job to appropriate workflow summary
4. Document in this README
5. Use meaningful job names

## Support

For issues with workflows:
- Check job logs in Actions tab
- Verify all required tools are installed
- Check secret configuration
- Review recent commits for breaking changes

## License

Same as project: MIT OR Apache-2.0
