#!/usr/bin/env python3
"""Test script for ELID Python bindings"""

import elid

print("Testing ELID Python Bindings...\n")

# Test 1: Levenshtein
print("1. Levenshtein Distance:")
dist = elid.levenshtein("kitten", "sitting")
print(f"   levenshtein('kitten', 'sitting') = {dist}")
assert dist == 3, "Levenshtein failed!"
print("   ✓ PASS\n")

# Test 2: Normalized Levenshtein
print("2. Normalized Levenshtein:")
sim = elid.normalized_levenshtein("hello", "hello")
print(f"   normalized_levenshtein('hello', 'hello') = {sim}")
assert sim == 1.0, "Normalized Levenshtein failed!"
print("   ✓ PASS\n")

# Test 3: Jaro-Winkler
print("3. Jaro-Winkler:")
jw = elid.jaro_winkler("martha", "marhta")
print(f"   jaro_winkler('martha', 'marhta') = {jw:.3f}")
assert jw > 0.9, "Jaro-Winkler failed!"
print("   ✓ PASS\n")

# Test 4: SimHash
print("4. SimHash:")
hash1 = elid.simhash("iPhone 14")
hash2 = elid.simhash("iPhone 15")
hash3 = elid.simhash("Galaxy S23")
print(f"   simhash('iPhone 14') = {hash1}")
print(f"   simhash('iPhone 15') = {hash2}")
print(f"   simhash('Galaxy S23') = {hash3}")

dist1 = elid.simhash_distance(hash1, hash2)
dist2 = elid.simhash_distance(hash1, hash3)
print(f"   Distance iPhone 14 vs 15: {dist1}")
print(f"   Distance iPhone 14 vs Galaxy: {dist2}")
assert dist1 < dist2, "SimHash distance failed!"
print("   ✓ PASS\n")

# Test 5: SimHash Similarity
print("5. SimHash Similarity:")
sim_sim = elid.simhash_similarity("iPhone 14", "iPhone 15")
print(f"   simhash_similarity('iPhone 14', 'iPhone 15') = {sim_sim:.3f}")
assert sim_sim > 0.5, "SimHash similarity failed!"
print("   ✓ PASS\n")

# Test 6: Find Best Match
print("6. Find Best Match:")
candidates = ["apple", "application", "apply"]
result = elid.find_best_match("app", candidates)
print(f"   find_best_match('app', [...]) = {result}")
assert result["index"] >= 0 and result["score"] > 0.5, "Find best match failed!"
print("   ✓ PASS\n")

# Test 7: Find Matches Above Threshold
print("7. Find Matches Above Threshold:")
matches = elid.find_matches_above_threshold("app", candidates, 0.5)
print(f"   Found {len(matches)} matches above 0.5 threshold")
print(f"   Matches: {matches}")
assert len(matches) >= 2, "Threshold matching failed!"
print("   ✓ PASS\n")

# Test 8: Hamming Distance
print("8. Hamming Distance:")
ham_dist = elid.hamming("karolin", "kathrin")
print(f"   hamming('karolin', 'kathrin') = {ham_dist}")
assert ham_dist == 3, "Hamming failed!"
print("   ✓ PASS\n")

# Test 9: OSA Distance
print("9. OSA Distance:")
osa = elid.osa_distance("ca", "ac")
print(f"   osa_distance('ca', 'ac') = {osa}")
assert osa == 1, "OSA failed!"
print("   ✓ PASS\n")

# Test 10: Best Match
print("10. Best Match:")
best = elid.best_match("hello", "hallo")
print(f"   best_match('hello', 'hallo') = {best:.3f}")
assert best > 0.7, "Best match failed!"
print("   ✓ PASS\n")

# Test 11: SimilarityOpts
print("11. SimilarityOpts:")
opts = elid.SimilarityOpts(case_sensitive=False, trim_whitespace=True)
print(f"   Created opts: {opts}")
dist_opts = elid.levenshtein_with_opts("  HELLO  ", "hello", opts)
print(f"   levenshtein_with_opts('  HELLO  ', 'hello', opts) = {dist_opts}")
assert dist_opts == 0, "Options failed!"
print("   ✓ PASS\n")

# Test 12: Find Similar Hashes
print("12. Find Similar Hashes:")
candidate_strings = ["iPhone 14 Pro", "iPhone 13", "Galaxy S23"]
hashes = [elid.simhash(s) for s in candidate_strings]
query_hash = elid.simhash("iPhone 14")
matches_hash = elid.find_similar_hashes(query_hash, hashes, 15)
print(f"   find_similar_hashes(query_hash, hashes, 15) = {matches_hash}")
assert len(matches_hash) >= 1, "Find similar hashes failed!"
print("   ✓ PASS\n")

# Test 13: Jaro
print("13. Jaro:")
jaro_val = elid.jaro("martha", "marhta")
print(f"   jaro('martha', 'marhta') = {jaro_val:.3f}")
assert jaro_val > 0.9, "Jaro failed!"
print("   ✓ PASS\n")

print("=" * 40)
print("✅ ALL PYTHON BINDINGS TESTS PASSED!")
print("=" * 40)
