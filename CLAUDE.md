# ELID Development Guidelines

## What This Is

ELID is a fast, zero-dependency Rust string similarity library with multi-language bindings (Python via PyO3, JavaScript via WASM, C via FFI). Single crate, not a workspace.

## Repository

`https://forge.blackleafdigital.com/BlackLeafDigital/ELID` (primary)
`https://github.com/ZachHandley/ELID` (mirror)

## Commands

```bash
cargo test                       # Core + integration tests
cargo test --features ffi        # Include FFI tests
cargo test --all-features        # All features (needs wasm target + pyo3)
cargo clippy --all-features -- -D warnings
cargo fmt -- --check
cargo bench                      # Criterion benchmarks
```

## Architecture

Single crate with feature-gated binding modules:

```
src/
  lib.rs              # Public API: best_match, find_best_match, find_matches_above_threshold, SimilarityOpts
  levenshtein.rs      # levenshtein, normalized_levenshtein, levenshtein_with_opts
  jaro_winkler.rs     # jaro, jaro_winkler, jaro_winkler_with_prefix
  hamming.rs          # hamming, normalized_hamming
  osa.rs              # osa_distance, normalized_osa
  simhash.rs          # simhash (64-bit FNV-1a), simhash_distance, simhash_similarity, find_similar_hashes
  common.rs           # Shared preprocessing (case folding, trimming)
  wasm.rs             # [feature = "wasm"] wasm-bindgen bindings
  python.rs           # [feature = "python"] PyO3 bindings
  ffi.rs              # [feature = "ffi"] C FFI with cbindgen header generation
tests/
  integration_tests.rs  # 20 integration tests covering all algorithms
  python/               # Python binding tests (run via maturin + pip install)
  wasm/                 # WASM tests (Node.js, bundler, web targets)
  ffi/                  # C FFI test source
benches/
  similarity_bench.rs   # Criterion benchmarks
examples/
  basic_usage.rs        # Rust usage examples
  appwrite/             # Appwrite integration examples
  javascript/           # JS/Node examples
  python/               # Python examples
  swift/                # Swift FFI example
site/                   # Astro SSG + Vue demo site (WASM showcase)
```

## Feature Flags

- `default` = [] (zero dependencies for core)
- `wasm` = wasm-bindgen, js-sys, serde, serde-wasm-bindgen, serde_json
- `python` = pyo3 0.22 with extension-module
- `ffi` = enables unsafe code for C bindings; `build.rs` generates `elid.h` via cbindgen

## Key Design Decisions

- `#![deny(missing_docs)]` enforced globally
- `#![deny(unsafe_code)]` enforced except when `ffi` feature is active
- Levenshtein uses O(min(m,n)) space optimization
- SimHash uses 64-bit FNV-1a hashing (stable, deterministic)
- All algorithms support Unicode (char-level, not byte-level)

## CI

Forgejo Actions at `.forgejo/workflows/` using reusable actions from `BlackLeafDigital/actions/`:
- `ci.yml` - check (fmt+clippy) -> test (rust+FFI) -> test-wasm -> test-python
- `release.yml` - tag-triggered publishing
- `benchmark.yml` - criterion + perf assertions
- `security.yml` - cargo-audit + weekly scan
- `docs.yml` - rustdoc + demo site build
