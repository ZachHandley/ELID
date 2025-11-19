# ELID WebAssembly Bindings

Fast string similarity search for JavaScript, powered by Rust and WebAssembly.

## Features

- 🚀 **Blazing Fast** - Compiled to WebAssembly from optimized Rust code
- 🌐 **Universal** - Works in browsers, Node.js, Deno, and Bun
- 📦 **Tiny Bundle** - Minimal WASM footprint
- 🎯 **Type-Safe** - Full TypeScript definitions included
- ⚡ **Query-Time Ready** - Fast enough for real-time search as you type

## Installation

### NPM/Yarn/PNPM

```bash
npm install elid-wasm
# or
yarn add elid-wasm
# or
pnpm add elid-wasm
```

## Quick Start

### Node.js

```javascript
const elid = require('elid-wasm');

// Find best match
const candidates = ["apple", "application", "apply"];
const result = elid.findBestMatch("app", candidates);
console.log(result); // { index: 0, score: 0.907 }

// Levenshtein distance
const distance = elid.levenshtein("kitten", "sitting");
console.log(distance); // 3

// Normalized similarity (0.0 to 1.0)
const similarity = elid.normalizedLevenshtein("hello", "hallo");
console.log(similarity); // 0.8
```

### Browser (ES Modules)

```html
<script type="module">
    import init, * as elid from './pkg/elid.js';

    await init(); // Initialize WASM

    const similarity = elid.jaroWinkler("Martha", "Marhta");
    console.log(similarity); // 0.961
</script>
```

### TypeScript

```typescript
import * as elid from 'elid-wasm';

const candidates: string[] = ["apple", "application", "apply"];
const result = elid.findBestMatch("app", candidates);

console.log(`Best match: ${candidates[result.index]}`);
console.log(`Score: ${result.score}`);
```

## Building from Source

### Prerequisites

- Rust (latest stable)
- wasm-pack: `cargo install wasm-pack`
- Node.js (for testing)

### Build Commands

```bash
# Build for bundlers (webpack, vite, etc.)
npm run build

# Build for Node.js
npm run build:node

# Build for browsers (as ES module)
npm run build:web

# Build all targets
npm run build:all

# Run tests
npm test
```

## API Reference

### Distance Metrics

#### `levenshtein(a: string, b: string): number`

Compute the Levenshtein distance between two strings.

```javascript
elid.levenshtein("kitten", "sitting"); // 3
```

#### `normalizedLevenshtein(a: string, b: string): number`

Returns normalized similarity (0.0 to 1.0).

```javascript
elid.normalizedLevenshtein("hello", "hallo"); // 0.8
```

#### `jaro(a: string, b: string): number`

Compute Jaro similarity (great for short strings).

```javascript
elid.jaro("martha", "marhta"); // 0.944
```

#### `jaroWinkler(a: string, b: string): number`

Jaro-Winkler similarity with prefix bonus.

```javascript
elid.jaroWinkler("martha", "marhta"); // 0.961
```

#### `hamming(a: string, b: string): number | null`

Hamming distance for equal-length strings. Returns `null` if lengths differ.

```javascript
elid.hamming("karolin", "kathrin"); // 3
elid.hamming("hello", "world!"); // null
```

#### `osaDistance(a: string, b: string): number`

OSA distance (considers transpositions).

```javascript
elid.osaDistance("ca", "ac"); // 1
```

### Helper Functions

#### `bestMatch(a: string, b: string): number`

Runs multiple algorithms and returns the highest score.

```javascript
elid.bestMatch("hello", "hallo"); // 0.8
```

#### `findBestMatch(query: string, candidates: string[]): { index: number, score: number }`

Find the best matching candidate.

```javascript
const result = elid.findBestMatch("app", ["apple", "application", "apply"]);
// { index: 0, score: 0.907 }
```

#### `findMatchesAboveThreshold(query: string, candidates: string[], threshold: number): Array<{ index: number, score: number }>`

Find all matches above a threshold.

```javascript
const matches = elid.findMatchesAboveThreshold("app", ["apple", "banana"], 0.5);
// [{ index: 0, score: 0.907 }]
```

### Configuration

#### `SimilarityOptions`

Configure comparison behavior.

```javascript
const opts = new elid.SimilarityOptions();
opts.setCaseSensitive(false);
opts.setTrimWhitespace(true);
opts.setPrefixScale(0.1); // For Jaro-Winkler

const distance = elid.levenshteinWithOpts("  HELLO  ", "hello", opts);
// 0
```

## Use Cases

### Product Search

```javascript
const products = ["iPhone 14 Pro", "Samsung Galaxy S23", "Google Pixel 7"];
const query = "iphone";

const scored = products.map((product, i) => ({
    product,
    score: elid.bestMatch(query.toLowerCase(), product.toLowerCase())
}));

scored.sort((a, b) => b.score - a.score);
console.log(scored[0]); // Best match
```

### Spell Checking

```javascript
function suggestCorrection(word, dictionary) {
    const result = elid.findBestMatch(word, dictionary);
    return dictionary[result.index];
}

const dictionary = ["receive", "believe", "achieve"];
const suggestion = suggestCorrection("recieve", dictionary);
console.log(suggestion); // "receive"
```

### Name Deduplication

```javascript
function isDuplicateName(name1, name2) {
    return elid.jaroWinkler(name1, name2) > 0.9;
}

isDuplicateName("John Smith", "Jon Smith"); // true
```

### Real-time Search

```javascript
// As user types
searchInput.addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase();
    const matches = elid.findMatchesAboveThreshold(query, items, 0.5);
    displayResults(matches);
});
```

## Appwrite Integration

See [Appwrite Guide](./APPWRITE.md) for detailed instructions on using ELID with Appwrite Functions.

Quick example:

```javascript
// In your Appwrite Function
const elid = require('elid-wasm');

module.exports = async ({ req, res }) => {
    const { query, candidates } = JSON.parse(req.bodyRaw);
    const results = elid.findMatchesAboveThreshold(query, candidates, 0.5);

    return res.json({ results });
};
```

## Performance

ELID WASM is optimized for query-time performance:

- **Small Bundle**: < 100KB gzipped
- **Fast Initialization**: < 10ms
- **Real-time Ready**: Can handle searches on 1000s of items in milliseconds

Benchmark example (1000 products):

```javascript
const start = performance.now();
const results = elid.findMatchesAboveThreshold(query, products, 0.5);
const duration = performance.now() - start;
console.log(`Search completed in ${duration.toFixed(2)}ms`);
```

## Browser Compatibility

- Chrome/Edge 89+
- Firefox 89+
- Safari 15+
- Node.js 12+
- Deno 1.x
- Bun 1.x

## Examples

See the `examples/` directory:

- `node-example.js` - Complete Node.js usage
- `browser-example.html` - Interactive browser demo
- `appwrite/fuzzy-search-function.js` - Appwrite Function template

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Links

- [GitHub Repository](https://github.com/ZachHandley/ELID)
- [Documentation](https://github.com/ZachHandley/ELID#readme)
- [Report Issues](https://github.com/ZachHandley/ELID/issues)
