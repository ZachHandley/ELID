/* tslint:disable */
/* eslint-disable */
/**
 * Encode an embedding vector to an ELID string.
 *
 * Converts a high-dimensional embedding (64-2048 dimensions) into a compact,
 * sortable identifier. The ELID preserves locality properties for efficient
 * similarity search.
 *
 * # Parameters
 *
 * - `embedding`: Float64 array of embedding values (64-2048 dimensions)
 * - `profile`: Encoding profile (Mini128, Morton10x10, or Hilbert10x10)
 *
 * # Returns
 *
 * A base32hex-encoded ELID string suitable for storage and comparison.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElid, ElidProfile } from 'elid';
 *
 * // OpenAI embeddings are 1536 dimensions
 * const embedding = await getEmbedding("Hello world");
 * const elid = encodeElid(embedding, ElidProfile.Mini128);
 * console.log(elid); // "012345abcdef..."
 * ```
 */
export function encodeElid(embedding: Float64Array, profile: ElidProfile): string;
/**
 * Compute the normalized Levenshtein similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { normalizedLevenshtein } from 'elid';
 *
 * const similarity = normalizedLevenshtein("hello", "hallo");
 * console.log(similarity); // ~0.8
 * ```
 */
export function normalizedLevenshtein(a: string, b: string): number;
/**
 * Compute the Jaro-Winkler similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 * Gives more favorable ratings to strings with common prefixes.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { jaroWinkler } from 'elid';
 *
 * const similarity = jaroWinkler("martha", "marhta");
 * console.log(similarity); // ~0.961
 * ```
 */
export function jaroWinkler(a: string, b: string): number;
/**
 * Compute the Hamming distance between two SimHash values.
 *
 * Returns the number of differing bits. Lower values = higher similarity.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { simhash, simhashDistance } from 'elid';
 *
 * const hash1 = simhash("iPhone 14");
 * const hash2 = simhash("iPhone 15");
 * const distance = simhashDistance(hash1, hash2);
 *
 * console.log(distance); // Low number = similar
 * ```
 */
export function simhashDistance(hash1: number, hash2: number): number;
/**
 * Encode an embedding with percentage-based compression.
 *
 * The retention percentage (0.0-1.0) controls how much information is preserved:
 * - 1.0 = lossless (Full32 precision, all dimensions)
 * - 0.5 = half precision and/or half dimensions
 * - 0.25 = quarter precision and/or quarter dimensions
 *
 * The algorithm optimizes for dimension reduction first (which preserves
 * more geometric relationships) before reducing precision.
 *
 * # Parameters
 *
 * - `embedding`: Float64 array of embedding values (64-2048 dimensions)
 * - `retention_pct`: Information retention percentage (0.0-1.0)
 *
 * # Returns
 *
 * A base32hex-encoded ELID string.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElidCompressed } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 *
 * // 50% retention - good balance of size and fidelity
 * const elid = encodeElidCompressed(embedding, 0.5);
 *
 * // 25% retention - smaller but less accurate
 * const smallElid = encodeElidCompressed(embedding, 0.25);
 * ```
 */
export function encodeElidCompressed(embedding: Float64Array, retention_pct: number): string;
/**
 * Compute the SimHash fingerprint of a string.
 *
 * Returns a 64-bit hash where similar strings produce similar numbers.
 * Use this for database queries by storing the hash and querying by numeric range.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { simhash } from 'elid';
 *
 * const hash1 = simhash("iPhone 14");
 * const hash2 = simhash("iPhone 15");
 * const hash3 = simhash("Galaxy S23");
 *
 * // hash1 and hash2 will be numerically close
 * // hash3 will be numerically distant
 *
 * // Store in database as bigint:
 * // { name: "iPhone 14", simhash: hash1 }
 * ```
 */
export function simhash(text: string): number;
/**
 * Compute the Jaro similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 * Particularly effective for short strings like names.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { jaro } from 'elid';
 *
 * const similarity = jaro("martha", "marhta");
 * console.log(similarity); // ~0.944
 * ```
 */
