# Using ELID with Appwrite

This guide shows you how to add fuzzy search capabilities to your Appwrite applications using ELID.

## Overview

ELID can be integrated with Appwrite in several ways:

1. **Appwrite Functions** - Deploy as a serverless function for query-time fuzzy search
2. **Client-side** - Use WASM in your frontend for instant search
3. **Hybrid** - Combine both for optimal performance

## Option 1: Appwrite Functions (Recommended)

### Setup

1. **Build WASM for Node.js**

```bash
cd /path/to/elid
npm run build:node
```

2. **Create Appwrite Function**

```bash
appwrite init function
# Name: fuzzy-search
# Runtime: Node.js 18+
```

3. **Copy ELID WASM to your function**

```bash
cp -r pkg-node your-function/
```

4. **Add dependencies to package.json**

```json
{
  "dependencies": {
    "node-appwrite": "^12.0.0"
  }
}
```

5. **Use the example function**

Copy `examples/appwrite/fuzzy-search-function.js` to your function's `index.js`.

### Usage

#### Basic Fuzzy Search

```javascript
// POST to your function
const response = await fetch(`${endpoint}/functions/${functionId}/executions`, {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'X-Appwrite-Project': projectId,
    },
    body: JSON.stringify({
        query: "iphone 14",
        candidates: [
            "iPhone 14 Pro Max",
            "iPhone 14 Pro",
            "iPhone 13",
            "Samsung Galaxy S23"
        ],
        threshold: 0.5,
        algorithm: "best"
    })
});

const result = await response.json();
console.log(result.results);
// [
//   { index: 0, value: "iPhone 14 Pro Max", score: 0.95 },
//   { index: 1, value: "iPhone 14 Pro", score: 0.93 },
//   ...
// ]
```

#### Search Appwrite Database

Here's a complete function that searches your Appwrite database with fuzzy matching:

```javascript
const sdk = require('node-appwrite');
const elid = require('./pkg-node/elid');

module.exports = async ({ req, res, log, error }) => {
    const client = new sdk.Client()
        .setEndpoint(process.env.APPWRITE_FUNCTION_ENDPOINT)
        .setProject(process.env.APPWRITE_FUNCTION_PROJECT_ID)
        .setKey(process.env.APPWRITE_API_KEY);

    const database = new sdk.Databases(client);

    try {
        const { query, collectionId, searchField, threshold = 0.5 } = JSON.parse(req.bodyRaw);

        // Fetch all documents (or use pagination for large collections)
        const documents = await database.listDocuments(
            process.env.DATABASE_ID,
            collectionId,
            []
        );

        // Score each document
        const scored = documents.documents.map(doc => ({
            document: doc,
            score: elid.bestMatch(
                query.toLowerCase(),
                (doc[searchField] || '').toLowerCase()
            )
        }));

        // Filter and sort
        const results = scored
            .filter(item => item.score >= threshold)
            .sort((a, b) => b.score - a.score)
            .slice(0, 20); // Return top 20

        return res.json({
            success: true,
            query,
            results: results.map(r => ({
                ...r.document,
                _score: r.score
            }))
        });

    } catch (err) {
        error(err.message);
        return res.json({ success: false, error: err.message }, 500);
    }
};
```

### Environment Variables

Set these in your Appwrite Function:

```bash
DATABASE_ID=your-database-id
APPWRITE_API_KEY=your-api-key
```

## Option 2: Client-Side Integration

Use ELID WASM directly in your web app for instant, client-side fuzzy search.

### Setup

1. **Install via npm**

```bash
npm install elid-wasm
```

2. **Use in your app**

```javascript
import init, * as elid from 'elid-wasm';

async function setupSearch() {
    await init(); // Initialize WASM

    const { Client, Databases } = Appwrite;

    const client = new Client()
        .setEndpoint('https://cloud.appwrite.io/v1')
        .setProject('your-project-id');

    const database = new Databases(client);

    // Fetch products
    const products = await database.listDocuments(
        'database-id',
        'products-collection'
    );

    // Setup real-time search
    searchInput.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase();

        const results = products.documents.map(product => ({
            product,
            score: elid.bestMatch(query, product.name.toLowerCase())
        }))
        .filter(r => r.score >= 0.5)
        .sort((a, b) => b.score - a.score)
        .slice(0, 10);

        displayResults(results);
    });
}

setupSearch();
```

### React Example

```tsx
import { useState, useEffect } from 'react';
import init, * as elid from 'elid-wasm';
import { Client, Databases } from 'appwrite';

function FuzzySearch() {
    const [wasmReady, setWasmReady] = useState(false);
    const [products, setProducts] = useState([]);
    const [results, setResults] = useState([]);
    const [query, setQuery] = useState('');

    useEffect(() => {
        init().then(() => setWasmReady(true));

        const client = new Client()
            .setEndpoint(process.env.REACT_APP_APPWRITE_ENDPOINT)
            .setProject(process.env.REACT_APP_APPWRITE_PROJECT);

        const database = new Databases(client);

        database.listDocuments('db-id', 'products').then(res => {
            setProducts(res.documents);
        });
    }, []);

    useEffect(() => {
        if (!wasmReady || !query) {
            setResults([]);
            return;
        }

        const scored = products.map(product => ({
            product,
            score: elid.bestMatch(
                query.toLowerCase(),
                product.name.toLowerCase()
            )
        }))
        .filter(r => r.score >= 0.5)
        .sort((a, b) => b.score - a.score);

        setResults(scored);
    }, [query, products, wasmReady]);

    return (
        <div>
            <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search products..."
            />
            <ul>
                {results.map(({ product, score }) => (
                    <li key={product.$id}>
                        {product.name} ({(score * 100).toFixed(0)}%)
                    </li>
                ))}
            </ul>
        </div>
    );
}
```

