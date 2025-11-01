# ELID Ruby Gem

Ruby bindings for **ELID** (Embedding Locality-Preserving IDs) - a high-performance library for generating sortable, locality-preserving identifiers from high-dimensional embeddings.

Built with Rust via [UniFFI](https://mozilla.github.io/uniffi-rs/) for maximum performance and safety.

## Features

- **🚀 Fast**: Rust-powered encoding with zero-copy FFI
- **📊 Three Encoding Profiles**:
  - `MINI128`: 128-bit SimHash for angular distance preservation
  - `MORTON10X10`: Morton Z-order curve for fast multi-dimensional indexing
  - `HILBERT10X10`: Hilbert curve for superior locality preservation
- **🔤 Lexicographically Sortable**: Base32hex encoding ensures database-friendly sorting
- **🔒 Type Safe**: Ruby wrapper with comprehensive error handling
- **⚡ Batch Processing**: Efficient parallel encoding for multiple embeddings

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'elid'
```

And then execute:

```bash
bundle install
```

Or install it yourself:

```bash
gem install elid
```

## Quick Start

```ruby
require 'elid'

# Encode a single embedding
embedding = Array.new(128) { rand }
elid = Elid.encode(embedding, Elid::Profile::MINI128)
# => "040fc2k94e9gs9eifsprppnfkqg04"

# Batch encode multiple embeddings (parallel processing)
embeddings = Array.new(1000) { Array.new(128) { rand } }
elids = Elid.encode_batch(embeddings, Elid::Profile::MINI128)
# => ["040fc2k94...", "040bs8490...", ...]

# Compute Hamming distance (Mini128 only)
distance = Elid.hamming_distance(elid1, elid2)
# => 42  (0-128, lower = more similar)

# Decode back to raw bytes
bytes = Elid.decode(elid)
# => [1, 0, 250, 10, ...]
```

## Encoding Profiles

### Mini128 - SimHash for Cosine Similarity

Best for: **Angular distance preservation**, semantic similarity search

```ruby
embedding = Array.new(128) { rand }  # 64-2048 dimensions supported
elid = Elid.encode(embedding, Elid::Profile::MINI128)

# Compute similarity via Hamming distance
distance = Elid.hamming_distance(elid1, elid2)
cosine_similarity_approx = 1.0 - (distance / 64.0)
```

**Output**: 29-character base32hex string (2-byte header + 128-bit payload)

**Use cases**:
- Semantic search
- Duplicate detection
- Clustering by similarity

### Morton10x10 - Z-Order Curve

Best for: **Fast range queries**, multi-dimensional indexing

```ruby
embedding = Array.new(1024) { rand }  # Must be 1024 dimensions
elid = Elid.encode(embedding, Elid::Profile::MORTON10X10)
```

**Output**: 24-character base32hex string

**Use cases**:
- Spatial databases
- Fast nearest-neighbor approximation
- Range queries in embedding space

### Hilbert10x10 - Space-Filling Curve

Best for: **Maximum locality preservation** (5-10% better than Morton)

```ruby
embedding = Array.new(1024) { rand }  # Must be 1024 dimensions
elid = Elid.encode(embedding, Elid::Profile::HILBERT10X10)
```

**Output**: 24-character base32hex string

**Use cases**:
- Database indexing where locality is critical
- Efficient range scans
- Better clustering than Morton

## Rails Integration

### Using ELID as Primary Keys

```ruby
# db/migrate/20250101000000_create_embeddings.rb
class CreateEmbeddings < ActiveRecord::Migration[7.0]
  def change
    create_table :embeddings, id: false do |t|
      t.string :id, primary_key: true, limit: 29  # ELID
      t.text :vector  # JSON array
      t.timestamps
    end

    add_index :embeddings, :id, unique: true
  end
end

# app/models/embedding.rb
class Embedding < ApplicationRecord
  before_create :generate_elid

  serialize :vector, JSON

  private

  def generate_elid
    self.id = Elid.encode(vector, Elid::Profile::MINI128)
  end
end

# Usage
embedding = Embedding.create!(vector: Array.new(128) { rand })
# => #<Embedding id="040fc2k94e9gs9eifsprppnfkqg04", ...>

# Find similar embeddings (assuming indexed)
similar = Embedding.where("id LIKE ?", "#{elid[0..5]}%").limit(10)
```

### Caching with Redis

```ruby
# config/initializers/elid_cache.rb
class ElidCache
  def self.set(embedding, value, ttl: 3600)
    elid = Elid.encode(embedding, Elid::Profile::MINI128)
    Rails.cache.write(elid, value, expires_in: ttl)
    elid
  end

  def self.get(elid)
    Rails.cache.read(elid)
  end

  def self.get_by_embedding(embedding)
    elid = Elid.encode(embedding, Elid::Profile::MINI128)
    Rails.cache.read(elid)
  end
end

# Usage
embedding = [0.1, 0.2, 0.3, ...]
ElidCache.set(embedding, { result: "cached data" })
ElidCache.get_by_embedding(embedding)  # => { result: "cached data" }
```

## API Reference

### `Elid.encode(embedding, profile) -> String`

Encode a single embedding into an ELID string.

**Parameters**:
- `embedding` (Array<Float>): Embedding vector (64-2048 dimensions)
- `profile` (Integer): Encoding profile constant

**Returns**: Base32hex-encoded ELID string

**Raises**: `Elid::Error` if dimensions are invalid or values are NaN/infinite

### `Elid.decode(elid) -> Array<Integer>`

Decode an ELID string back to raw bytes.

**Parameters**:
- `elid` (String): ELID string to decode

**Returns**: Array of bytes (with header)

**Raises**: `Elid::Error` if ELID format is invalid

### `Elid.hamming_distance(elid1, elid2) -> Integer`

Compute Hamming distance between two Mini128 ELIDs.

**Parameters**:
- `elid1` (String): First ELID (must be Mini128 profile)
- `elid2` (String): Second ELID (must be Mini128 profile)

**Returns**: Hamming distance (0-128)

**Raises**: `Elid::Error` if ELIDs are not Mini128 profile

### `Elid.encode_batch(embeddings, profile) -> Array<String>`

Encode multiple embeddings in parallel using Rayon.

**Parameters**:
- `embeddings` (Array<Array<Float>>): Array of embedding vectors
- `profile` (Integer): Encoding profile constant

**Returns**: Array of ELID strings

**Raises**: `Elid::Error` if any embedding is invalid

## Performance

Benchmarks on Apple M1 Pro (Rust backend):

- **Single encode**: ~1-2 μs per embedding
- **Batch encode (1000 items)**: ~500 μs (parallel)
- **Hamming distance**: ~50 ns per comparison
- **Decode**: ~100 ns per ELID

Ruby FFI overhead adds ~100-200 ns per call (negligible for batch operations).

## Error Handling

All operations raise `Elid::Error` with descriptive messages:

```ruby
begin
  elid = Elid.encode(embedding, Elid::Profile::MINI128)
rescue Elid::Error => e
  puts "Encoding failed: #{e.message}"
  # => "Encoding failed: Invalid embedding dimension: expected 64-2048, got 32"
end
```

**Common errors**:
- **InvalidDimension**: Embedding has < 64 or > 2048 dimensions
- **InvalidValue**: Embedding contains NaN or infinite values
- **InvalidEncoding**: ELID string has invalid base32hex characters
- **ProfileMismatch**: Hamming distance called on non-Mini128 ELIDs

## Development

```bash
# Clone repository
git clone https://github.com/zachhandley/ELID
cd ELID/bindings/elid-ruby

# Install dependencies
bundle install

# Build native library
cd ../elid-ffi
cargo build --release
cd ../elid-ruby

# Run tests
bundle exec rspec

# Build gem
gem build elid.gemspec
```

## Testing

```bash
bundle exec rspec
```

Test coverage includes:
- ✅ All three encoding profiles
- ✅ Batch encoding
- ✅ Hamming distance calculations
- ✅ Error handling (invalid dimensions, NaN, infinity)
- ✅ Deterministic encoding
- ✅ Lexicographic sorting
- ✅ Cross-language validation (via test vectors)

## Platform Support

The gem includes precompiled native extensions for:

- **Linux**: x86_64-linux-gnu (libelid_ffi.so)
- **macOS**: x86_64-darwin, arm64-darwin (libelid_ffi.dylib)
- **Windows**: x86_64-windows (elid_ffi.dll) - experimental

Minimum Ruby version: **3.0.0**

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please see the main [ELID repository](https://github.com/zachhandley/ELID) for guidelines.

## Links

- [Documentation](https://github.com/zachhandley/ELID)
- [Rust Core Library](../../elid-core)
- [UniFFI Bindings](../elid-ffi)
- [Issue Tracker](https://github.com/zachhandley/ELID/issues)

## Acknowledgments

- Built with [UniFFI](https://mozilla.github.io/uniffi-rs/) for cross-language FFI
- SimHash algorithm: Charikar, Moses S. (2002). "Similarity estimation techniques from rounding algorithms"
- Hilbert curve implementation based on research by Skilling (2004)