export function jaro(a: string, b: string): number;
/**
 * Compute Levenshtein distance with custom options.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { levenshteinWithOpts, SimilarityOptions } from 'elid';
 *
 * const opts = new SimilarityOptions();
 * opts.setCaseSensitive(false);
 * opts.setTrimWhitespace(true);
 *
 * const distance = levenshteinWithOpts("  HELLO  ", "hello", opts);
 * console.log(distance); // 0
 * ```
 */
export function levenshteinWithOpts(a: string, b: string, opts: SimilarityOptions): number;
/**
 * Compute the OSA (Optimal String Alignment) distance between two strings.
 *
 * Similar to Levenshtein but also considers transpositions as a single operation.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { osaDistance } from 'elid';
 *
 * const distance = osaDistance("ca", "ac");
 * console.log(distance); // 1 (transposition)
 * ```
 */
export function osaDistance(a: string, b: string): number;
/**
 * Compute the best matching similarity between two strings.
 *
 * Runs multiple algorithms and returns the highest score.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { bestMatch } from 'elid';
 *
 * const score = bestMatch("hello", "hallo");
 * console.log(score); // ~0.8
 * ```
 */
export function bestMatch(a: string, b: string): number;
/**
 * Find all matches above a threshold score.
 *
 * Returns an array of objects with index and score for all candidates above the threshold.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { findMatchesAboveThreshold } from 'elid';
 *
 * const candidates = ["apple", "application", "apply", "banana"];
 * const matches = findMatchesAboveThreshold("app", candidates, 0.5);
 * console.log(matches); // [{ index: 0, score: 0.907 }, { index: 1, score: 0.830 }, ...]
 * ```
 */
export function findMatchesAboveThreshold(query: string, candidates: string[], threshold: number): any;
/**
 * Find all hashes within a given distance threshold.
 *
 * Useful for database queries - pre-compute hashes, then find similar ones.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { simhash, findSimilarHashes } from 'elid';
 *
 * const candidates = ["iPhone 14 Pro", "iPhone 13", "Galaxy S23"];
 * const hashes = candidates.map(s => simhash(s));
 *
 * const queryHash = simhash("iPhone 14");
 * const matches = findSimilarHashes(queryHash, hashes, 10);
 *
 * console.log(matches); // [0, 1] - indices of similar items
 * ```
 */
export function findSimilarHashes(query_hash: number, candidate_hashes: Float64Array, max_distance: number): Uint32Array;
/**
 * Decode an ELID string to raw bytes.
 *
 * Returns the raw byte representation of an ELID, including the header
 * and payload bytes. Useful for custom processing or debugging.
 *
 * # Parameters
 *
 * - `elid_str`: A valid ELID string (base32hex encoded)
 *
 * # Returns
 *
 * A Uint8Array containing the raw bytes (header + payload).
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { decodeElid } from 'elid';
 *
 * const bytes = decodeElid("012345abcdef...");
 * console.log(bytes); // Uint8Array [...]
 * ```
 */
export function decodeElid(elid_str: string): Uint8Array;
/**
 * Get metadata about a FullVector ELID.
 *
 * Returns an object containing information about how the ELID was encoded,
 * including original dimensions, precision, and dimension mode.
 *
 * # Parameters
 *
 * - `elid_str`: A valid ELID string (base32hex encoded)
 *
 * # Returns
 *
 * An object with metadata fields, or null if not a FullVector ELID.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElidCompressed, getElidMetadata } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 * const elid = encodeElidCompressed(embedding, 0.5);
 *
 * const meta = getElidMetadata(elid);
 * if (meta) {
 *     console.log(meta.originalDims);  // 768
 *     console.log(meta.encodedDims);   // depends on compression
 *     console.log(meta.isLossless);    // false
 * }
 * ```
 */
