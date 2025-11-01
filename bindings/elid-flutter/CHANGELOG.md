# Changelog

All notable changes to the ELID Flutter bindings will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-31

### Added
- Initial release of ELID Flutter bindings
- Synchronous encoding/decoding functions (`encode`, `decode`, `hammingDistance`)
- Asynchronous batch encoding (`encodeBatch`)
- Stream-based encoding with progress updates (`encodeStream`)
- Support for three encoding profiles:
  - Mini128: 128-bit SimHash for similarity search
  - Morton10x10: 10D Morton curve for fast indexing
  - Hilbert10x10: 10D Hilbert curve for maximum locality
- Cross-platform support:
  - iOS (arm64, x86_64 simulator)
  - Android (arm64-v8a, armeabi-v7a, x86_64)
  - macOS (arm64, x86_64)
  - Linux (x86_64)
  - Windows (x86_64)
- Comprehensive test suite with cross-language validation
- Example Flutter app demonstrating all features
- Full API documentation and README

### Performance
- Single encoding: <1ms for 768D embedding
- Batch encoding: 5000 embeddings in <10s on mid-range mobile
- Hamming distance: <100μs per comparison
