#!/bin/bash
# Cross-language compatibility test for ELID bindings
#
# This script validates that all language bindings produce byte-identical
# ELID output for identical embedding input across all platforms.
#
# Usage: ./scripts/cross-language-test.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test vectors file
TEST_VECTORS="bindings/test-vectors.json"

if [ ! -f "$TEST_VECTORS" ]; then
    echo -e "${RED}Error: Test vectors not found at $TEST_VECTORS${NC}"
    echo "Run: cargo run --quiet --bin generate-test-vectors > $TEST_VECTORS"
    exit 1
fi

echo "=========================================="
echo "ELID Cross-Language Compatibility Test"
echo "=========================================="
echo ""

# Check which bindings are available
AVAILABLE_BINDINGS=()

# Rust (always available as reference)
AVAILABLE_BINDINGS+=("rust")

# Python
if command -v python3 &> /dev/null && python3 -c "import elid" 2>/dev/null; then
    AVAILABLE_BINDINGS+=("python")
fi

# TypeScript/Node.js
if command -v node &> /dev/null && node -e "require('elid')" 2>/dev/null; then
    AVAILABLE_BINDINGS+=("node")
fi

# Dart/Flutter
if command -v dart &> /dev/null && dart pub get --directory=bindings/elid-flutter &>/dev/null; then
    AVAILABLE_BINDINGS+=("dart")
fi

# Swift
if command -v swift &> /dev/null && [ -f "bindings/elid-swift/Package.swift" ]; then
    AVAILABLE_BINDINGS+=("swift")
fi

# Kotlin
if command -v kotlin &> /dev/null && [ -f "bindings/elid-kotlin/build.gradle.kts" ]; then
    AVAILABLE_BINDINGS+=("kotlin")
fi

# Ruby
if command -v ruby &> /dev/null && ruby -e "require 'elid'" 2>/dev/null; then
    AVAILABLE_BINDINGS+=("ruby")
fi

# PHP
if command -v php &> /dev/null && [ -f "bindings/elid-php/vendor/autoload.php" ]; then
    # Check if FFI is enabled and test basic loading
    if php -r "if (!extension_loaded('ffi')) exit(1); require 'bindings/elid-php/vendor/autoload.php';" 2>/dev/null; then
        AVAILABLE_BINDINGS+=("php")
    fi
fi

echo "Available bindings: ${AVAILABLE_BINDINGS[*]}"
echo ""

# Function to encode using Rust (reference implementation)
encode_rust() {
    local embedding_json="$1"
    local profile="$2"

    # Convert profile name to CLI format (lowercase)
    local cli_profile=$(echo "$profile" | tr '[:upper:]' '[:lower:]')

    # Create JSONL input with embedding field
    local jsonl_input="{\"embedding\":$embedding_json}"

    # Use elid-cli to encode via stdin/stdout
    echo "$jsonl_input" | /home/zach/github/ELID/target/release/elid encode \
        --profile "$cli_profile" \
        --format jsonl \
        --input - \
        --output-format text 2>/dev/null | head -1
}

# Function to encode using Python
encode_python() {
    local embedding_json="$1"
    local profile="$2"

    python3 -c "
import elid
import json
embedding = json.loads('$embedding_json')
profile_map = {'Mini128': elid.Profile.Mini128, 'Morton10x10': elid.Profile.Morton10x10, 'Hilbert10x10': elid.Profile.Hilbert10x10}
print(elid.encode(embedding, profile_map['$profile']))
"
}

# Function to encode using Node.js
encode_node() {
    local embedding_json="$1"
    local profile="$2"

    node -e "
const elid = require('elid');
const embedding = JSON.parse('$embedding_json');
const profileMap = {
  'Mini128': elid.Profile.Mini128,
  'Morton10x10': elid.Profile.Morton10x10,
  'Hilbert10x10': elid.Profile.Hilbert10x10
};
console.log(elid.encode(new Float32Array(embedding), profileMap['$profile']));
"
}

# Function to encode using Dart/Flutter
encode_dart() {
    local embedding_json="$1"
    local profile="$2"

    # Create a temporary Dart script
    local temp_script=$(mktemp /tmp/elid-dart-test-XXXXXX.dart)
    cat > "$temp_script" << EOF
import 'dart:convert';
import 'package:elid/elid.dart';

void main() async {
  await Elid.init();

  final embedding = (jsonDecode('$embedding_json') as List<dynamic>)
      .map((e) => (e as num).toDouble())
      .toList();

  Profile profile;
  switch ('$profile') {
    case 'Mini128':
      profile = Profile.mini128;
      break;
    case 'Morton10x10':
      profile = Profile.morton10X10;
      break;
    case 'Hilbert10x10':
      profile = Profile.hilbert10X10;
      break;
    default:
      throw Exception('Unknown profile: $profile');
  }

  final elid = Elid.encode(embedding, profile);
  print(elid);
}
EOF

    # Run the Dart script from the Flutter package directory
    (cd bindings/elid-flutter && dart run "$temp_script" 2>/dev/null)
    local result=$?
    rm -f "$temp_script"
    return $result
}

# Function to encode using Ruby
encode_ruby() {
    local embedding_json="$1"
    local profile="$2"

    ruby -r elid -r json -e "
embedding = JSON.parse('$embedding_json')
profile_map = {
  'Mini128' => Elid::Profile::MINI128,
  'Morton10x10' => Elid::Profile::MORTON10X10,
  'Hilbert10x10' => Elid::Profile::HILBERT10X10
}
puts Elid.encode(embedding, profile_map['$profile'])
"
}

