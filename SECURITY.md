# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in ELID, please report it responsibly:

### 🔒 **DO NOT** Open a Public Issue

Public disclosure of security vulnerabilities puts the entire community at risk. Instead:

### 📧 Email Security Issues

Send an email to **zachhandley@gmail.com** with:

- **Subject**: `[SECURITY] Brief description of issue`
- **Description**: Detailed description of the vulnerability
- **Impact**: Potential impact if exploited
- **Reproduction**: Steps to reproduce the issue
- **Suggested Fix**: If you have one (optional)

### What to Expect

1. **Acknowledgment**: We'll acknowledge receipt within 48 hours
2. **Assessment**: We'll assess the vulnerability within 1 week
3. **Update**: We'll provide updates on our progress
4. **Fix**: We'll develop a fix and coordinate disclosure
5. **Credit**: We'll credit you in the security advisory (unless you prefer to remain anonymous)

### Timeline

- **Critical vulnerabilities**: Fixed within 7 days
- **High vulnerabilities**: Fixed within 14 days
- **Medium/Low vulnerabilities**: Fixed in next scheduled release

## Security Considerations

### Rust Core (`elid-core`)

- ✅ **No unsafe code**: `#![forbid(unsafe_code)]` enforced
- ✅ **Memory safety**: Guaranteed by Rust's type system
- ✅ **No panics**: All operations return `Result` types
- ✅ **Input validation**: All inputs validated before processing

### FFI Layer (`elid-ffi`)

- ⚠️ **Unsafe code necessary**: FFI requires unsafe blocks
- ✅ **Audited**: All unsafe code is audited and tested
- ✅ **Boundary validation**: Inputs validated at FFI boundary
- ✅ **Memory management**: Clear ownership and deallocation patterns
- ✅ **Null pointer checks**: All pointers validated before dereferencing

### Language Bindings

Each binding follows language-specific security best practices:

- **Python**: PyO3 handles memory safety automatically
- **TypeScript**: napi-rs provides safe FFI abstractions
- **Flutter**: flutter_rust_bridge generates safe code
- **Swift/Kotlin/Ruby**: UniFFI generates safe bindings
- **PHP**: Manual memory management documented and tested

## Known Security Considerations

### 1. Input Validation

**Risk**: Malformed embeddings could cause undefined behavior

**Mitigation**: All embeddings validated for:
- Dimension count (64-2048)
- Non-NaN/Inf values
- Proper normalization (where required)

### 2. Resource Exhaustion

**Risk**: Large batch operations could exhaust memory

**Mitigation**:
- Batch sizes should be limited by caller
- Stream processing for large datasets
- Memory usage is O(n) where n = embedding size

### 3. Timing Attacks

**Risk**: Hamming distance computation could leak information via timing

**Assessment**: Not applicable - ELID IDs are designed to be public identifiers, not secrets. Timing variations are not a security concern for this use case.

### 4. FFI Memory Safety

**Risk**: Manual memory management in PHP bindings could cause issues

**Mitigation**:
- All C functions thoroughly tested
- Memory leak testing (100+ iterations)
- Clear documentation of ownership
- Valgrind testing in CI (future)

## Security Best Practices for Users

### When Using ELID

1. **Validate inputs**: Check embedding dimensions before encoding
2. **Handle errors**: Don't ignore `Result` types in Rust or exceptions in other languages
3. **Limit batch sizes**: Don't load millions of embeddings into memory at once
4. **Update regularly**: Keep ELID updated to latest version
5. **Review dependencies**: Audit your dependency tree regularly

### When Storing ELIDs

1. **Index properly**: Use appropriate database indexes
2. **Don't expose raw IDs unnecessarily**: While ELIDs are designed to be public, consider your threat model
3. **Validate on input**: When accepting ELIDs from external sources, validate the format

## Dependency Security

We monitor our dependencies for security vulnerabilities using:

- **Rust**: `cargo audit` (runs in CI)
- **Python**: Dependabot
- **TypeScript**: npm audit
- **Other bindings**: Language-specific tools

## Security Updates

Security updates will be:

1. **Released immediately** for critical vulnerabilities
2. **Announced** via GitHub Security Advisories
3. **Documented** in CHANGELOG.md
4. **Coordinated** if affecting multiple bindings

Subscribe to security advisories:
- Watch the repository
- Enable GitHub Security Advisories notifications

## Responsible Disclosure

We follow responsible disclosure practices:

1. Vulnerabilities are fixed before public disclosure
2. Security advisories published after fix is available
3. Credit given to reporters (with permission)
4. 90-day disclosure timeline (can be extended if needed)

## Security Checklist for Contributors

When submitting code:

- [ ] No unsafe code added to `elid-core` (unless absolutely necessary and documented)
- [ ] All inputs validated
- [ ] Error handling is correct (no unwrap/panic in library code)
- [ ] FFI boundary checks for null pointers
- [ ] Memory management is documented
- [ ] Security implications considered and documented

## Contact

For security issues: **zachhandley@gmail.com**

For general questions: [GitHub Discussions](https://github.com/zachhandley/ELID/discussions)

---

Last updated: 2025-10-31
