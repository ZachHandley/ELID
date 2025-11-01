/**
 * Semantic Search Example using ELID
 *
 * This example demonstrates how to use ELID for efficient semantic similarity search
 * over a collection of text documents. It shows:
 *
 * 1. Encoding document embeddings into ELIDs
 * 2. Building an in-memory search index
 * 3. Finding similar documents using Hamming distance
 * 4. Optimizing search with pre-filtering
 */

import { encodeElid, hammingDistanceElid, encodeBatch, ElidProfile } from '../index';

// Mock embedding generator (replace with actual embedding model in production)
// In practice, use a model like OpenAI's text-embedding-3-small, sentence-transformers, etc.
function getEmbedding(text: string): Float64Array {
  // This is a mock - in reality, call your embedding model API
  const embedding = new Float64Array(768);

  // Generate deterministic pseudo-embeddings based on text hash
  let hash = 0;
  for (let i = 0; i < text.length; i++) {
    hash = ((hash << 5) - hash) + text.charCodeAt(i);
    hash = hash & hash; // Convert to 32bit integer
  }

  for (let i = 0; i < 768; i++) {
    // Seed RNG with hash and index for determinism
    const seed = hash + i;
    const x = Math.sin(seed) * 10000;
    embedding[i] = x - Math.floor(x);
  }

  return embedding;
}

// Document interface
interface Document {
  id: string;
  title: string;
  content: string;
}

// Indexed document with ELID
interface IndexedDocument extends Document {
  embedding: Float64Array;
  elid: string;
}

// Sample documents
const documents: Document[] = [
  {
    id: '1',
    title: 'Introduction to Machine Learning',
    content: 'Machine learning is a subset of artificial intelligence that enables systems to learn and improve from experience without explicit programming.',
  },
  {
    id: '2',
    title: 'Deep Learning Fundamentals',
    content: 'Deep learning uses neural networks with multiple layers to progressively extract higher-level features from raw input data.',
  },
  {
    id: '3',
    title: 'Natural Language Processing',
    content: 'NLP combines linguistics and machine learning to enable computers to understand, interpret and generate human language.',
  },
  {
    id: '4',
    title: 'Computer Vision Techniques',
    content: 'Computer vision enables machines to derive meaningful information from digital images, videos and other visual inputs.',
  },
  {
    id: '5',
    title: 'Reinforcement Learning Basics',
    content: 'Reinforcement learning trains agents to make decisions by rewarding desired behaviors and punishing undesired ones.',
  },
  {
    id: '6',
    title: 'Neural Network Architectures',
    content: 'Neural networks consist of interconnected nodes organized in layers, with each connection having adjustable weights.',
  },
  {
    id: '7',
    title: 'Training Deep Models',
    content: 'Training deep neural networks requires careful selection of architectures, optimization algorithms, and regularization techniques.',
  },
  {
    id: '8',
    title: 'Transfer Learning Applications',
    content: 'Transfer learning reuses pre-trained models on new tasks, significantly reducing training time and data requirements.',
  },
  {
    id: '9',
    title: 'Transformer Models',
    content: 'Transformers use self-attention mechanisms to process sequential data in parallel, revolutionizing NLP tasks.',
  },
  {
    id: '10',
    title: 'Embeddings and Representation Learning',
    content: 'Embeddings map discrete objects to continuous vector spaces, capturing semantic relationships between items.',
  },
];

/**
 * Build search index from documents
 */
async function buildIndex(docs: Document[]): Promise<IndexedDocument[]> {
  console.log('📚 Building search index...\n');

  // Generate embeddings for all documents
  console.log('🔄 Generating embeddings...');
  const embeddings = docs.map(doc => getEmbedding(doc.title + ' ' + doc.content));

  // Encode embeddings to ELIDs in batch
  console.log('🔢 Encoding ELIDs in batch...');
  const startTime = Date.now();
  const elids = await encodeBatch(embeddings, ElidProfile.Mini128);
  const duration = Date.now() - startTime;

  console.log(`✅ Indexed ${docs.length} documents in ${duration}ms\n`);

  // Create indexed documents
  return docs.map((doc, i) => ({
    ...doc,
    embedding: embeddings[i],
    elid: elids[i],
  }));
}

/**
 * Search for similar documents
 */
function search(
  query: string,
  index: IndexedDocument[],
  topK: number = 5
): Array<{ doc: IndexedDocument; distance: number; similarity: number }> {
  console.log(`🔍 Searching for: "${query}"\n`);

  // Generate query embedding
  const queryEmbedding = getEmbedding(query);

  // Encode query to ELID
  const queryElid = encodeElid(queryEmbedding, ElidProfile.Mini128);
  console.log(`Query ELID: ${queryElid}`);

  // Calculate Hamming distances to all documents
  const startTime = Date.now();
  const results = index.map(doc => {
    const distance = hammingDistanceElid(queryElid, doc.elid);
    // Convert Hamming distance to similarity score (0-1)
    // Lower distance = higher similarity
    const similarity = 1 - (distance / 128);

    return { doc, distance, similarity };
  });

  // Sort by similarity (ascending distance)
  results.sort((a, b) => a.distance - b.distance);

  const duration = Date.now() - startTime;
  console.log(`⚡ Search completed in ${duration}ms\n`);

  // Return top K results
  return results.slice(0, topK);
}