export function getElidMetadata(elid_str: string): any;
/**
 * Compute the Hamming distance between two strings.
 *
 * Returns the number of positions at which the characters differ.
 * Returns null if strings have different lengths.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { hamming } from 'elid';
 *
 * const distance = hamming("karolin", "kathrin");
 * console.log(distance); // 3
 *
 * const invalid = hamming("hello", "world!");
 * console.log(invalid); // null
 * ```
 */
export function hamming(a: string, b: string): number | undefined;
/**
 * Encode an embedding with a maximum output string length constraint.
 *
 * Calculates the optimal precision and dimension settings to fit within
 * the specified character limit while maximizing fidelity.
 *
 * # Parameters
 *
 * - `embedding`: Float64 array of embedding values (64-2048 dimensions)
 * - `max_chars`: Maximum output string length in characters
 *
 * # Returns
 *
 * A base32hex-encoded ELID string guaranteed to be <= max_chars in length.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElidMaxLength } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 *
 * // Fit in 100 characters (e.g., for database column constraints)
 * const elid = encodeElidMaxLength(embedding, 100);
 * console.log(elid.length <= 100); // true
 *
 * // Fit in 50 characters (more compression)
 * const shortElid = encodeElidMaxLength(embedding, 50);
 * ```
 */
export function encodeElidMaxLength(embedding: Float64Array, max_chars: number): string;
/**
 * Compute the Levenshtein distance between two strings.
 *
 * Returns the minimum number of single-character edits needed to transform one string into another.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { levenshtein } from 'elid';
 *
 * const distance = levenshtein("kitten", "sitting");
 * console.log(distance); // 3
 * ```
 */
export function levenshtein(a: string, b: string): number;
/**
 * Encode an embedding using lossless full vector encoding.
 *
 * Preserves the exact embedding values (32-bit float precision) and all dimensions.
 * This produces the largest output but allows exact reconstruction.
 *
 * # Parameters
 *
 * - `embedding`: Float64 array of embedding values (64-2048 dimensions)
 *
 * # Returns
 *
 * A base32hex-encoded ELID string that can be decoded back to the original embedding.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElidLossless, decodeElidToEmbedding } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 * const elid = encodeElidLossless(embedding);
 *
 * // Later, recover the exact embedding
 * const recovered = decodeElidToEmbedding(elid);
 * // recovered is identical to embedding
 * ```
 */
export function encodeElidLossless(embedding: Float64Array): string;
/**
 * Split an existing Mini128 hash into LSH bands.
 *
 * Takes a 128-bit hash (16 bytes) and splits it into bands for
 * Locality-Sensitive Hashing (LSH) indexing.
 *
 * # Parameters
 *
 * - `hash`: Uint8Array containing exactly 16 bytes (128-bit hash)
 * - `num_bands`: Number of bands to split into (must be 1, 2, 4, 8, or 16)
 *
 * # Returns
 *
 * An array of base32hex-encoded band strings.
 *
 * # Throws
 *
 * Throws an error if the hash is not exactly 16 bytes.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { mini128ToBands, encodeElid, decodeElid, ElidProfile } from 'elid';
 *
 * // Get hash bytes from an existing Mini128 ELID
 * const embedding = new Float64Array(768).fill(0.1);
 * const elid = encodeElid(embedding, ElidProfile.Mini128);
 * const bytes = decodeElid(elid);
 *
 * // Extract the 16-byte hash (skip header byte)
 * const hashBytes = bytes.slice(1, 17);
 *
 * // Split into bands
 * const bands = mini128ToBands(hashBytes, 4);
 * console.log(bands.length); // 4
 * ```
 */
export function mini128ToBands(hash: Uint8Array, num_bands: number): string[];
/**
 * Decode an ELID string back to an embedding vector.
 *
 * Only works for ELIDs encoded with a FullVector profile (lossless,
 * compressed, or max_length). Returns null for non-reversible profiles
 * like Mini128, Morton, or Hilbert.
 *
 * # Parameters
 *
 * - `elid_str`: A valid ELID string (base32hex encoded)
 *
 * # Returns
 *
 * A Float64Array containing the decoded embedding, or null if the ELID
 * is not reversible.
 *
 * Note: If dimension reduction was used during encoding, the decoded
 * embedding will be in the reduced dimension space, not the original.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElidLossless, decodeElidToEmbedding, isElidReversible } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 * const elid = encodeElidLossless(embedding);
 *
 * if (isElidReversible(elid)) {
 *     const recovered = decodeElidToEmbedding(elid);
 *     console.log(recovered.length); // 768
 * }
 * ```
 */
