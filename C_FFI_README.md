# ELID C FFI Bindings

ELID provides C-compatible FFI bindings that can be used from C, C++, Swift, Objective-C, Go, Ruby, and many other languages.

## Building the C Library

Build the library with the `ffi` feature:

```bash
cargo build --release --features ffi
```

This will:
1. Build the shared library (`libelid.so` on Linux, `libelid.dylib` on macOS, `elid.dll` on Windows)
2. Generate the C header file `elid.h`

## Files Generated

- **`elid.h`** - C header file with all function declarations
- **`target/release/libelid.so`** (or `.dylib`/`.dll`) - Shared library

## Installation

### Linux/macOS

```bash
# Copy header
sudo cp elid.h /usr/local/include/

# Copy library
sudo cp target/release/libelid.so /usr/local/lib/  # Linux
# Or for macOS:
# sudo cp target/release/libelid.dylib /usr/local/lib/

# Update library cache (Linux only)
sudo ldconfig
```

### Homebrew (macOS)

You can create a Homebrew formula or install manually as shown above.

## Usage

### C

```c
#include <stdio.h>
#include "elid.h"

int main(void) {
    // Levenshtein distance
    uintptr_t dist = elid_levenshtein("kitten", "sitting");
    printf("Distance: %zu\n", dist);  // 3

    // Normalized similarity
    double sim = elid_normalized_levenshtein("hello", "hallo");
    printf("Similarity: %.2f\n", sim);  // 0.80

    // Jaro-Winkler
    double jw = elid_jaro_winkler("martha", "marhta");
    printf("Jaro-Winkler: %.3f\n", jw);  // 0.961

    // SimHash for database queries
    uint64_t hash1 = elid_simhash("iPhone 14");
    uint64_t hash2 = elid_simhash("iPhone 15");
    uint32_t distance = elid_simhash_distance(hash1, hash2);
    printf("Hash distance: %u\n", distance);  // Low number = similar

    return 0;
}
```

**Compile:**
```bash
gcc -o myapp myapp.c -lelid
# Or specify paths:
gcc -o myapp myapp.c -I/usr/local/include -L/usr/local/lib -lelid
```

### C++

```cpp
#include <iostream>
#include <string>
extern "C" {
    #include "elid.h"
}

int main() {
    std::string a = "programming";
    std::string b = "programmer";

    auto dist = elid_levenshtein(a.c_str(), b.c_str());
    std::cout << "Distance: " << dist << std::endl;

    auto sim = elid_normalized_levenshtein(a.c_str(), b.c_str());
    std::cout << "Similarity: " << sim << std::endl;

    return 0;
}
```

**Compile:**
```bash
g++ -o myapp myapp.cpp -lelid
```

### Swift

```swift
import Foundation

// In a real project, create a module map or bridging header
// Example module.modulemap:
// module ELID {
//     header "/usr/local/include/elid.h"
//     link "elid"
//     export *
// }

let distance = elid_levenshtein("kitten", "sitting")
print("Distance: \(distance)")

let similarity = elid_normalized_levenshtein("hello", "hallo")
print("Similarity: \(similarity)")

// SimHash for Swift arrays
let products = ["iPhone 14", "iPhone 15", "Galaxy S23"]
let hashes = products.map { elid_simhash($0) }

let queryHash = elid_simhash("iPhone 14")
for (index, hash) in hashes.enumerated() {
    let dist = elid_simhash_distance(queryHash, hash)
    print("\(products[index]): distance \(dist)")
}
```

**Compile:**
```bash
swiftc -o myapp main.swift -I/usr/local/include -L/usr/local/lib -lelid
```

### Objective-C

```objc
#import <Foundation/Foundation.h>
#include "elid.h"

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSString *str1 = @"kitten";
        NSString *str2 = @"sitting";

        uintptr_t dist = elid_levenshtein(
            [str1 UTF8String],
            [str2 UTF8String]
        );
        NSLog(@"Distance: %zu", dist);

        double sim = elid_normalized_levenshtein(
            [str1 UTF8String],
            [@"kitty" UTF8String]
        );
        NSLog(@"Similarity: %.2f", sim);
    }
    return 0;
}
```