/**
 * Display search results
 */
function displayResults(
  results: Array<{ doc: IndexedDocument; distance: number; similarity: number }>
): void {
  console.log('📊 Top Results:\n');

  results.forEach((result, i) => {
    const { doc, distance, similarity } = result;
    console.log(`${i + 1}. ${doc.title}`);
    console.log(`   Similarity: ${(similarity * 100).toFixed(1)}% (Hamming distance: ${distance})`);
    console.log(`   ELID: ${doc.elid}`);
    console.log(`   Content: ${doc.content.substring(0, 80)}...`);
    console.log('');
  });
}

/**
 * Advanced search with pre-filtering
 */
function advancedSearch(
  query: string,
  index: IndexedDocument[],
  maxDistance: number = 64, // Only consider documents within this Hamming distance
  topK: number = 5
): Array<{ doc: IndexedDocument; distance: number; similarity: number }> {
  console.log(`🔍 Advanced search with pre-filtering (max distance: ${maxDistance})\n`);

  const queryEmbedding = getEmbedding(query);
  const queryElid = encodeElid(queryEmbedding, ElidProfile.Mini128);

  // Pre-filter by Hamming distance
  const startTime = Date.now();
  const candidates = index
    .map(doc => {
      const distance = hammingDistanceElid(queryElid, doc.elid);
      return { doc, distance };
    })
    .filter(result => result.distance <= maxDistance);

  const duration = Date.now() - startTime;
  console.log(`Found ${candidates.length} candidates in ${duration}ms\n`);

  // Sort by distance
  candidates.sort((a, b) => a.distance - b.distance);

  // Convert to result format
  return candidates.slice(0, topK).map(({ doc, distance }) => ({
    doc,
    distance,
    similarity: 1 - (distance / 128),
  }));
}

/**
 * Batch similarity comparison
 */
function batchSimilarity(
  targetDoc: IndexedDocument,
  index: IndexedDocument[]
): Array<{ doc: IndexedDocument; distance: number }> {
  const startTime = Date.now();

  const similarities = index
    .filter(doc => doc.id !== targetDoc.id) // Exclude self
    .map(doc => ({
      doc,
      distance: hammingDistanceElid(targetDoc.elid, doc.elid),
    }))
    .sort((a, b) => a.distance - b.distance);

  const duration = Date.now() - startTime;
  console.log(`⚡ Computed ${similarities.length} similarities in ${duration}ms\n`);

  return similarities;
}

/**
 * Main execution
 */
async function main() {
  console.log('🚀 ELID Semantic Search Demo\n');
  console.log('='.repeat(50));
  console.log('');

  // Build index
  const index = await buildIndex(documents);

  console.log('='.repeat(50));
  console.log('');

  // Example 1: Basic search
  const query1 = 'neural networks and deep learning';
  const results1 = search(query1, index, 3);
  displayResults(results1);

  console.log('='.repeat(50));
  console.log('');

  // Example 2: Different query
  const query2 = 'understanding natural language';
  const results2 = search(query2, index, 3);
  displayResults(results2);

  console.log('='.repeat(50));
  console.log('');

  // Example 3: Advanced search with filtering
  const query3 = 'training machine learning models';
  const results3 = advancedSearch(query3, index, 50, 3);
  displayResults(results3);

  console.log('='.repeat(50));
  console.log('');

  // Example 4: Find similar documents to a specific document
  const targetDoc = index[1]; // "Deep Learning Fundamentals"
  console.log(`📖 Finding documents similar to: "${targetDoc.title}"\n`);

  const similarDocs = batchSimilarity(targetDoc, index).slice(0, 3);

  console.log('📊 Most Similar Documents:\n');
  similarDocs.forEach((result, i) => {
    console.log(`${i + 1}. ${result.doc.title}`);
    console.log(`   Hamming Distance: ${result.distance}`);
    console.log(`   Similarity: ${((1 - result.distance / 128) * 100).toFixed(1)}%`);
    console.log('');
  });

  console.log('='.repeat(50));
  console.log('');

  // Performance stats
  console.log('📈 Performance Summary:\n');
  console.log(`- Documents indexed: ${index.length}`);
  console.log(`- Embedding dimensions: 768`);
  console.log(`- ELID length: 29 characters (base32hex)`);
  console.log(`- Hamming distance computation: <1ms per comparison`);
  console.log(`- Search latency: ~${index.length}ms for ${index.length} documents`);
  console.log('');

  console.log('✅ Demo completed!');
}

// Run the demo
main().catch(console.error);
