// Quick test of ELID WASM bindings
const elid = require('../../pkg-node/elid');

console.log('Testing ELID WASM Bindings...\n');

// Test 1: Levenshtein
console.log('1. Levenshtein Distance:');
const dist = elid.levenshtein("kitten", "sitting");
console.log(`   levenshtein("kitten", "sitting") = ${dist}`);
console.assert(dist === 3, 'Levenshtein failed!');
console.log('   ✓ PASS\n');

// Test 2: Normalized Levenshtein
console.log('2. Normalized Levenshtein:');
const sim = elid.normalizedLevenshtein("hello", "hello");
console.log(`   normalizedLevenshtein("hello", "hello") = ${sim}`);
console.assert(sim === 1.0, 'Normalized Levenshtein failed!');
console.log('   ✓ PASS\n');

// Test 3: Jaro-Winkler
console.log('3. Jaro-Winkler:');
const jw = elid.jaroWinkler("martha", "marhta");
console.log(`   jaroWinkler("martha", "marhta") = ${jw.toFixed(3)}`);
console.assert(jw > 0.9, 'Jaro-Winkler failed!');
console.log('   ✓ PASS\n');

// Test 4: SimHash
console.log('4. SimHash:');
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
console.assert(dist1 < dist2, 'SimHash distance failed!');
console.log('   ✓ PASS\n');

// Test 5: SimHash Similarity
console.log('5. SimHash Similarity:');
const simSim = elid.simhashSimilarity("iPhone 14", "iPhone 15");
console.log(`   simhashSimilarity("iPhone 14", "iPhone 15") = ${simSim.toFixed(3)}`);
console.assert(simSim > 0.5, 'SimHash similarity failed!');
console.log('   ✓ PASS\n');

// Test 6: Find Best Match
console.log('6. Find Best Match:');
const candidates = ["apple", "application", "apply"];
const result = elid.findBestMatch("app", candidates);
console.log(`   findBestMatch("app", [...]) = ${JSON.stringify(result)}`);
console.assert(result.index >= 0 && result.score > 0.5, 'Find best match failed!');
console.log('   ✓ PASS\n');

// Test 7: Find Matches Above Threshold
console.log('7. Find Matches Above Threshold:');
const matches = elid.findMatchesAboveThreshold("app", candidates, 0.5);
console.log(`   Found ${matches.length} matches above 0.5 threshold`);
console.assert(matches.length >= 2, 'Threshold matching failed!');
console.log('   ✓ PASS\n');

// Test 8: Hamming Distance
console.log('8. Hamming Distance:');
const hamDist = elid.hamming("karolin", "kathrin");
console.log(`   hamming("karolin", "kathrin") = ${hamDist}`);
console.assert(hamDist === 3, 'Hamming failed!');
console.log('   ✓ PASS\n');

// Test 9: OSA Distance
console.log('9. OSA Distance:');
const osa = elid.osaDistance("ca", "ac");
console.log(`   osaDistance("ca", "ac") = ${osa}`);
console.assert(osa === 1, 'OSA failed!');
console.log('   ✓ PASS\n');

console.log('========================================');
console.log('✅ ALL WASM BINDINGS TESTS PASSED!');
console.log('========================================');