export function decodeElidToEmbedding(elid_str: string): any;
/**
 * Compute the Hamming distance between two ELID strings.
 *
 * Returns the number of differing bits between two Mini128 ELIDs.
 * This distance is proportional to the angular distance between the
 * original embeddings (lower = more similar).
 *
 * # Requirements
 *
 * Both ELIDs must use the Mini128 profile.
 *
 * # Parameters
 *
 * - `elid1`: First ELID string
 * - `elid2`: Second ELID string
 *
 * # Returns
 *
 * Hamming distance (0-128). 0 means identical, 128 means completely different.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElid, elidHammingDistance, ElidProfile } from 'elid';
 *
 * const elid1 = encodeElid(embedding1, ElidProfile.Mini128);
 * const elid2 = encodeElid(embedding2, ElidProfile.Mini128);
 *
 * const distance = elidHammingDistance(elid1, elid2);
 * if (distance < 20) {
 *     console.log("Very similar embeddings!");
 * }
 * ```
 */
export function elidHammingDistance(elid1: string, elid2: string): number;
/**
 * Convert an embedding vector directly to LSH bands.
 *
 * Computes the 128-bit SimHash of the embedding and splits it into bands
 * for Locality-Sensitive Hashing (LSH) indexing in databases.
 *
 * # Parameters
 *
 * - `embedding`: Float64 array of embedding values (64-2048 dimensions)
 * - `num_bands`: Number of bands to split into (must be 1, 2, 4, 8, or 16)
 * - `seed`: Optional seed for deterministic hashing (defaults to standard ELID seed)
 *
 * # Returns
 *
 * An array of base32hex-encoded band strings. Returns an empty array if
 * `num_bands` is invalid.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { embeddingToBands } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 *
 * // Split into 4 bands (32 bits each) - good balance for most use cases
 * const bands = embeddingToBands(embedding, 4);
 * console.log(bands.length); // 4
 *
 * // Store bands in database for efficient OR queries:
 * // SELECT * FROM embeddings WHERE band0 = ? OR band1 = ? OR band2 = ? OR band3 = ?
 *
 * // Use custom seed for different hash family
 * const bandsWithSeed = embeddingToBands(embedding, 4, 12345n);
 * ```
 */
export function embeddingToBands(embedding: Float64Array, num_bands: number, seed?: bigint | null): string[];
/**
 * Compute the normalized SimHash similarity between two strings.
 *
 * Returns a value between 0.0 (completely different) and 1.0 (identical).
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { simhashSimilarity } from 'elid';
 *
 * const similarity = simhashSimilarity("iPhone 14", "iPhone 15");
 * console.log(similarity); // ~0.9 (very similar)
 *
 * const similarity2 = simhashSimilarity("iPhone", "Galaxy");
 * console.log(similarity2); // ~0.4 (different)
 * ```
 */
export function simhashSimilarity(a: string, b: string): number;
/**
 * Find the best match for a query string in an array of candidates.
 *
 * Returns an object with the index and similarity score of the best match.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { findBestMatch } from 'elid';
 *
 * const candidates = ["apple", "application", "apply"];
 * const result = findBestMatch("app", candidates);
 * console.log(result); // { index: 0, score: 0.907 }
 * ```
 */
