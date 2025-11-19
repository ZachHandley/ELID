#!/usr/bin/env swift

import Foundation

// Import the C header - in a real project you'd use a module map or bridging header
// For this example, we're assuming the library is linked

print("ELID Swift Example\n")

// Example 1: Levenshtein Distance
print("1. Levenshtein Distance:")
let dist = elid_levenshtein("kitten", "sitting")
print("   Distance between 'kitten' and 'sitting': \(dist)")
print("   (3 single-character edits needed)\n")

// Example 2: Normalized Similarity
print("2. Normalized Similarity:")
let similarity = elid_normalized_levenshtein("programming", "programmer")
print("   Similarity: \(String(format: "%.1f%%", similarity * 100))")
print("   (0.0 = different, 1.0 = identical)\n")

// Example 3: Jaro-Winkler (Great for names)
print("3. Jaro-Winkler (Great for names):")
let jw1 = elid_jaro_winkler("martha", "marhta")
let jw2 = elid_jaro_winkler("dwayne", "duane")
print("   'martha' vs 'marhta': \(String(format: "%.3f", jw1))")
print("   'dwayne' vs 'duane': \(String(format: "%.3f", jw2))\n")

// Example 4: SimHash for Database Queries
print("4. SimHash for Database Queries:")
let products = [
    "iPhone 14 Pro Max",
    "iPhone 14 Pro",
    "iPhone 13",
    "Samsung Galaxy S23"
]

print("   Computing hashes for product database...")
let hashes = products.map { elid_simhash($0) }

let query = "iPhone 14"
let queryHash = elid_simhash(query)
print("   Query: '\(query)' -> hash: \(queryHash)\n")

print("   Finding similar products:")
for (index, product) in products.enumerated() {
    let distance = elid_simhash_distance(queryHash, hashes[index])
    let similarity = elid_simhash_similarity(query, product)
    print("   - \(product)")
    print("     Distance: \(distance) bits, Similarity: \(String(format: "%.1f%%", similarity * 100))")
}
print()

// Example 5: Best Match Algorithm
print("5. Best Match (runs multiple algorithms):")
let candidates = ["apply", "apple", "application"]
print("   Finding best match for 'app' in: \(candidates)")
for candidate in candidates {
    let score = elid_best_match("app", candidate)
    print("   - \(candidate): \(String(format: "%.1f%%", score * 100))")
}
print()

// Example 6: Hamming Distance
print("6. Hamming Distance (equal-length strings):")
let ham1 = elid_hamming("karolin", "kathrin")
let ham2 = elid_hamming("hello", "world")
print("   'karolin' vs 'kathrin': \(ham1) positions differ")
print("   'hello' vs 'world': \(ham2) (-1 = different lengths)\n")

// Example 7: Library Version
print("7. Library Info:")
if let version = String(cString: elid_version(), encoding: .utf8) {
    print("   ELID Version: \(version)")
}

print("\n✨ All examples completed successfully!")
