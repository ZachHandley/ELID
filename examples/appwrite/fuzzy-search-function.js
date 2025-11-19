/**
 * Appwrite Function: Fuzzy Search with ELID
 *
 * This function provides fuzzy string similarity search for Appwrite databases.
 * It can be deployed as an Appwrite Function and called via the Functions API.
 *
 * Setup:
 * 1. Build WASM for Node.js: npm run build:node
 * 2. Copy pkg-node folder to your Appwrite function directory
 * 3. Deploy this function to Appwrite
 *
 * Example Usage:
 *
 * POST /v1/functions/{functionId}/executions
 * {
 *   "query": "iphone 14",
 *   "candidates": ["iPhone 14 Pro", "iPhone 13", "Samsung Galaxy"],
 *   "threshold": 0.5,
 *   "algorithm": "best"  // or "levenshtein", "jaro-winkler"
 * }
 */

const sdk = require('node-appwrite');
const elid = require('../../pkg-node/elid');

/**
 * Main function handler
 */
module.exports = async function ({ req, res, log, error }) {
    try {
        // Parse request body
        const body = JSON.parse(req.bodyRaw || '{}');
        const {
            query,
            candidates = [],
            threshold = 0.5,
            algorithm = 'best',
            limit = 10,
            caseSensitive = false
        } = body;

        // Validate input
        if (!query) {
            return res.json({
                success: false,
                error: 'Query parameter is required'
            }, 400);
        }

        if (!Array.isArray(candidates) || candidates.length === 0) {
            return res.json({
                success: false,
                error: 'Candidates array is required and must not be empty'
            }, 400);
        }

        log(`Fuzzy search: "${query}" in ${candidates.length} candidates`);

        // Prepare query (case handling)
        const processedQuery = caseSensitive ? query : query.toLowerCase();
        const processedCandidates = candidates.map(c =>
            caseSensitive ? c : (typeof c === 'string' ? c.toLowerCase() : c)
        );

        // Perform similarity search based on algorithm
        let results;
        switch (algorithm) {
            case 'levenshtein':
                results = processedCandidates.map((candidate, i) => ({
                    index: i,
                    value: candidates[i],
                    score: elid.normalizedLevenshtein(processedQuery, candidate)
                }));
                break;

            case 'jaro-winkler':
                results = processedCandidates.map((candidate, i) => ({
                    index: i,
                    value: candidates[i],
                    score: elid.jaroWinkler(processedQuery, candidate)
                }));
                break;

            case 'best':
            default:
                results = processedCandidates.map((candidate, i) => ({
                    index: i,
                    value: candidates[i],
                    score: elid.bestMatch(processedQuery, candidate)
                }));
                break;
        }

        // Filter by threshold and sort by score
        results = results
            .filter(r => r.score >= threshold)
            .sort((a, b) => b.score - a.score)
            .slice(0, limit);

        log(`Found ${results.length} matches above threshold ${threshold}`);

        return res.json({
            success: true,
            query,
            algorithm,
            threshold,
            results,
            metadata: {
                totalCandidates: candidates.length,
                matchesFound: results.length,
                processingTime: Date.now()
            }
        });

    } catch (err) {
        error(`Error in fuzzy search function: ${err.message}`);
        return res.json({
            success: false,
            error: err.message
        }, 500);
    }
};

/**
 * Example: Fuzzy search in Appwrite Database
 *
 * This demonstrates how to use the function with Appwrite Database queries
 */
async function fuzzySearchDatabase({ query, collectionId, searchField, threshold = 0.5 }) {
    // This would be called from your Appwrite Function
    const client = new sdk.Client();
    const database = new sdk.Databases(client);

    // Configure client (these would come from environment variables)
    client
        .setEndpoint(process.env.APPWRITE_FUNCTION_ENDPOINT)
        .setProject(process.env.APPWRITE_FUNCTION_PROJECT_ID)
        .setKey(process.env.APPWRITE_API_KEY);

    try {
        // Get all documents from the collection
        const documents = await database.listDocuments(
            process.env.DATABASE_ID,
            collectionId,
            []
        );

        // Extract search field values
        const candidates = documents.documents.map(doc => ({
            id: doc.$id,
            value: doc[searchField],
            document: doc
        }));

        // Score each candidate
        const processedQuery = query.toLowerCase();
        const scored = candidates.map(candidate => ({
            ...candidate,
            score: elid.bestMatch(
                processedQuery,
                candidate.value.toLowerCase()
            )
        }));

        // Filter and sort
        const results = scored
            .filter(r => r.score >= threshold)
            .sort((a, b) => b.score - a.score);

        return {
            success: true,
            query,
            results: results.map(r => ({
                documentId: r.id,
                value: r.value,
                score: r.score,
                document: r.document
            }))
        };

    } catch (err) {
        return {
            success: false,
            error: err.message
        };
    }
}

// Export for use in other contexts
module.exports.fuzzySearchDatabase = fuzzySearchDatabase;
