/**
 * Appwrite Function: SimHash Database Queries
 *
 * This example shows how to use SimHash for efficient similarity queries
 * in an Appwrite database by storing numeric hashes.
 *
 * Setup:
 * 1. Build WASM for Node.js: npm run build:node
 * 2. Copy pkg-node folder to your Appwrite function directory
 * 3. Deploy this function to Appwrite
 *
 * Database Schema Example:
 * {
 *   name: "iPhone 14 Pro",
 *   simhash: 8234567890123456,  // Pre-computed SimHash
 *   category: "phones"
 * }
 */

const sdk = require('node-appwrite');
const elid = require('../../pkg-node/elid');

/**
 * Main function handler
 */
module.exports = async function ({ req, res, log, error }) {
    try {
        const body = JSON.parse(req.bodyRaw || '{}');
        const {
            query,
            collectionId,
            searchField = 'name',
            maxDistance = 10,  // Hamming distance threshold
            limit = 20
        } = body;

        if (!query || !collectionId) {
            return res.json({
                success: false,
                error: 'query and collectionId are required'
            }, 400);
        }

        log(`SimHash query: "${query}" in collection ${collectionId}`);

        // Initialize Appwrite client
        const client = new sdk.Client()
            .setEndpoint(process.env.APPWRITE_FUNCTION_ENDPOINT)
            .setProject(process.env.APPWRITE_FUNCTION_PROJECT_ID)
            .setKey(process.env.APPWRITE_API_KEY);

        const database = new sdk.Databases(client);

        // Compute SimHash for the query
        const queryHash = elid.simhash(query);

        log(`Query SimHash: ${queryHash}`);

        // Option 1: If you have simhash field in database (RECOMMENDED)
        // Query by numeric range for efficiency
        const rangeSize = Math.pow(2, maxDistance); // Approximate range based on max distance
        const minHash = Math.max(0, queryHash - rangeSize);
        const maxHash = queryHash + rangeSize;

        try {
            // First, try to query by simhash range (if indexed)
            const documents = await database.listDocuments(
                process.env.DATABASE_ID,
                collectionId,
                [
                    sdk.Query.greaterThanEqual('simhash', minHash),
                    sdk.Query.lessThanEqual('simhash', maxHash),
                    sdk.Query.limit(100)
                ]
            );

            // Then filter by exact Hamming distance
            const matches = documents.documents
                .map(doc => ({
                    document: doc,
                    storedHash: doc.simhash,
                    distance: elid.simhashDistance(queryHash, doc.simhash)
                }))
                .filter(item => item.distance <= maxDistance)
                .sort((a, b) => a.distance - b.distance)
                .slice(0, limit);

            log(`Found ${matches.length} matches within distance ${maxDistance}`);

            return res.json({
                success: true,
                query,
                queryHash,
                results: matches.map(m => ({
                    documentId: m.document.$id,
                    name: m.document[searchField],
                    simhash: m.storedHash,
                    distance: m.distance,
                    similarity: 1.0 - (m.distance / 64.0),
                    document: m.document
                })),
                metadata: {
                    totalCandidates: documents.documents.length,
                    matchesFound: matches.length,
                    maxDistance,
                    queryHash
                }
            });

        } catch (dbError) {
            // Option 2: Fallback - if simhash field doesn't exist, compute on the fly
            log('SimHash field not found, computing hashes on the fly...');

            const documents = await database.listDocuments(
                process.env.DATABASE_ID,
                collectionId,
                []
            );

            const matches = documents.documents
                .map(doc => ({
                    document: doc,
                    hash: elid.simhash(doc[searchField] || ''),
                    distance: elid.simhashDistance(queryHash, elid.simhash(doc[searchField] || ''))
                }))
                .filter(item => item.distance <= maxDistance)
                .sort((a, b) => a.distance - b.distance)
                .slice(0, limit);

            return res.json({
                success: true,
                query,
                queryHash,
                results: matches.map(m => ({
                    documentId: m.document.$id,
                    name: m.document[searchField],
                    computedHash: m.hash,
                    distance: m.distance,
                    similarity: 1.0 - (m.distance / 64.0),
                    document: m.document
                })),
                metadata: {
                    totalCandidates: documents.documents.length,
                    matchesFound: matches.length,
                    maxDistance,
                    queryHash,
                    note: 'Hashes computed on the fly - consider adding simhash field for better performance'
                }
            });
        }

    } catch (err) {
        error(`Error in SimHash search: ${err.message}`);
        return res.json({
            success: false,
            error: err.message
        }, 500);
    }
};

/**
 * Helper function to update all documents with SimHash values
 * Call this once to initialize simhash field for all existing documents
 */
async function initializeSimHashes({ collectionId, searchField = 'name' }) {
    const client = new sdk.Client()
        .setEndpoint(process.env.APPWRITE_FUNCTION_ENDPOINT)
        .setProject(process.env.APPWRITE_FUNCTION_PROJECT_ID)
        .setKey(process.env.APPWRITE_API_KEY);

    const database = new sdk.Databases(client);

    try {
        const documents = await database.listDocuments(
            process.env.DATABASE_ID,
            collectionId,
            []
        );

        for (const doc of documents.documents) {
            const hash = elid.simhash(doc[searchField] || '');

            await database.updateDocument(
                process.env.DATABASE_ID,
                collectionId,
                doc.$id,
                { simhash: hash }
            );
        }

        return {
            success: true,
            updatedCount: documents.documents.length
        };

    } catch (err) {
        return {
            success: false,
            error: err.message
        };
    }
}

module.exports.initializeSimHashes = initializeSimHashes;
