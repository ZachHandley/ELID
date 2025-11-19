#!/usr/bin/env python3
"""
Live Demo: ELID String Similarity Library
Real-world use cases across all platforms
"""

import elid

print("=" * 60)
print("  ELID - String Similarity Library Live Demo")
print("=" * 60)
print()

# Demo 1: Spell Checking / Typo Correction
print("📝 Demo 1: Spell Checking / Typo Correction")
print("-" * 60)
dictionary = ["programming", "algorithm", "database", "python", "javascript"]
typo = "progamming"

print(f"User typed: '{typo}'")
print(f"Checking against dictionary: {dictionary}\n")

for word in dictionary:
    similarity = elid.normalized_levenshtein(typo, word)
    if similarity > 0.7:
        print(f"  ✓ '{word}' - {similarity:.1%} match (suggested)")
    else:
        print(f"    '{word}' - {similarity:.1%} match")

best_match = elid.find_best_match(typo, dictionary)
print(f"\n💡 Best suggestion: '{dictionary[best_match['index']]}' ({best_match['score']:.1%})\n")

# Demo 2: Name Matching (Jaro-Winkler)
print("👤 Demo 2: Name Matching (Great for Names!)")
print("-" * 60)
database_names = ["Martha Stewart", "Martin Smith", "Mary Johnson"]
search_query = "Marta"

print(f"Searching for: '{search_query}'")
print(f"In database: {database_names}\n")

for name in database_names:
    jw_score = elid.jaro_winkler(search_query.lower(), name.split()[0].lower())
    if jw_score > 0.8:
        print(f"  ✓ {name} - {jw_score:.1%} match (Jaro-Winkler)")
    else:
        print(f"    {name} - {jw_score:.1%} match")
print()

# Demo 3: Product Search (SimHash for Database)
print("🛍️  Demo 3: Product Search with SimHash")
print("-" * 60)
products = [
    "Apple iPhone 14 Pro Max 256GB",
    "Apple iPhone 14 Pro 128GB",
    "Apple iPhone 13 Pro Max",
    "Samsung Galaxy S23 Ultra",
    "Samsung Galaxy S22",
    "Google Pixel 7 Pro"
]

print("Indexing products with SimHash...")
product_hashes = [(p, elid.simhash(p)) for p in products]
print(f"✓ Indexed {len(products)} products\n")

search = "iPhone 14"
search_hash = elid.simhash(search)

print(f"Searching for: '{search}'")
print(f"Query SimHash: {search_hash}\n")
print("Results (sorted by similarity):\n")

results = []
for product, hash_val in product_hashes:
    distance = elid.simhash_distance(search_hash, hash_val)
    similarity = 1.0 - (distance / 64.0)  # Convert Hamming distance to similarity
    results.append((product, distance, similarity))

results.sort(key=lambda x: x[1])  # Sort by distance (lower = more similar)

for i, (product, distance, similarity) in enumerate(results[:5], 1):
    if similarity > 0.5:
        print(f"  {i}. ✓ {product}")
        print(f"     Distance: {distance} bits | Similarity: {similarity:.1%}")
    else:
        print(f"  {i}.   {product}")
        print(f"     Distance: {distance} bits | Similarity: {similarity:.1%}")
print()

# Demo 4: Deduplication
print("🔍 Demo 4: Duplicate Detection")
print("-" * 60)
user_entries = [
    "New York City",
    "New York, NY",
    "NYC",
    "Los Angeles",
    "LA",
    "San Francisco"
]

print("Finding duplicate entries...")
print()

threshold = 0.7
found_duplicates = False

for i, entry1 in enumerate(user_entries):
    for j, entry2 in enumerate(user_entries[i+1:], i+1):
        similarity = elid.best_match(entry1.lower(), entry2.lower())
        if similarity > threshold:
            found_duplicates = True
            print(f"  ⚠️  Possible duplicate:")
            print(f"     '{entry1}' ≈ '{entry2}' ({similarity:.1%} similar)")

if not found_duplicates:
    print("  ✓ No duplicates found above threshold\n")
else:
    print()

# Demo 5: DNA/Genome Sequence Similarity
print("🧬 Demo 5: DNA Sequence Similarity")
print("-" * 60)
sequence1 = "ACGTACGT"
sequence2 = "ACGTACGA"  # One mutation
sequence3 = "ACGGACGT"  # One transposition

print(f"Reference: {sequence1}")
print(f"Sample 1:  {sequence2}")
print(f"Sample 2:  {sequence3}\n")

lev_dist1 = elid.levenshtein(sequence1, sequence2)
osa_dist1 = elid.osa_distance(sequence1, sequence2)

lev_dist2 = elid.levenshtein(sequence1, sequence3)
osa_dist2 = elid.osa_distance(sequence1, sequence3)

print(f"Sample 1 vs Reference:")
print(f"  Levenshtein: {lev_dist1} edits | OSA: {osa_dist1} operations")
print(f"\nSample 2 vs Reference:")
print(f"  Levenshtein: {lev_dist2} edits | OSA: {osa_dist2} operations")
print(f"  (OSA counts transposition as 1 operation!)")
print()

# Performance Stats
print("⚡ Demo 6: Performance Test")
print("-" * 60)
import time

iterations = 10000
start = time.time()
for _ in range(iterations):
    elid.levenshtein("hello", "world")
elapsed = time.time() - start

print(f"Computed {iterations:,} Levenshtein distances")
print(f"Total time: {elapsed:.3f} seconds")
print(f"Average: {(elapsed/iterations)*1000:.4f} ms per operation")
print(f"Throughput: {iterations/elapsed:,.0f} operations/second")
print()

print("=" * 60)
print("  ✨ All Demos Completed Successfully!")
print("=" * 60)
print()
print("Supported Languages:")
print("  ✅ Rust (native)")
print("  ✅ JavaScript/TypeScript (WASM)")
print("  ✅ Python (PyO3)")
print("  ✅ C/C++ (FFI)")
print("  ✅ Swift (FFI)")
print("  ✅ Go (CGO)")
print("  ✅ Ruby (FFI)")
print()
print("For more info: https://github.com/ZachHandley/ELID")
print()