# Function to encode using Kotlin
encode_kotlin() {
    local embedding_json="$1"
    local profile="$2"

    # Use kotlin REPL-like approach via kotlin -e with library path
    # We need to set LD_LIBRARY_PATH for the native library
    export LD_LIBRARY_PATH="$(pwd)/target/release:$LD_LIBRARY_PATH"

    # Get the JAR path with dependencies
    local jar_path="bindings/elid-kotlin/build/libs/elid-kotlin-0.1.0.jar"
    local jna_jar=$(find ~/.gradle/caches -name "jna-5.14.0.jar" 2>/dev/null | head -1)

    if [ ! -f "$jar_path" ]; then
        # Try to build the JAR first
        (cd bindings/elid-kotlin && ./gradlew jar -q 2>/dev/null)
    fi

    # Create a simple Kotlin runner
    kotlin -J-Djava.library.path="$(pwd)/target/release" \
           -classpath "$jar_path:$jna_jar" \
           -e "
import com.elid.*

val embeddingJson = \"\"\"$embedding_json\"\"\"
val embedding = embeddingJson
    .trim('[', ']')
    .split(',')
    .map { it.trim().toFloat() }

val profile = when (\"$profile\") {
    \"Mini128\" -> UniProfile.MINI128
    \"Morton10x10\" -> UniProfile.MORTON10X10
    \"Hilbert10x10\" -> UniProfile.HILBERT10X10
    else -> error(\"Unknown profile: $profile\")
}

val elid = uniEncode(embedding, profile)
println(elid)
" 2>/dev/null
}

# Function to encode using PHP
encode_php() {
    local embedding_json="$1"
    local profile="$2"

    # Set library path for FFI
    export ELID_LIBRARY_PATH="$(pwd)/target/release/libelid_ffi.so"

    php -d extension=ffi -r "
require 'bindings/elid-php/vendor/autoload.php';

\$embedding = json_decode('$embedding_json');
\$profileMap = [
    'Mini128' => Elid\Elid::MINI128,
    'Morton10x10' => Elid\Elid::MORTON10X10,
    'Hilbert10x10' => Elid\Elid::HILBERT10X10
];
echo Elid\Elid::encode(\$embedding, \$profileMap['$profile']) . PHP_EOL;
"
}

# TODO: Add encoding functions for other languages
# encode_swift() { ... }

# Run tests
NUM_VECTORS=$(jq '.vectors | length' "$TEST_VECTORS")
echo "Testing $NUM_VECTORS test vectors..."
echo ""

for i in $(seq 0 $((NUM_VECTORS - 1))); do
    VECTOR=$(jq ".vectors[$i]" "$TEST_VECTORS")
    EMBEDDING=$(echo "$VECTOR" | jq -c '.embedding')
    PROFILE=$(echo "$VECTOR" | jq -r '.profile' | cut -d' ' -f1)
    EXPECTED_ELID=$(echo "$VECTOR" | jq -r '.elid')
    DIMS=$(echo "$VECTOR" | jq -r '.dimensions')

    echo "Test $((i + 1))/$NUM_VECTORS: $PROFILE with ${DIMS}D embedding"

    # Test each available binding
    for binding in "${AVAILABLE_BINDINGS[@]}"; do
        TOTAL_TESTS=$((TOTAL_TESTS + 1))

        case $binding in
            rust)
                ACTUAL=$(encode_rust "$EMBEDDING" "$PROFILE" 2>/dev/null || echo "ERROR")
                ;;
            python)
                ACTUAL=$(encode_python "$EMBEDDING" "$PROFILE" 2>/dev/null || echo "ERROR")
                ;;
            node)
                ACTUAL=$(encode_node "$EMBEDDING" "$PROFILE" 2>/dev/null || echo "ERROR")
                ;;
            dart)
                ACTUAL=$(encode_dart "$EMBEDDING" "$PROFILE" 2>/dev/null || echo "ERROR")
                ;;
            ruby)
                ACTUAL=$(encode_ruby "$EMBEDDING" "$PROFILE" 2>/dev/null || echo "ERROR")
                ;;
            kotlin)
                ACTUAL=$(encode_kotlin "$EMBEDDING" "$PROFILE" 2>/dev/null || echo "ERROR")
                ;;
            php)
                ACTUAL=$(encode_php "$EMBEDDING" "$PROFILE" 2>/dev/null || echo "ERROR")
                ;;
            *)
                # Skip unimplemented bindings
                echo -e "  ${YELLOW}[$binding] SKIP: Not yet implemented${NC}"
                TOTAL_TESTS=$((TOTAL_TESTS - 1))
                continue
                ;;
        esac

        if [ "$ACTUAL" = "$EXPECTED_ELID" ]; then
            echo -e "  ${GREEN}[$binding] PASS${NC}: $ACTUAL"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        elif [ "$ACTUAL" = "ERROR" ]; then
            echo -e "  ${RED}[$binding] ERROR: Failed to encode${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        else
            echo -e "  ${RED}[$binding] FAIL${NC}"
            echo -e "    Expected: $EXPECTED_ELID"
            echo -e "    Got:      $ACTUAL"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    done

    echo ""
done

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Total tests:  $TOTAL_TESTS"
echo -e "${GREEN}Passed:       $PASSED_TESTS${NC}"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed:       $FAILED_TESTS${NC}"
else
    echo "Failed:       $FAILED_TESTS"
fi

echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All cross-language tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed. See output above for details.${NC}"
    exit 1
fi
