# SimHash Guide - Numeric Similarity Queries for Databases

SimHash creates a **numeric fingerprint** (64-bit integer) where **similar strings produce similar numbers**. This allows you to query databases using numeric comparisons instead of fetching everything and computing similarity.

## What is SimHash?

SimHash is a **locality-sensitive hash** - similar inputs produce similar outputs:

```javascript
const hash1 = elid.simhash("iPhone 14");     // 8234567890123456
const hash2 = elid.simhash("iPhone 15");     // 8234567890125678 (close!)
const hash3 = elid.simhash("Galaxy S23");    // 1827364950273849 (far!)
```

**Key Properties:**
- Similar strings → Similar hash values
- Fast to compute (O(n) where n is string length)
- Fixed size output (64-bit integer)
- Can query by numeric range in databases
- Hamming distance between hashes correlates with string similarity

## How It Works

1. **Extract features** from the string (character trigrams + words)
2. **Hash each feature** to get a 64-bit number
3. **Vote on each bit** position using all features
4. **Create final fingerprint** based on bit votes

**Example:**
```
"iPhone 14" →
  Features: ["iph", "pho", "hon", "one", " 14", "iphone", "14"]
  Each feature votes on 64 bit positions
  Result: 8234567890123456 (64-bit hash)
```

## Database Storage

### Schema Design

Add a `simhash` field to store the pre-computed hash:

```javascript
// Example document
{
  $id: "doc123",
  name: "iPhone 14 Pro",
  category: "phones",
  price: 999,
  simhash: 8234567890123456  // Store this!
}
```

**Create index on simhash field** for fast queries.

### Computing Hashes

#### Option 1: Pre-compute on Insert/Update (RECOMMENDED)

```javascript
// When creating a document
const product = {
  name: "iPhone 14 Pro",
  price: 999
};

product.simhash = elid.simhash(product.name);

await database.createDocument('products', product);
```

#### Option 2: Batch Update Existing Data

```javascript
// Update all existing documents
const documents = await database.listDocuments('products');

for (const doc of documents.documents) {
  const hash = elid.simhash(doc.name);
  await database.updateDocument('products', doc.$id, { simhash: hash });
}
```

## Querying with SimHash

### 1. Basic Similarity Query

```javascript
const query = "iPhone 14";
const queryHash = elid.simhash(query);

// Method A: Fetch candidates in numeric range, then filter by distance
const rangeSize = 1000; // Adjust based on your needs
const candidates = await database.listDocuments('products', [
  Query.greaterThanEqual('simhash', queryHash - rangeSize),
  Query.lessThanEqual('simhash', queryHash + rangeSize)
]);

// Filter by exact Hamming distance
const maxDistance = 10;
const matches = candidates.documents
  .filter(doc => {
    const distance = elid.simhashDistance(queryHash, doc.simhash);
    return distance <= maxDistance;
  })
  .sort((a, b) => {
    const distA = elid.simhashDistance(queryHash, a.simhash);
    const distB = elid.simhashDistance(queryHash, b.simhash);
    return distA - distB;
  });
```

### 2. Appwrite Example

```javascript
const elid = require('./pkg-node/elid');
const { Client, Databases, Query } = require('node-appwrite');

async function fuzzySearch(query, collectionId) {
  const queryHash = elid.simhash(query);
  const maxDistance = 10;

  // Query by range
  const documents = await database.listDocuments('db', collectionId, [
    Query.greaterThanEqual('simhash', queryHash - 10000),
    Query.lessThanEqual('simhash', queryHash + 10000),
    Query.limit(100)
  ]);

  // Filter by exact distance
  return documents.documents
    .map(doc => ({
      ...doc,
      distance: elid.simhashDistance(queryHash, doc.simhash)
    }))
    .filter(doc => doc.distance <= maxDistance)
    .sort((a, b) => a.distance - b.distance);
}
```

### 3. SQL Database Example

