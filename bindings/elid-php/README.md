# ELID PHP Bindings

PHP bindings for ELID (Embedding Locality-preserving Identifiers) using PHP FFI.

## Requirements

- PHP 7.4 or higher
- FFI extension enabled (built-in since PHP 7.4)
- ELID native library (libelid_ffi.so/dylib/dll)

## Installation

### Via Composer

```bash
composer require elid/elid
```

### Building the Native Library

Before using the PHP bindings, you need to build the ELID FFI library:

```bash
cd bindings/elid-ffi
cargo build --release
```

The library will be in `target/release/`:
- Linux: `libelid_ffi.so`
- macOS: `libelid_ffi.dylib`
- Windows: `elid_ffi.dll`

### Library Location

The PHP bindings will automatically search for the library in:
1. Path specified by `ELID_LIBRARY_PATH` environment variable
2. `../../elid-ffi/target/release/` (development)
3. `../../elid-ffi/target/debug/` (development)
4. `/usr/local/lib/` (system-wide)
5. `/usr/lib/` (system-wide)
6. Current working directory

You can set a custom path:

```bash
export ELID_LIBRARY_PATH=/path/to/libelid_ffi.so
```

## Usage

### Basic Encoding

```php
<?php

require 'vendor/autoload.php';

use Elid\Elid;

// Create an embedding vector (768 dimensions)
$embedding = array_fill(0, 768, 0.5);

// Encode with Mini128 profile (SimHash-based)
$elid = Elid::encode($embedding, Elid::MINI128);
echo "ELID: {$elid}\n";

// Decode back to bytes
$bytes = Elid::decode($elid);
echo "Decoded bytes: " . strlen($bytes) . " bytes\n";
```

### Encoding Profiles

ELID supports three encoding profiles:

```php
// Mini128 - SimHash-based, 128-bit, locality-preserving (default)
$elid1 = Elid::encode($embedding, Elid::MINI128);

// Morton10x10 - Z-order curve, fast encoding
$elid2 = Elid::encode($embedding, Elid::MORTON10X10);

// Hilbert10x10 - Hilbert curve, better locality than Morton
$elid3 = Elid::encode($embedding, Elid::HILBERT10X10);
```

### Hamming Distance

Calculate Hamming distance between two Mini128 ELIDs:

```php
$embedding1 = array_fill(0, 768, 0.1);
$embedding2 = array_fill(0, 768, 0.2);

$elid1 = Elid::encode($embedding1, Elid::MINI128);
$elid2 = Elid::encode($embedding2, Elid::MINI128);

$distance = Elid::hammingDistance($elid1, $elid2);
echo "Hamming distance: {$distance} bits\n";
```

**Note:** Hamming distance only works with Mini128 profile.

### Batch Processing

Encode multiple embeddings at once:

```php
$embeddings = [
    array_fill(0, 768, 0.1),
    array_fill(0, 768, 0.2),
    array_fill(0, 768, 0.3),
];

$elids = Elid::encodeBatch($embeddings, Elid::MINI128);

foreach ($elids as $i => $elid) {
    echo "Embedding {$i}: {$elid}\n";
}
```

### Error Handling

```php
use Elid\Exception\ElidException;

try {
    // Invalid dimension (must be 64-2048)
    $invalidEmbedding = array_fill(0, 50, 0.5);
    $elid = Elid::encode($invalidEmbedding);
} catch (ElidException $e) {
    echo "Error: {$e->getMessage()}\n";
}
```

## API Reference

### `Elid::encode(array $embedding, int $profile): string`

Encode an embedding vector into an ELID string.

**Parameters:**
- `$embedding` - Array of floats (64-2048 dimensions)
- `$profile` - Profile constant: `MINI128`, `MORTON10X10`, or `HILBERT10X10` (default: `MINI128`)

**Returns:** ELID string (29 characters, base32hex encoded)

**Throws:** `ElidException` on invalid input or encoding failure

### `Elid::decode(string $elid): string`

Decode an ELID string to raw bytes.

**Parameters:**
- `$elid` - ELID string to decode

**Returns:** Binary string containing decoded bytes

**Throws:** `ElidException` on invalid ELID or decoding failure

### `Elid::hammingDistance(string $elid1, string $elid2): int`

Compute Hamming distance between two Mini128 ELIDs.

**Parameters:**
- `$elid1` - First ELID string (must be Mini128)
- `$elid2` - Second ELID string (must be Mini128)

**Returns:** Hamming distance (0-128 bits)

**Throws:** `ElidException` if ELIDs are not Mini128 profile

### `Elid::encodeBatch(array $embeddings, int $profile): array`

Encode multiple embeddings at once.

**Parameters:**
- `$embeddings` - Array of embedding arrays
- `$profile` - Profile constant (default: `MINI128`)

**Returns:** Array of ELID strings

**Throws:** `ElidException` if any encoding fails

## Constants

- `Elid::MINI128` (0) - SimHash-based 128-bit encoding
- `Elid::MORTON10X10` (1) - Z-order curve encoding
- `Elid::HILBERT10X10` (2) - Hilbert curve encoding

## Testing

Run the test suite:

```bash
composer install
composer test
```

Or with PHPUnit directly:

```bash
vendor/bin/phpunit --colors=always
```

## Performance Considerations

- **Memory Management**: The library automatically manages memory cleanup using `elid_free_string` and `elid_free_bytes` to prevent leaks
- **Batch Processing**: Use `encodeBatch()` for multiple embeddings to reduce FFI overhead
- **Profile Selection**: Morton is fastest, Hilbert has best locality, Mini128 supports Hamming distance

## WordPress Integration Example

See `examples/wordpress_plugin.php` for a complete WordPress plugin example.

## Platform Support

- **Linux**: Tested on Ubuntu 20.04+
- **macOS**: Tested on macOS 12+
- **Windows**: Requires PHP 7.4+ with FFI enabled

## License

MIT OR Apache-2.0

## Contributing

See the main ELID repository for contribution guidelines.