### Go (CGO)

```go
package main

/*
#cgo LDFLAGS: -lelid
#include <elid.h>
*/
import "C"
import "fmt"

func main() {
    dist := C.elid_levenshtein(C.CString("kitten"), C.CString("sitting"))
    fmt.Printf("Distance: %d\n", dist)

    sim := C.elid_normalized_levenshtein(C.CString("hello"), C.CString("hallo"))
    fmt.Printf("Similarity: %.2f\n", sim)

    hash := C.elid_simhash(C.CString("iPhone 14"))
    fmt.Printf("Hash: %d\n", hash)
}
```

**Run:**
```bash
CGO_LDFLAGS="-L/usr/local/lib" go run main.go
```

### Python (ctypes)

While we have PyO3 bindings, you can also use ctypes:

```python
import ctypes

# Load library
lib = ctypes.CDLL("/usr/local/lib/libelid.so")

# Define function signatures
lib.elid_levenshtein.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.elid_levenshtein.restype = ctypes.c_size_t

lib.elid_normalized_levenshtein.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.elid_normalized_levenshtein.restype = ctypes.c_double

# Use the functions
dist = lib.elid_levenshtein(b"kitten", b"sitting")
print(f"Distance: {dist}")

sim = lib.elid_normalized_levenshtein(b"hello", b"hallo")
print(f"Similarity: {sim}")
```

**Note:** For Python, we recommend using the PyO3 bindings instead (`pip install elid`).

### Ruby (FFI)

```ruby
require 'ffi'

module ELID
  extend FFI::Library
  ffi_lib 'elid'

  attach_function :elid_levenshtein, [:string, :string], :size_t
  attach_function :elid_normalized_levenshtein, [:string, :string], :double
  attach_function :elid_jaro_winkler, [:string, :string], :double
  attach_function :elid_simhash, [:string], :uint64
  attach_function :elid_simhash_distance, [:uint64, :uint64], :uint32
end

dist = ELID.elid_levenshtein("kitten", "sitting")
puts "Distance: #{dist}"

sim = ELID.elid_normalized_levenshtein("hello", "hallo")
puts "Similarity: #{sim}"

hash1 = ELID.elid_simhash("iPhone 14")
hash2 = ELID.elid_simhash("iPhone 15")
distance = ELID.elid_simhash_distance(hash1, hash2)
puts "Hash distance: #{distance}"
```

## API Reference

### String Similarity Functions

#### `elid_levenshtein`
```c
uintptr_t elid_levenshtein(const char *a, const char *b);
```
Returns the minimum number of single-character edits needed to transform one string into another.

#### `elid_normalized_levenshtein`
```c
double elid_normalized_levenshtein(const char *a, const char *b);
```
Returns normalized similarity between 0.0 (different) and 1.0 (identical).

#### `elid_jaro`
```c
double elid_jaro(const char *a, const char *b);
```
Compute Jaro similarity (0.0 to 1.0). Good for short strings like names.

#### `elid_jaro_winkler`
```c
double elid_jaro_winkler(const char *a, const char *b);
```
Compute Jaro-Winkler similarity with prefix bonus (0.0 to 1.0).

#### `elid_hamming`
```c
int64_t elid_hamming(const char *a, const char *b);
```
Returns the number of differing positions. Returns -1 if strings have different lengths.

#### `elid_osa_distance`
```c
uintptr_t elid_osa_distance(const char *a, const char *b);
```
Optimal String Alignment distance (like Levenshtein but counts transpositions).

#### `elid_best_match`
```c
double elid_best_match(const char *a, const char *b);
```
Runs multiple algorithms and returns the highest similarity score (0.0 to 1.0).

### SimHash Functions

#### `elid_simhash`
```c
uint64_t elid_simhash(const char *text);
```
Generate a 64-bit SimHash fingerprint. Similar strings produce similar hashes.

#### `elid_simhash_distance`
```c
uint32_t elid_simhash_distance(uint64_t hash1, uint64_t hash2);
```
Compute Hamming distance between two SimHash values. Lower values = more similar.

#### `elid_simhash_similarity`
```c
double elid_simhash_similarity(const char *a, const char *b);
```
Compute normalized SimHash similarity (0.0 to 1.0).