```sql
-- Create table with simhash column
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255),
  simhash BIGINT
);

-- Create index on simhash
CREATE INDEX idx_simhash ON products(simhash);

-- Query by range
SELECT * FROM products
WHERE simhash BETWEEN 8234567890123456 - 10000
                  AND 8234567890123456 + 10000;
```

Then filter results in application code by Hamming distance.

### 4. MongoDB Example

```javascript
const queryHash = elid.simhash("iPhone 14");
const rangeSize = 10000;

// Query by range
const candidates = await db.collection('products').find({
  simhash: {
    $gte: queryHash - rangeSize,
    $lte: queryHash + rangeSize
  }
}).toArray();

// Filter by Hamming distance
const maxDistance = 10;
const matches = candidates.filter(doc => {
  return elid.simhashDistance(queryHash, doc.simhash) <= maxDistance;
});
```

## Choosing the Right Threshold

The `maxDistance` parameter controls how similar matches must be:

| Max Distance | Similarity | Use Case |
|--------------|------------|----------|
| 0-5 | Very similar (95%+) | Exact match, minor typos |
| 6-10 | Similar (85-95%) | Product search, deduplication |
| 11-15 | Moderately similar (75-85%) | Loose matching |
| 16-20 | Somewhat similar (65-75%) | Broad search |
| 20+ | Low similarity (<65%) | Very fuzzy matching |

**Example:**
```javascript
// Strict matching - only very similar items
const strictMatches = findWithDistance(queryHash, hashes, 5);

// Loose matching - broader results
const looseMatches = findWithDistance(queryHash, hashes, 15);
```

## Performance Optimization

### 1. Index the SimHash Field

Always create an index on the `simhash` column for fast range queries:

```javascript
// Appwrite - create index in console or via API
await database.createIndex('products', 'simhash_idx', 'key', ['simhash']);
```

### 2. Adjust Range Size

Start with a smaller range and increase if needed:

```javascript
// Conservative range (faster, might miss some results)
const rangeSize = 1000;

// Liberal range (slower, more comprehensive)
const rangeSize = 100000;
```

### 3. Limit Initial Query

Always limit the initial database query:

```javascript
const documents = await database.listDocuments('products', [
  Query.greaterThanEqual('simhash', minHash),
  Query.lessThanEqual('simhash', maxHash),
  Query.limit(100)  // Prevent fetching too many
]);
```

### 4. Combine with Other Filters

Use SimHash with other query filters for better performance:

```javascript
const documents = await database.listDocuments('products', [
  Query.equal('category', 'phones'),  // Filter by category first
  Query.greaterThanEqual('simhash', minHash),
  Query.lessThanEqual('simhash', maxHash)
]);
```

## Real-World Examples

### Example 1: Product Search with Appwrite

```javascript
// products.js - Appwrite Function
const elid = require('./pkg-node/elid');
const { Databases } = require('node-appwrite');

module.exports = async ({ req, res }) => {
  const { query } = JSON.parse(req.bodyRaw);
  const database = new Databases(client);

  // Compute query hash
  const queryHash = elid.simhash(query.toLowerCase());

  // Query database
  const docs = await database.listDocuments('db', 'products', [
    Query.greaterThanEqual('simhash', queryHash - 5000),
    Query.lessThanEqual('simhash', queryHash + 5000),
    Query.limit(50)
  ]);

  // Filter and sort by similarity
  const results = docs.documents
    .map(doc => ({
      id: doc.$id,
      name: doc.name,
      price: doc.price,
      distance: elid.simhashDistance(queryHash, doc.simhash),
      similarity: 1.0 - (elid.simhashDistance(queryHash, doc.simhash) / 64.0)
    }))
    .filter(item => item.distance <= 10)
    .sort((a, b) => a.distance - b.distance);

  return res.json({ results });
};
```

### Example 2: Deduplication