export function findBestMatch(query: string, candidates: string[]): object;
/**
 * Check if an ELID can be decoded back to an embedding.
 *
 * Returns true if the ELID was encoded with a FullVector profile
 * (lossless, compressed, or max_length), false otherwise.
 *
 * # Parameters
 *
 * - `elid_str`: A valid ELID string (base32hex encoded)
 *
 * # Returns
 *
 * `true` if decodeElidToEmbedding will return an embedding, `false` otherwise.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElid, encodeElidLossless, isElidReversible, ElidProfile } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 *
 * // Mini128 is NOT reversible
 * const mini128Elid = encodeElid(embedding, ElidProfile.Mini128);
 * console.log(isElidReversible(mini128Elid)); // false
 *
 * // Lossless IS reversible
 * const losslessElid = encodeElidLossless(embedding);
 * console.log(isElidReversible(losslessElid)); // true
 * ```
 */
export function isElidReversible(elid_str: string): boolean;
/**
 * Encode an embedding for cross-dimensional comparison.
 *
 * Projects the embedding to a common dimension space, allowing comparison
 * between embeddings of different original dimensions (e.g., 256d vs 768d).
 *
 * # Parameters
 *
 * - `embedding`: Float64 array of embedding values (64-2048 dimensions)
 * - `common_dims`: Target dimension space (all vectors projected here)
 *
 * # Returns
 *
 * A base32hex-encoded ELID string.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { encodeElidCrossDimensional, decodeElidToEmbedding } from 'elid';
 *
 * // Different sized embeddings from different models
 * const embedding256 = new Float64Array(256).fill(0.1);
 * const embedding768 = new Float64Array(768).fill(0.1);
 *
 * // Project both to 128-dim common space
 * const elid1 = encodeElidCrossDimensional(embedding256, 128);
 * const elid2 = encodeElidCrossDimensional(embedding768, 128);
 *
 * // Now they can be compared directly (both decode to 128 dims)
 * const dec1 = decodeElidToEmbedding(elid1);
 * const dec2 = decodeElidToEmbedding(elid2);
 * // Both have length 128
 * ```
 */
export function encodeElidCrossDimensional(embedding: Float64Array, common_dims: number): string;
/**
 * Dimension handling mode for full vector encoding.
 *
 * Controls whether to preserve original dimensions, reduce them,
 * or project to a common space for cross-dimensional comparison.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { ElidDimensionMode, encodeElidFullVector } from 'elid';
 *
 * // Preserve all dimensions
 * // Reduce to fewer dimensions for smaller output
 * // Common space for comparing different-sized embeddings
 * ```
 */
export enum ElidDimensionMode {
  /**
   * Preserve all original dimensions (no projection)
   */
  Preserve = 0,
  /**
   * Reduce dimensions using random projection
   */
  Reduce = 1,
  /**
   * Project to common space for cross-dimensional comparison
   */
  Common = 2,
}
/**
 * ELID encoding profile for vector embeddings.
 *
 * Choose a profile based on your use case:
 * - `Mini128`: Fast 128-bit SimHash, good for similarity via Hamming distance
 * - `Morton10x10`: Z-order curve encoding, good for range queries
 * - `Hilbert10x10`: Hilbert curve encoding, best locality preservation
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { ElidProfile, encodeElid } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 * const elid = encodeElid(embedding, ElidProfile.Mini128);
 * ```
 */
export enum ElidProfile {
  /**
   * 128-bit SimHash (cosine similarity via Hamming distance)
   */
  Mini128 = 0,
  /**
   * Morton/Z-order curve encoding (10 dims, 10 bits each)
   */
  Morton10x10 = 1,
  /**
   * Hilbert curve encoding (10 dims, 10 bits each)
   */
  Hilbert10x10 = 2,
}
/**
 * Precision options for full vector encoding.
 *
 * Controls how many bits are used to represent each dimension value.
 * Higher precision means more accurate reconstruction but larger output.
 *
 * # JavaScript Example
 *
 * ```javascript
 * import { ElidVectorPrecision, encodeElidWithPrecision } from 'elid';
 *
 * const embedding = new Float64Array(768).fill(0.1);
 * // Full32 = lossless, Half16 = smaller with minimal error
 * ```
 */
