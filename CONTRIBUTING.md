# Contributing to ELID

Thank you for your interest in contributing to ELID! This document provides guidelines for contributing to the project across all implementations (Rust core + 7 language bindings).

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Contribution Workflow](#contribution-workflow)
- [Testing Guidelines](#testing-guidelines)
- [Code Style](#code-style)
- [Language-Specific Guidelines](#language-specific-guidelines)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and considerate
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Assume good intentions

## Getting Started

### Prerequisites

**Core Development** (Rust):
- Rust 1.70+ (MSRV)
- Cargo

**Language Bindings** (optional, depending on what you're contributing to):
- **Python**: Python 3.9+, uv, maturin
- **TypeScript**: Node.js 18+, npm
- **Flutter**: Flutter SDK 3.0+, Dart 3.0+
- **Swift**: Xcode 14+, Swift 5.9+
- **Kotlin**: JDK 11+, Gradle 8+
- **Ruby**: Ruby 3.0+, bundler
- **PHP**: PHP 7.4+, Composer

### Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/ELID.git
cd ELID

# Add upstream remote
git remote add upstream https://github.com/zachhandley/ELID.git
```

### Build Everything

```bash
# Core Rust library and CLI
cargo build --workspace

# Run all Rust tests
cargo test --workspace

# Run benchmarks
cargo bench
```

## Development Environment

### Rust Core (`elid-core` and `elid-cli`)

```bash
# Build
cargo build --release

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Documentation
cargo doc --open
```

### Language Bindings

Each binding has its own development workflow. See binding-specific READMEs:

- [Python](bindings/elid-python/README.md)
- [TypeScript](bindings/elid-node/README.md)
- [Flutter](bindings/elid-flutter/README.md)
- [Swift](bindings/elid-swift/README.md)
- [Kotlin](bindings/elid-kotlin/README.md)
- [Ruby](bindings/elid-ruby/README.md)
- [PHP](bindings/elid-php/README.md)

## Project Structure

```
ELID/
├── elid-core/          # Core Rust library (CRITICAL PATH)
├── elid-cli/           # Command-line interface
├── bindings/           # Language bindings
│   ├── elid-ffi/       # FFI foundation (C + UniFFI)
│   ├── elid-python/    # Python bindings
│   ├── elid-node/      # TypeScript/Node.js bindings
│   ├── elid-flutter/   # Flutter/Dart bindings
│   ├── elid-swift/     # Swift bindings
│   ├── elid-kotlin/    # Kotlin bindings
│   ├── elid-ruby/      # Ruby bindings
│   └── elid-php/       # PHP bindings
├── scripts/            # Build and test scripts
└── .github/workflows/  # CI/CD pipelines
```

### Critical Paths

Changes to these components affect ALL bindings:

1. **`elid-core/src/`** - Core encoding algorithms
2. **`bindings/elid-ffi/src/elid.udl`** - UniFFI interface (affects Swift, Kotlin, Ruby)
3. **`bindings/elid-ffi/src/lib.rs`** - FFI exports (affects PHP and UniFFI bindings)

If you modify these, run ALL binding tests to ensure nothing breaks.

## Contribution Workflow

### 1. Create an Issue (for major changes)

For significant changes:
- Open an issue describing the problem or feature
- Wait for discussion and approval
- Reference the issue in your PR

### 2. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation only
- `refactor/description` - Code refactoring
- `perf/description` - Performance improvements

### 3. Make Your Changes

- Follow the [Code Style](#code-style) guidelines
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 4. Commit Your Changes

```bash
git add .
git commit -m "Brief description of changes"
```

Commit message format:
```
<type>: <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

Example:
```
feat: add Hilbert curve neighbor search

Implements k-nearest neighbor search using Hilbert curve
locality properties. Improves search accuracy by 5-10% vs
Morton encoding.

Closes #123
```

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub.

## Testing Guidelines

### Rust Core Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Property-Based Testing

We use `proptest` for property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode_roundtrip(embedding in prop::collection::vec(any::<f32>(), 64..=2048)) {
        let profile = Profile::default();
        let elid = encode(&embedding, &profile)?;
        let decoded = decode(&elid)?;
        // Verify properties...
    }
}
```

### Cross-Language Validation

After changes to core or FFI:

```bash
# Run cross-language test script
./scripts/cross-language-test.sh
```

This validates that all 7 language bindings produce byte-identical output.

### Benchmarks

Before and after performance changes:

```bash
# Baseline
cargo bench > baseline.txt

# Make changes...

# Compare
cargo bench > after.txt
diff baseline.txt after.txt
```

## Code Style

### Rust

Follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

Key conventions:
- Use `snake_case` for functions and variables
- Use `PascalCase` for types
- Maximum line length: 100 characters
- Document all public APIs with `///` comments
- Use `?` for error propagation
- Prefer explicit types over `turbofish` when ambiguous

### Python

Follow PEP 8:

```bash
cd bindings/elid-python
black src/
ruff check src/
mypy src/
```

### TypeScript

Follow the project's ESLint config:

```bash
cd bindings/elid-node
npm run lint
npm run typecheck
```

### Other Languages

See language-specific style guides in each binding's README.

## Language-Specific Guidelines

### Rust Core (`elid-core`)

**DO:**
- ✅ Follow the MSRV (1.70+)
- ✅ Forbid unsafe code (except in `elid-ffi`)
- ✅ Use `#![forbid(unsafe_code)]` in library crates
- ✅ Document all public APIs
- ✅ Add property-based tests for new algorithms
- ✅ Run benchmarks for performance-critical code

**DON'T:**
- ❌ Add dependencies without discussion
- ❌ Break backward compatibility without major version bump
- ❌ Use unstable Rust features

### FFI Layer (`elid-ffi`)

**DO:**
- ✅ Keep C exports minimal and focused
- ✅ Document memory management requirements
- ✅ Add integration tests for FFI functions
- ✅ Update `.udl` file when changing UniFFI interfaces

**DON'T:**
- ❌ Change function signatures without updating ALL bindings
- ❌ Leak memory (verify with Valgrind or similar)
- ❌ Use complex types in C exports (keep to primitives + pointers)

### Python Bindings (`elid-python`)

**DO:**
- ✅ Use PyO3 best practices
- ✅ Provide type stubs (.pyi files)
- ✅ Add pytest tests for new features
- ✅ Document NumPy compatibility

**DON'T:**
- ❌ Break zero-copy where possible
- ❌ Forget to release GIL for long operations

### TypeScript Bindings (`elid-node`)

**DO:**
- ✅ Generate TypeScript definitions automatically
- ✅ Support both Node.js and browser (WASM fallback)
- ✅ Use vitest for testing
- ✅ Provide async APIs for batch operations

**DON'T:**
- ❌ Block the event loop
- ❌ Forget to test WASM target

### Other Bindings

See language-specific READMEs for detailed guidelines.

## Documentation

### Code Documentation

**Rust:**
```rust
/// Encode an embedding into an ELID string.
///
/// # Arguments
///
/// * `embedding` - Float array of 64-2048 dimensions
/// * `profile` - Encoding profile (Mini128, Morton, Hilbert)
///
/// # Returns
///
/// * `Ok(Elid)` - Encoded identifier
/// * `Err(ElidError)` - If dimensions invalid or encoding fails
///
/// # Examples
///
/// ```
/// use elid_core::{encode, Profile};
///
/// let embedding = vec![0.1, 0.2, 0.3, /* ... */];
/// let profile = Profile::default();
/// let elid = encode(&embedding, &profile)?;
/// ```
pub fn encode(embedding: &[f32], profile: &Profile) -> Result<Elid, ElidError> {
    // Implementation...
}
```

**Python:**
```python
def encode(embedding: np.ndarray, profile: Profile) -> str:
    """
    Encode an embedding into an ELID string.

    Args:
        embedding: NumPy array of float32 values (64-2048 dimensions)
        profile: Encoding profile (Profile.Mini128, Profile.Morton10x10, etc.)

    Returns:
        ELID string (24-29 characters, base32hex encoded)

    Raises:
        ValueError: If embedding dimensions are invalid
        RuntimeError: If encoding fails

    Examples:
        >>> import numpy as np
        >>> embedding = np.random.randn(768).astype(np.float32)
        >>> elid = encode(embedding, Profile.Mini128)
        >>> print(len(elid))
        29
    """
```

### Project Documentation

Update relevant documentation files:
- **README.md** - High-level overview, quick start
- **CONTRIBUTING.md** - This file
- **Binding READMEs** - Language-specific guides
- **IMPLEMENTATION_REPORT.md** - Implementation details

### Examples

Add runnable examples for new features:

```bash
# Rust example
examples/my_feature.rs

# Python example
bindings/elid-python/examples/my_feature.py

# etc.
```

## Pull Request Process

### Before Submitting

- [ ] All tests pass locally
- [ ] Code is formatted (`cargo fmt`, etc.)
- [ ] No linter warnings (`cargo clippy`, etc.)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] Cross-language tests pass (if modifying core/FFI)

### PR Description Template

```markdown
## Description

Brief description of changes.

## Motivation

Why is this change necessary? What problem does it solve?

## Changes

- Bullet list of changes
- Be specific

## Testing

How was this tested?
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manual testing performed
- [ ] Cross-language tests pass

## Checklist

- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All tests pass
- [ ] No breaking changes (or documented)

## Related Issues

Closes #123
```

### Review Process

1. **Automated Checks**: CI must pass (all platforms, all tests)
2. **Code Review**: At least one maintainer approval required
3. **Discussion**: Address feedback and make requested changes
4. **Approval**: Once approved, maintainers will merge

### Merge Requirements

- ✅ All CI checks passing
- ✅ At least 1 approving review
- ✅ No unresolved conversations
- ✅ Branch up to date with main
- ✅ Signed commits (recommended)

## Types of Contributions

### Bug Fixes

- **Priority**: High
- **Process**: Open issue → create PR → reference issue
- **Testing**: Add regression test to prevent recurrence

### New Features

- **Priority**: Medium
- **Process**: Open issue for discussion → get approval → implement
- **Requirements**:
  - Tests covering new functionality
  - Documentation
  - Examples (if user-facing)

### Performance Improvements

- **Priority**: Medium
- **Process**: Benchmark before/after → create PR with results
- **Requirements**:
  - Benchmarks showing improvement
  - No accuracy/correctness regressions

### Documentation

- **Priority**: High (always welcome!)
- **Process**: Direct PR (no issue needed for docs-only changes)
- **Examples**:
  - Fix typos
  - Improve clarity
  - Add examples
  - Translate documentation

### Adding Language Bindings

Want to add another language (Go, C#, etc.)?

1. Open an issue proposing the new language
2. Discuss approach (manual FFI vs UniFFI vs other)
3. Implement following existing binding patterns
4. Add comprehensive tests
5. Add CI/CD workflow
6. Document thoroughly

**Minimum requirements for new bindings:**
- Cross-language validation (byte-identical output)
- Comprehensive test suite
- CI/CD pipeline
- Complete README with examples
- Performance benchmarks

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue
- **Chat**: (future: Discord/Slack link)
- **Email**: zachhandley@gmail.com

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in relevant documentation

Thank you for contributing to ELID! 🚀
