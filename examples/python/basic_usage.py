"""
Basic usage examples for ELID Python bindings.

Run with: python basic_usage.py

First, install the package:
  maturin develop
"""

import elid

print("=== ELID Python Example ===\n")

# Example 1: Levenshtein Distance
print("1. Levenshtein Distance:")
distance = elid.levenshtein("kitten", "sitting")
print(f"   Distance between 'kitten' and 'sitting': {distance}")

similarity = elid.normalized_levenshtein("kitten", "sitting")
print(f"   Normalized similarity: {similarity:.2f}\n")

# Example 2: Jaro-Winkler Similarity
print("2. Jaro-Winkler Similarity (best for names):")
names = [
    ("Martha", "Marhta"),
    ("John Smith", "Jon Smith"),
    ("DIXON", "DICKSON"),
]

for name1, name2 in names:
    jaro_sim = elid.jaro(name1, name2)
    jw_sim = elid.jaro_winkler(name1, name2)
    print(f"   '{name1}' vs '{name2}':")
    print(f"     Jaro: {jaro_sim:.3f}, Jaro-Winkler: {jw_sim:.3f}")
print()

# Example 3: Finding best match
print("3. Finding Best Match:")
candidates = ["apple", "application", "apply", "apricot", "banana"]
query = "app"

result = elid.find_best_match(query, candidates)
print(f"   Query: '{query}'")
print(
    f"   Best match: '{candidates[result['index']]}' (score: {result['score']:.3f})\n"
)

# Example 4: Finding all matches above threshold
print("4. Finding All Matches Above Threshold:")
matches = elid.find_matches_above_threshold(query, candidates, 0.5)
print(f"   Query: '{query}', Threshold: 0.5")
print("   Matches:")
for match in matches:
    print(f"     - '{candidates[match['index']]}' (score: {match['score']:.3f})")
print()

# Example 5: Product Search Simulation
print("5. Product Search Simulation:")
products = [
    "iPhone 14 Pro Max",
    "iPhone 14 Pro",
    "iPhone 14",
    "iPhone 13 Pro",
    "Samsung Galaxy S23",
    "Google Pixel 7",
]

searches = ["iphone 14 pro", "galaxy", "pixel"]

for search in searches:
    # Score all products
    scored = [
        {
            "index": i,
            "product": product,
            "score": elid.best_match(search.lower(), product.lower()),
        }
        for i, product in enumerate(products)
    ]

    # Sort by score descending
    scored.sort(key=lambda x: x["score"], reverse=True)

    print(f"   Search: '{search}'")
    print("   Top 3 matches:")
    for match in scored[:3]:
        print(f"     - {match['product']} (score: {match['score']:.3f})")
    print()

# Example 6: Using SimilarityOpts
print("6. Using SimilarityOpts (case-insensitive, trimmed):")
opts = elid.SimilarityOpts(case_sensitive=False, trim_whitespace=True)

pairs = [
    ("  HELLO  ", "hello"),
    ("World", "WORLD"),
    ("\tTest\n", "test"),
]

for str1, str2 in pairs:
    dist = elid.levenshtein_with_opts(str1, str2, opts)
    print(f"   '{str1}' vs '{str2}': distance = {dist}")
print()

# Example 7: Hamming Distance
print("7. Hamming Distance (equal-length strings):")
hamming_pairs = [
    ("ACGTACGT", "ACGTACCT"),
    ("hello", "hallo"),
]

for seq1, seq2 in hamming_pairs:
    dist = elid.hamming(seq1, seq2)
    if dist is not None:
        print(f"   '{seq1}' vs '{seq2}': {dist} differences")
print()

# Example 8: OSA Distance (with transpositions)
print("8. OSA Distance (handles transpositions):")
osa_pairs = [
    ("ca", "ac"),
    ("abcd", "acbd"),
]

for str1, str2 in osa_pairs:
    osa = elid.osa_distance(str1, str2)
    lev = elid.levenshtein(str1, str2)
    print(f"   '{str1}' vs '{str2}':")
    print(f"     OSA: {osa}, Levenshtein: {lev}")
print()

# Example 9: Spell Checker
print("9. Simple Spell Checker:")
dictionary = ["receive", "believe", "achieve", "receive", "ceiling", "deceive"]
misspellings = ["recieve", "beleive", "achive", "cieling"]

for misspelling in misspellings:
    result = elid.find_best_match(misspelling, dictionary)
    suggestion = dictionary[result["index"]]
    print(f"   '{misspelling}' → '{suggestion}' (confidence: {result['score']:.2%})")
print()

# Example 10: Name Deduplication
print("10. Name Deduplication:")
names = [
    ("John Smith", "Jon Smith"),
    ("Robert Johnson", "Bob Johnson"),
    ("Mary Williams", "Marie Williams"),
    ("James Brown", "Jane Brown"),
]

threshold = 0.85
for name1, name2 in names:
    similarity = elid.jaro_winkler(name1, name2)
    is_duplicate = similarity > threshold
    status = "✓ Likely duplicate" if is_duplicate else "✗ Different people"
    print(f"   '{name1}' vs '{name2}': {similarity:.3f} - {status}")

print("\n=== End of Examples ===")