## Option 3: Hybrid Approach

Combine client-side and server-side for best performance:

1. **Client-side** - For instant filtering of already-loaded data
2. **Server-side** - For searching across all database records

```javascript
async function search(query) {
    // First, search locally loaded items
    const localResults = searchLocalItems(query);

    // If we have good matches, return them immediately
    if (localResults.length > 0 && localResults[0].score > 0.8) {
        return localResults;
    }

    // Otherwise, search full database via function
    const serverResults = await searchViaFunction(query);
    return serverResults;
}

function searchLocalItems(query) {
    return loadedItems.map(item => ({
        item,
        score: elid.bestMatch(query.toLowerCase(), item.name.toLowerCase())
    }))
    .filter(r => r.score >= 0.5)
    .sort((a, b) => b.score - a.score);
}

async function searchViaFunction(query) {
    const response = await fetch(`${endpoint}/functions/${functionId}/executions`, {
        method: 'POST',
        body: JSON.stringify({ query })
    });
    return response.json();
}
```

## Advanced Use Cases

### Product Autocomplete

```javascript
// Appwrite Function for product autocomplete
module.exports = async ({ req, res }) => {
    const { query } = JSON.parse(req.bodyRaw);
    const database = new sdk.Databases(client);

    const products = await database.listDocuments('db', 'products', [
        sdk.Query.limit(100) // Limit for performance
    ]);

    const suggestions = products.documents
        .map(p => ({
            id: p.$id,
            name: p.name,
            score: elid.bestMatch(query.toLowerCase(), p.name.toLowerCase())
        }))
        .filter(p => p.score >= 0.4)
        .sort((a, b) => b.score - a.score)
        .slice(0, 5);

    return res.json({ suggestions });
};
```

### User Name Deduplication

```javascript
// Check for duplicate users when registering
module.exports = async ({ req, res }) => {
    const { name, email } = JSON.parse(req.bodyRaw);
    const database = new sdk.Databases(client);

    const users = await database.listDocuments('db', 'users');

    // Find potential duplicates
    const duplicates = users.documents.filter(user => {
        const nameSim = elid.jaroWinkler(
            name.toLowerCase(),
            user.name.toLowerCase()
        );
        const emailSim = elid.normalizedLevenshtein(
            email.toLowerCase(),
            user.email.toLowerCase()
        );

        return nameSim > 0.85 || emailSim > 0.9;
    });

    if (duplicates.length > 0) {
        return res.json({
            warning: true,
            message: 'Potential duplicate account found',
            duplicates
        });
    }

    return res.json({ ok: true });
};
```

### Multi-field Search

```javascript
// Search across multiple fields with weighted scoring
function multiFieldSearch(query, documents) {
    return documents.map(doc => {
        const titleScore = elid.bestMatch(query, doc.title || '') * 2.0;  // Weight: 2x
        const descScore = elid.bestMatch(query, doc.description || '');   // Weight: 1x
        const tagScore = doc.tags?.some(tag =>
            elid.bestMatch(query, tag) > 0.7
        ) ? 1.0 : 0;  // Bonus: 1.0

        const totalScore = (titleScore + descScore + tagScore) / 4;

        return { document: doc, score: totalScore };
    })
    .filter(r => r.score >= 0.4)
    .sort((a, b) => b.score - a.score);
}
```

## Performance Tips

### 1. Limit Dataset Size

For large collections, pre-filter before fuzzy matching:

```javascript
const documents = await database.listDocuments('db', 'collection', [
    sdk.Query.search('name', query),  // Appwrite's built-in search first
    sdk.Query.limit(100)
]);

// Then apply fuzzy matching to these 100
const fuzzyResults = documents.documents.map(/* ... */);
```

### 2. Cache Results

```javascript
const cache = new Map();

function cachedFuzzySearch(query, items) {
    const cacheKey = `${query}:${items.length}`;
    if (cache.has(cacheKey)) {
        return cache.get(cacheKey);
    }

    const results = performFuzzySearch(query, items);
    cache.set(cacheKey, results);
    return results;
}
```

### 3. Debounce User Input

```javascript
import { debounce } from 'lodash';

const debouncedSearch = debounce((query) => {
    performSearch(query);
}, 300); // Wait 300ms after user stops typing

searchInput.addEventListener('input', (e) => {
    debouncedSearch(e.target.value);
});
```

## Deployment

### Deploy Function

```bash
appwrite deploy function
```

### Set Function Permissions

```javascript
// Allow all users to execute
await functions.updateExecution(
    functionId,
    'role:all'
);
```

## Testing

Test your function locally:

```bash
curl -X POST http://localhost:3000/v1/functions/fuzzy-search/executions \
  -H "Content-Type: application/json" \
  -d '{
    "query": "test",
    "candidates": ["test1", "test2", "testing"]
  }'
```

## Monitoring

Track performance in your function:

```javascript
module.exports = async ({ req, res, log }) => {
    const start = Date.now();

    // ... your search logic ...

    const duration = Date.now() - start;
    log(`Search completed in ${duration}ms`);

    return res.json({ results, meta: { duration } });
};
```

## Need Help?

- [ELID Documentation](https://github.com/ZachHandley/ELID)
- [Appwrite Functions Docs](https://appwrite.io/docs/functions)
- [Report Issues](https://github.com/ZachHandley/ELID/issues)
