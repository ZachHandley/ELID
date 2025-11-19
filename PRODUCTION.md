# Production Readiness Guide

This document outlines production readiness considerations for deploying ELID in production environments.

## ✅ Production-Ready Features

### Core Library
- **48 comprehensive tests** - All passing with good coverage
- **Zero unsafe code** - Safe Rust throughout
- **Zero external dependencies** - Core algorithms are pure Rust
- **Stable SimHash** - Uses FNV-1a hash for consistency across Rust versions
- **Well-documented API** - Comprehensive docs and examples

### Language Bindings
- **Python** - 13/13 functions tested and working (PyO3)
- **JavaScript/WASM** - 9/9 functions tested and working
- **TypeScript** - Type definitions included
- **Multiple targets** - Browser, Node.js, Deno, Bun support

### Performance
- **Space-optimized Levenshtein** - O(min(m,n)) space complexity
- **Efficient algorithms** - Industry-standard implementations
- **Release builds** - Optimized with LTO and single codegen unit

## ⚠️ Production Considerations

### 1. Input Validation (Recommended)

The library does not enforce maximum string length limits to maintain flexibility. For production use, validate input sizes to prevent DoS attacks:

```rust
const MAX_INPUT_LENGTH: usize = 1_000_000; // 1MB

fn safe_levenshtein(a: &str, b: &str) -> Result<usize, &'static str> {
    if a.len() > MAX_INPUT_LENGTH || b.len() > MAX_INPUT_LENGTH {
        return Err("Input exceeds maximum length");
    }
    Ok(elid::levenshtein(a, b))
}
```

```python
MAX_INPUT_LENGTH = 1_000_000

def safe_levenshtein(a: str, b: str) -> int:
    if len(a) > MAX_INPUT_LENGTH or len(b) > MAX_INPUT_LENGTH:
        raise ValueError("Input exceeds maximum length")
    return elid.levenshtein(a, b)
```

```javascript
const MAX_INPUT_LENGTH = 1_000_000;

function safeLevenshtein(a, b) {
    if (a.length > MAX_INPUT_LENGTH || b.length > MAX_INPUT_LENGTH) {
        throw new Error("Input exceeds maximum length");
    }
    return elid.levenshtein(a, b);
}
```

### 2. SimHash Database Migration

**IMPORTANT:** SimHash values changed in v0.1.0 from DefaultHasher to FNV-1a for stability.

If you have existing SimHash values in your database:
1. They will need to be recomputed
2. Run a migration to update all stored hashes
3. Consider versioning your hash values

Example migration script:

```javascript
// Appwrite migration example
const elid = require('./pkg-node/elid');
const sdk = require('node-appwrite');

async function migrateSimHashes() {
    const database = new sdk.Databases(client);
    const docs = await database.listDocuments(dbId, collectionId);

    for (const doc of docs.documents) {
        const newHash = elid.simhash(doc.name);
        await database.updateDocument(dbId, collectionId, doc.$id, {
            simhash: newHash,
            simhash_version: 2  // Track version
        });
    }
}
```

### 3. Rate Limiting

For public-facing APIs, implement rate limiting:

```javascript
// Appwrite Function example
const rateLimit = new Map();
const MAX_REQUESTS_PER_MINUTE = 100;

function checkRateLimit(userId) {
    const now = Date.now();
    const userRequests = rateLimit.get(userId) || [];

    // Remove old requests
    const recentRequests = userRequests.filter(
        time => now - time < 60000
    );

    if (recentRequests.length >= MAX_REQUESTS_PER_MINUTE) {
        throw new Error('Rate limit exceeded');
    }

    recentRequests.push(now);
    rateLimit.set(userId, recentRequests);
}
```

### 4. Monitoring and Observability

Track key metrics in production:

- **Request volume** - Monitor similarity computation requests
- **Input sizes** - Track average/max string lengths
- **Response times** - Monitor p50, p95, p99 latency
- **Error rates** - Track validation failures, timeouts
- **SimHash distances** - Monitor distance distributions

Example metrics:

```javascript
const metrics = {
    requestCount: 0,
    avgInputLength: 0,
    maxInputLength: 0,
    avgResponseTime: 0
};

function recordMetrics(inputA, inputB, duration) {
    metrics.requestCount++;
    metrics.avgInputLength =
        (metrics.avgInputLength * (metrics.requestCount - 1) +
         inputA.length + inputB.length) / (2 * metrics.requestCount);
    metrics.maxInputLength = Math.max(
        metrics.maxInputLength,
        inputA.length,
        inputB.length
    );
    metrics.avgResponseTime =
        (metrics.avgResponseTime * (metrics.requestCount - 1) + duration) /
        metrics.requestCount;
}
```