```javascript
// Find potential duplicates when inserting new product
async function checkDuplicates(newProduct) {
  const hash = elid.simhash(newProduct.name);

  const existing = await database.listDocuments('products', [
    Query.greaterThanEqual('simhash', hash - 2000),
    Query.lessThanEqual('simhash', hash + 2000)
  ]);

  const duplicates = existing.documents.filter(doc => {
    const distance = elid.simhashDistance(hash, doc.simhash);
    return distance <= 5; // Very strict threshold for duplicates
  });

  if (duplicates.length > 0) {
    return {
      isDuplicate: true,
      matches: duplicates
    };
  }

  return { isDuplicate: false };
}
```

### Example 3: Autocomplete

```javascript
// Fast autocomplete with SimHash
async function autocomplete(partialQuery) {
  const queryHash = elid.simhash(partialQuery.toLowerCase());

  const suggestions = await database.listDocuments('products', [
    Query.greaterThanEqual('simhash', queryHash - 3000),
    Query.lessThanEqual('simhash', queryHash + 3000),
    Query.limit(20)
  ]);

  return suggestions.documents
    .map(doc => ({
      name: doc.name,
      distance: elid.simhashDistance(queryHash, doc.simhash)
    }))
    .filter(item => item.distance <= 12)
    .sort((a, b) => a.distance - b.distance)
    .slice(0, 5)
    .map(item => item.name);
}
```

## Python Examples

### Django ORM

```python
import elid
from django.db.models import Q

def fuzzy_search(query, max_distance=10):
    query_hash = elid.simhash(query.lower())
    range_size = 10000

    # Query by range
    candidates = Product.objects.filter(
        simhash__gte=query_hash - range_size,
        simhash__lte=query_hash + range_size
    )

    # Filter by Hamming distance
    results = []
    for product in candidates:
        distance = elid.simhash_distance(query_hash, product.simhash)
        if distance <= max_distance:
            results.append({
                'product': product,
                'distance': distance,
                'similarity': 1.0 - (distance / 64.0)
            })

    return sorted(results, key=lambda x: x['distance'])
```

### SQLAlchemy

```python
import elid
from sqlalchemy import and_

def fuzzy_search(session, query_text, max_distance=10):
    query_hash = elid.simhash(query_text.lower())
    range_size = 10000

    candidates = session.query(Product).filter(
        and_(
            Product.simhash >= query_hash - range_size,
            Product.simhash <= query_hash + range_size
        )
    ).all()

    matches = [
        {
            'product': p,
            'distance': elid.simhash_distance(query_hash, p.simhash)
        }
        for p in candidates
        if elid.simhash_distance(query_hash, p.simhash) <= max_distance
    ]

    return sorted(matches, key=lambda x: x['distance'])
```

## Limitations

1. **Not reversible**: You can't convert a hash back to the original string
2. **Approximate matching**: Some different strings may have similar hashes (rare)
3. **Language dependent**: Works best with Latin alphabets
4. **Short strings**: Less reliable for very short strings (< 3 characters)

## When to Use SimHash vs. Other Methods

**Use SimHash when:**
- ✅ You need to query by similarity in a database
- ✅ You have many items to search through
- ✅ You want O(log n) query time (with indexed simhash)
- ✅ You can pre-compute and store the hash

**Use regular similarity metrics when:**
- ❌ You have a small dataset (< 1000 items)
- ❌ You need exact similarity scores
- ❌ You can't add a simhash field to your database
- ❌ Query-time computation is acceptable

## Best Practices

1. **Always index the simhash field** in your database
2. **Pre-compute hashes** on insert/update, not at query time
3. **Start with conservative range sizes** and adjust based on results
4. **Combine with other filters** (category, date, etc.) for better performance
5. **Test different max_distance values** to find the right balance
6. **Monitor query performance** and adjust range/threshold accordingly

## Summary

SimHash enables **fast similarity queries** by converting strings to numbers:

```javascript
// 1. Pre-compute and store
product.simhash = elid.simhash(product.name);

// 2. Query by numeric range
const candidates = await db.query(
  simhash BETWEEN queryHash - range AND queryHash + range
);

// 3. Filter by exact distance
const matches = candidates.filter(c =>
  elid.simhashDistance(queryHash, c.simhash) <= 10
);
```

This approach is **10-100x faster** than computing similarity for every item at query time!