export enum ElidVectorPrecision {
  /**
   * Full 32-bit float (lossless, 4 bytes per dimension)
   */
  Full32 = 0,
  /**
   * 16-bit half-precision float (2 bytes per dimension)
   */
  Half16 = 1,
  /**
   * 8-bit quantized (1 byte per dimension, ~1% error)
   */
  Quant8 = 2,
}
/**
 * Options for configuring string similarity algorithms
 */
export class SimilarityOptions {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Set prefix scale
   */
  setPrefixScale(value: number): void;
  /**
   * Set case sensitivity
   */
  setCaseSensitive(value: boolean): void;
  /**
   * Set whitespace trimming
   */
  setTrimWhitespace(value: boolean): void;
  /**
   * Create a new SimilarityOptions with default values
   */
  constructor();
  /**
   * Case-sensitive comparison (default: true)
   */
  case_sensitive: boolean;
  /**
   * Trim whitespace before comparison (default: false)
   */
  trim_whitespace: boolean;
  /**
   * Prefix scale for Jaro-Winkler (default: 0.1, max: 0.25)
   */
  prefix_scale: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_get_similarityoptions_case_sensitive: (a: number) => number;
  readonly __wbg_get_similarityoptions_prefix_scale: (a: number) => number;
  readonly __wbg_get_similarityoptions_trim_whitespace: (a: number) => number;
  readonly __wbg_set_similarityoptions_case_sensitive: (a: number, b: number) => void;
  readonly __wbg_set_similarityoptions_prefix_scale: (a: number, b: number) => void;
  readonly __wbg_set_similarityoptions_trim_whitespace: (a: number, b: number) => void;
  readonly __wbg_similarityoptions_free: (a: number, b: number) => void;
  readonly bestMatch: (a: number, b: number, c: number, d: number) => number;
  readonly decodeElid: (a: number, b: number) => [number, number, number, number];
  readonly decodeElidToEmbedding: (a: number, b: number) => [number, number, number];
  readonly elidHammingDistance: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly embeddingToBands: (a: number, b: number, c: number, d: number, e: bigint) => [number, number];
  readonly encodeElid: (a: number, b: number, c: number) => [number, number, number, number];
  readonly encodeElidCompressed: (a: number, b: number, c: number) => [number, number, number, number];
  readonly encodeElidCrossDimensional: (a: number, b: number, c: number) => [number, number, number, number];
  readonly encodeElidLossless: (a: number, b: number) => [number, number, number, number];
  readonly encodeElidMaxLength: (a: number, b: number, c: number) => [number, number, number, number];
  readonly findBestMatch: (a: number, b: number, c: number, d: number) => any;
  readonly findMatchesAboveThreshold: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly findSimilarHashes: (a: number, b: number, c: number, d: number) => [number, number];
  readonly getElidMetadata: (a: number, b: number) => [number, number, number];
  readonly hamming: (a: number, b: number, c: number, d: number) => number;
  readonly isElidReversible: (a: number, b: number) => [number, number, number];
  readonly jaro: (a: number, b: number, c: number, d: number) => number;
  readonly jaroWinkler: (a: number, b: number, c: number, d: number) => number;
  readonly levenshtein: (a: number, b: number, c: number, d: number) => number;
  readonly levenshteinWithOpts: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly mini128ToBands: (a: number, b: number, c: number) => [number, number, number, number];
  readonly normalizedLevenshtein: (a: number, b: number, c: number, d: number) => number;
  readonly osaDistance: (a: number, b: number, c: number, d: number) => number;
  readonly simhash: (a: number, b: number) => number;
  readonly simhashDistance: (a: number, b: number) => number;
  readonly simhashSimilarity: (a: number, b: number, c: number, d: number) => number;
  readonly similarityoptions_new: () => number;
  readonly similarityoptions_setCaseSensitive: (a: number, b: number) => void;
  readonly similarityoptions_setPrefixScale: (a: number, b: number) => void;
  readonly similarityoptions_setTrimWhitespace: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_drop_slice: (a: number, b: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