### Utility Functions

#### `elid_version`
```c
const char *elid_version(void);
```
Returns the library version string (does not need to be freed).

#### `elid_free_string`
```c
void elid_free_string(char *s);
```
Free a string allocated by Rust. **Must** be called on all strings returned by ELID functions.

#### `elid_free_match_array`
```c
void elid_free_match_array(struct ElidMatchArray array);
```
Free a match array allocated by Rust.

## Memory Management

**Important:** All strings passed to ELID functions must be null-terminated UTF-8 strings.

**NULL Safety:** All functions check for NULL pointers and return safe defaults:
- Distance functions return 0
- Similarity functions return 0.0
- `elid_hamming` returns -1

## Thread Safety

All ELID functions are thread-safe and can be called from multiple threads simultaneously.

## Performance

The C FFI has minimal overhead - function calls are direct and there's no serialization. Performance is nearly identical to native Rust performance:

- **Levenshtein**: O(m×n) time, O(min(m,n)) space
- **Jaro-Winkler**: O(m×n) time, O(1) space
- **Hamming**: O(n) time, O(1) space
- **SimHash**: O(n) time, O(1) space

## iOS/macOS Integration

### Using in Xcode

1. **Add the library to your project:**
   ```bash
   cargo build --release --features ffi --target aarch64-apple-ios  # iOS
   cargo build --release --features ffi --target aarch64-apple-darwin  # macOS ARM
   cargo build --release --features ffi --target x86_64-apple-darwin  # macOS Intel
   ```

2. **Create a module map** (`module.modulemap`):
   ```
   module ELID {
       header "elid.h"
       export *
   }
   ```

3. **Add to Xcode:**
   - Add `elid.h` to your project
   - Add `libelid.dylib` to "Link Binary With Libraries"
   - Set the module map in Build Settings

4. **Use in Swift:**
   ```swift
   import ELID

   let distance = elid_levenshtein("hello", "world")
   ```

### Swift Package Manager

You can create a Swift package that wraps the C library:

```swift
// Package.swift
let package = Package(
    name: "ELID",
    products: [
        .library(name: "ELID", targets: ["ELID"]),
    ],
    targets: [
        .systemLibrary(
            name: "CELID",
            pkgConfig: "elid"
        ),
        .target(
            name: "ELID",
            dependencies: ["CELID"]
        ),
    ]
)
```

## Testing

Run the included C test:

```bash
# Build the library
cargo build --release --features ffi

# Compile the test
gcc test-c-ffi.c -I. -L./target/release -lelid -o test-c-ffi

# Run the test
LD_LIBRARY_PATH=./target/release ./test-c-ffi
```

Expected output:
```
✅ ALL 10 C FFI TESTS PASSED!
```

## Examples

See the `examples/` directory for complete examples:

- **C**: `examples/c/basic_usage.c`
- **C++**: `examples/cpp/string_similarity.cpp`
- **Swift**: `examples/swift/ElidExample.swift`
- **Go**: `examples/go/main.go`

## Troubleshooting

### Library not found

**Linux:**
```bash
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
# Or add to /etc/ld.so.conf and run ldconfig
```

**macOS:**
```bash
export DYLD_LIBRARY_PATH=/usr/local/lib:$DYLD_LIBRARY_PATH
```

**Windows:**
Add the library directory to your PATH.

### Undefined symbols

Make sure you're linking against the library:
```bash
gcc myapp.c -lelid  # Add -lelid flag
```

### UTF-8 Encoding Issues

All strings must be UTF-8 encoded. Most modern systems use UTF-8 by default, but if you encounter issues:

**C:**
```c
#include <locale.h>
setlocale(LC_ALL, "en_US.UTF-8");
```

**Swift:**
```swift
let cString = myString.cString(using: .utf8)
```

## License

MIT OR Apache-2.0

## See Also

- [Main README](README.md) - Rust usage
- [Python README](PYTHON_README.md) - Python bindings
- [WASM README](WASM_README.md) - JavaScript/WASM bindings
- [Production Guide](PRODUCTION.md) - Deployment best practices
