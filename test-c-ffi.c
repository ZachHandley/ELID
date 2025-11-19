#include <stdio.h>
#include <stdint.h>
#include "elid.h"

int main(void) {
    printf("Testing ELID C FFI Bindings...\n\n");

    int passed = 0;
    int failed = 0;

    // Test 1: Levenshtein Distance
    printf("1. Levenshtein Distance:\n");
    uintptr_t dist = elid_levenshtein("kitten", "sitting");
    printf("   elid_levenshtein(\"kitten\", \"sitting\") = %zu\n", dist);
    if (dist == 3) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (expected 3)\n\n");
        failed++;
    }

    // Test 2: Normalized Levenshtein
    printf("2. Normalized Levenshtein:\n");
    double norm = elid_normalized_levenshtein("hello", "hello");
    printf("   elid_normalized_levenshtein(\"hello\", \"hello\") = %.3f\n", norm);
    if (norm == 1.0) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (expected 1.0)\n\n");
        failed++;
    }

    // Test 3: Jaro-Winkler
    printf("3. Jaro-Winkler:\n");
    double jw = elid_jaro_winkler("martha", "marhta");
    printf("   elid_jaro_winkler(\"martha\", \"marhta\") = %.3f\n", jw);
    if (jw > 0.9) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (expected > 0.9)\n\n");
        failed++;
    }

    // Test 4: Hamming Distance
    printf("4. Hamming Distance:\n");
    int64_t ham = elid_hamming("karolin", "kathrin");
    printf("   elid_hamming(\"karolin\", \"kathrin\") = %lld\n", (long long)ham);
    if (ham == 3) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (expected 3)\n\n");
        failed++;
    }

    // Test 5: OSA Distance
    printf("5. OSA Distance:\n");
    uintptr_t osa = elid_osa_distance("ca", "ac");
    printf("   elid_osa_distance(\"ca\", \"ac\") = %zu\n", osa);
    if (osa == 1) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (expected 1)\n\n");
        failed++;
    }

    // Test 6: Best Match
    printf("6. Best Match:\n");
    double best = elid_best_match("hello", "hallo");
    printf("   elid_best_match(\"hello\", \"hallo\") = %.3f\n", best);
    if (best > 0.7) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (expected > 0.7)\n\n");
        failed++;
    }

    // Test 7: SimHash
    printf("7. SimHash:\n");
    uint64_t hash1 = elid_simhash("iPhone 14");
    uint64_t hash2 = elid_simhash("iPhone 15");
    uint64_t hash3 = elid_simhash("Galaxy S23");
    printf("   elid_simhash(\"iPhone 14\") = %llu\n", (unsigned long long)hash1);
    printf("   elid_simhash(\"iPhone 15\") = %llu\n", (unsigned long long)hash2);
    printf("   elid_simhash(\"Galaxy S23\") = %llu\n", (unsigned long long)hash3);

    uint32_t dist1 = elid_simhash_distance(hash1, hash2);
    uint32_t dist2 = elid_simhash_distance(hash1, hash3);
    printf("   Distance iPhone 14 vs 15: %u\n", dist1);
    printf("   Distance iPhone 14 vs Galaxy: %u\n", dist2);

    if (dist1 < dist2) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (expected dist1 < dist2)\n\n");
        failed++;
    }

    // Test 8: SimHash Similarity
    printf("8. SimHash Similarity:\n");
    double sim = elid_simhash_similarity("iPhone 14", "iPhone 15");
    printf("   elid_simhash_similarity(\"iPhone 14\", \"iPhone 15\") = %.3f\n", sim);
    if (sim > 0.5) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (expected > 0.5)\n\n");
        failed++;
    }

    // Test 9: NULL Safety
    printf("9. NULL Pointer Safety:\n");
    uintptr_t null_dist = elid_levenshtein(NULL, NULL);
    printf("   elid_levenshtein(NULL, NULL) = %zu\n", null_dist);
    if (null_dist == 0) {
        printf("   ✓ PASS (safe NULL handling)\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (should return 0)\n\n");
        failed++;
    }

    // Test 10: Version
    printf("10. Library Version:\n");
    const char *version = elid_version();
    printf("   elid_version() = %s\n", version);
    if (version != NULL) {
        printf("   ✓ PASS\n\n");
        passed++;
    } else {
        printf("   ✗ FAIL (version is NULL)\n\n");
        failed++;
    }

    // Summary
    printf("========================================\n");
    if (failed == 0) {
        printf("✅ ALL %d C FFI TESTS PASSED!\n", passed);
    } else {
        printf("⚠️  %d passed, %d failed\n", passed, failed);
        return 1;
    }
    printf("========================================\n");

    return 0;
}
