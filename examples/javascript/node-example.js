// Node.js example using ELID WASM bindings
// Run with: node node-example.js
//
// First, build the WASM package:
// npm run build:node

const elid = require('../../pkg-node/elid');

console.log('=== ELID WASM Node.js Example ===\n');

// Example 1: Levenshtein Distance
console.log('1. Levenshtein Distance:');
const distance = elid.levenshtein("kitten", "sitting");
console.log(`   Distance between 'kitten' and 'sitting': ${distance}`);

const similarity = elid.normalizedLevenshtein("kitten", "sitting");
console.log(`   Normalized similarity: ${similarity.toFixed(2)}\n`);

// Example 2: Jaro-Winkler Similarity
console.log('2. Jaro-Winkler Similarity (best for names):');
const names = [
    ["Martha", "Marhta"],
    ["John Smith", "Jon Smith"],
    ["DIXON", "DICKSON"],
];

for (const [name1, name2] of names) {
    const jaroSim = elid.jaro(name1, name2);
    const jwSim = elid.jaroWinkler(name1, name2);
    console.log(`   '${name1}' vs '${name2}':`);
    console.log(`     Jaro: ${jaroSim.toFixed(3)}, Jaro-Winkler: ${jwSim.toFixed(3)}`);
}
console.log();

// Example 3: Finding best match
console.log('3. Finding Best Match:');
const candidates = ["apple", "application", "apply", "apricot", "banana"];
const query = "app";

const result = elid.findBestMatch(query, candidates);
console.log(`   Query: '${query}'`);
console.log(`   Best match: '${candidates[result.index]}' (score: ${result.score.toFixed(3)})\n`);

// Example 4: Finding all matches above threshold
console.log('4. Finding All Matches Above Threshold:');
const matches = elid.findMatchesAboveThreshold(query, candidates, 0.5);
console.log(`   Query: '${query}', Threshold: 0.5`);
console.log('   Matches:');
for (const match of matches) {
    console.log(`     - '${candidates[match.index]}' (score: ${match.score.toFixed(3)})`);
}
console.log();

// Example 5: Product Search Simulation
console.log('5. Product Search Simulation:');
const products = [
    "iPhone 14 Pro Max",
    "iPhone 14 Pro",
    "iPhone 14",
    "iPhone 13 Pro",
    "Samsung Galaxy S23",
    "Google Pixel 7",
];

const searches = ["iphone 14 pro", "galaxy", "pixel"];

for (const search of searches) {
    // Score all products
    const scored = products.map((product, i) => ({
        index: i,
        score: elid.bestMatch(search.toLowerCase(), product.toLowerCase())
    }));

    // Sort by score descending
    scored.sort((a, b) => b.score - a.score);

    console.log(`   Search: '${search}'`);
    console.log('   Top 3 matches:');
    for (const match of scored.slice(0, 3)) {
        console.log(`     - ${products[match.index]} (score: ${match.score.toFixed(3)})`);
    }
    console.log();
}

// Example 6: Using SimilarityOptions
console.log('6. Using SimilarityOptions (case-insensitive, trimmed):');
const opts = new elid.SimilarityOptions();
opts.setCaseSensitive(false);
opts.setTrimWhitespace(true);

const pairs = [
    ["  HELLO  ", "hello"],
    ["World", "WORLD"],
    ["\tTest\n", "test"],
];

for (const [str1, str2] of pairs) {
    const dist = elid.levenshteinWithOpts(str1, str2, opts);
    console.log(`   '${str1}' vs '${str2}': distance = ${dist}`);
}
console.log();

// Example 7: Hamming Distance
console.log('7. Hamming Distance (equal-length strings):');
const hammingPairs = [
    ["ACGTACGT", "ACGTACCT"],
    ["hello", "hallo"],
];

for (const [seq1, seq2] of hammingPairs) {
    const dist = elid.hamming(seq1, seq2);
    if (dist !== null) {
        console.log(`   '${seq1}' vs '${seq2}': ${dist} differences`);
    }
}
console.log();

// Example 8: OSA Distance (with transpositions)
console.log('8. OSA Distance (handles transpositions):');
const osaPairs = [
    ["ca", "ac"],
    ["abcd", "acbd"],
];

for (const [str1, str2] of osaPairs) {
    const osa = elid.osaDistance(str1, str2);
    const lev = elid.levenshtein(str1, str2);
    console.log(`   '${str1}' vs '${str2}':`);
    console.log(`     OSA: ${osa}, Levenshtein: ${lev}`);
}

console.log('\n=== End of Examples ===');
