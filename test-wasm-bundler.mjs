#!/usr/bin/env node
/**
 * Test script for ELID WASM Bundler target
 * Uses ES modules (import syntax)
 */

import * as elid from './pkg/elid.js';

console.log("Testing ELID WASM Bundler Target...\n");

let passed = 0;
let failed = 0;

// Test 1: Levenshtein
console.log("1. Levenshtein Distance:");
const dist = elid.levenshtein("kitten", "sitting");
console.log(`   levenshtein("kitten", "sitting") = ${dist}`);
if (dist === 3) {
    console.log("   ✓ PASS\n");
    passed++;
} else {
    console.log("   ✗ FAIL\n");
    failed++;
}

// Test 2: Normalized Levenshtein
console.log("2. Normalized Levenshtein:");
const norm = elid.normalizedLevenshtein("hello", "hello");
console.log(`   normalizedLevenshtein("hello", "hello") = ${norm}`);
if (norm === 1.0) {
    console.log("   ✓ PASS\n");
    passed++;
} else {
    console.log("   ✗ FAIL\n");
    failed++;
}

// Test 3: Jaro-Winkler
console.log("3. Jaro-Winkler:");
const jw = elid.jaroWinkler("martha", "marhta");
console.log(`   jaroWinkler("martha", "marhta") = ${jw.toFixed(3)}`);
if (jw > 0.9) {
    console.log("   ✓ PASS\n");
    passed++;
} else {
    console.log("   ✗ FAIL\n");
    failed++;
}

// Test 4: SimHash
console.log("4. SimHash:");
const hash1 = elid.simhash("iPhone 14");
const hash2 = elid.simhash("iPhone 15");
const hash3 = elid.simhash("Galaxy S23");
console.log(`   simhash("iPhone 14") = ${hash1}`);
console.log(`   simhash("iPhone 15") = ${hash2}`);
console.log(`   simhash("Galaxy S23") = ${hash3}`);
const dist1 = elid.simhashDistance(hash1, hash2);
const dist2 = elid.simhashDistance(hash1, hash3);
console.log(`   Distance iPhone 14 vs 15: ${dist1}`);
console.log(`   Distance iPhone 14 vs Galaxy: ${dist2}`);
if (dist1 < dist2) {
    console.log("   ✓ PASS\n");
    passed++;
} else {
    console.log("   ✗ FAIL\n");
    failed++;
}

// Test 5: Find Best Match
console.log("5. Find Best Match:");
const candidates = ["apple", "application", "apply"];
const result = elid.findBestMatch("app", candidates);
console.log(`   findBestMatch("app", [...]) = ${JSON.stringify(result)}`);
if (result.index >= 0 && result.score > 0.5) {
    console.log("   ✓ PASS\n");
    passed++;
} else {
    console.log("   ✗ FAIL\n");
    failed++;
}

// Test 6: Hamming
console.log("6. Hamming Distance:");
const ham = elid.hamming("karolin", "kathrin");
console.log(`   hamming("karolin", "kathrin") = ${ham}`);
if (ham === 3) {
    console.log("   ✓ PASS\n");
    passed++;
} else {
    console.log("   ✗ FAIL\n");
    failed++;
}

// Test 7: OSA Distance
console.log("7. OSA Distance:");
const osa = elid.osaDistance("ca", "ac");
console.log(`   osaDistance("ca", "ac") = ${osa}`);
if (osa === 1) {
    console.log("   ✓ PASS\n");
    passed++;
} else {
    console.log("   ✗ FAIL\n");
    failed++;
}

console.log("=".repeat(40));
if (failed === 0) {
    console.log(`✅ ALL ${passed} BUNDLER TESTS PASSED!`);
} else {
    console.log(`⚠️  ${passed} passed, ${failed} failed`);
    process.exit(1);
}
console.log("=".repeat(40));