### 5. Database Indexing

For SimHash database queries, ensure proper indexing:

**Appwrite:**
```javascript
// Create index on simhash field
await database.createIndex(
    databaseId,
    collectionId,
    'simhash_idx',
    'key',
    ['simhash'],
    ['asc']
);
```

**PostgreSQL:**
```sql
CREATE INDEX idx_simhash ON products(simhash);
-- For range queries
CREATE INDEX idx_simhash_btree ON products USING BTREE(simhash);
```

**MySQL:**
```sql
CREATE INDEX idx_simhash ON products(simhash);
```

### 6. Error Handling

Implement proper error handling:

```javascript
try {
    const result = elid.levenshtein(userInput1, userInput2);
    return res.json({ success: true, distance: result });
} catch (err) {
    log.error(`Similarity computation failed: ${err.message}`);
    return res.json({
        success: false,
        error: 'Internal error processing request'
    }, 500);
}
```

### 7. Caching

Cache frequently computed similarities:

```javascript
const cache = new Map();
const CACHE_SIZE = 10000;

function getCachedSimilarity(a, b) {
    const key = `${a}:${b}`;
    if (cache.has(key)) {
        return cache.get(key);
    }

    const similarity = elid.normalizedLevenshtein(a, b);

    if (cache.size >= CACHE_SIZE) {
        // Remove oldest entry
        const firstKey = cache.keys().next().value;
        cache.delete(firstKey);
    }

    cache.set(key, similarity);
    return similarity;
}
```

### 8. Security Considerations

- **No injection vulnerabilities** - Pure computation, no SQL/code execution
- **No cryptographic guarantees** - SimHash uses FNV-1a (non-cryptographic)
- **DoS protection** - Implement input size limits and rate limiting
- **No PII exposure** - Ensure similarity queries don't leak sensitive data

### 9. Performance Testing

Before production deployment, benchmark with your data:

```bash
# Rust benchmarks
cargo bench

# Custom tests with your data
cat your_test_data.txt | while read line; do
    time node -e "const elid = require('./pkg-node/elid'); console.log(elid.simhash('$line'));"
done
```

### 10. Versioning Strategy

For APIs exposing ELID:
- Version your API endpoints
- Track algorithm versions in responses
- Document breaking changes
- Consider backwards compatibility

Example:

```javascript
return res.json({
    version: "1.0",
    algorithm: "levenshtein",
    result: {
        distance: 3,
        similarity: 0.857
    },
    metadata: {
        library_version: "0.1.0"
    }
});
```

## Testing Checklist

Before production deployment:

- [ ] Run all tests: `cargo test`, `node test-wasm.js`, `python test-python.py`
- [ ] Load test with production-like data
- [ ] Test with maximum expected input sizes
- [ ] Verify database query performance with SimHash
- [ ] Test error handling and edge cases
- [ ] Verify monitoring and alerting
- [ ] Test rate limiting
- [ ] Review security considerations
- [ ] Document deployment process
- [ ] Plan rollback strategy

## Known Limitations

1. **No true edit distance variants** - OSA is not Damerau-Levenshtein
2. **Unicode handling** - Works with UTF-8, but doesn't normalize (e.g., "é" vs "e´")
3. **Memory usage** - O(n) for most algorithms, can be high for very large strings
4. **SimHash approximation** - Similar strings may have different hashes occasionally
5. **WASM f64 limitation** - SimHash uses f64 in WASM (loses precision for some u64 values)

## Deployment Examples

### Appwrite Function

See `examples/appwrite/simhash-database-query.js` for a complete production-ready Appwrite Function.

### AWS Lambda (Node.js)

```javascript
const elid = require('./pkg-node/elid');

exports.handler = async (event) => {
    try {
        const { query, candidates } = JSON.parse(event.body);
        const result = elid.findBestMatch(query, candidates);

        return {
            statusCode: 200,
            body: JSON.stringify(result)
        };
    } catch (err) {
        return {
            statusCode: 500,
            body: JSON.stringify({ error: err.message })
        };
    }
};
```

### Docker Deployment (Python)

```dockerfile
FROM python:3.11-slim

WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
RUN pip install elid

COPY . .
CMD ["python", "app.py"]
```

## Support and Updates

- **GitHub Issues**: Report bugs at https://github.com/ZachHandley/ELID/issues
- **Version Updates**: Check releases for new features and bug fixes
- **Breaking Changes**: Will be documented in CHANGELOG.md

## License

MIT OR Apache-2.0 - See LICENSE files for details.
