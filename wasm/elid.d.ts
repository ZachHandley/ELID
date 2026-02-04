/* tslint:disable */
/* eslint-disable */
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
  readonly findBestMatch: (a: number, b: number, c: number, d: number) => any;
  readonly findMatchesAboveThreshold: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly findSimilarHashes: (a: number, b: number, c: number, d: number) => [number, number];
  readonly hamming: (a: number, b: number, c: number, d: number) => number;
  readonly jaro: (a: number, b: number, c: number, d: number) => number;
  readonly jaroWinkler: (a: number, b: number, c: number, d: number) => number;
  readonly levenshtein: (a: number, b: number, c: number, d: number) => number;
  readonly levenshteinWithOpts: (a: number, b: number, c: number, d: number, e: number) => number;
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
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
