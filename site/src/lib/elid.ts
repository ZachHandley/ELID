/**
 * ELID WASM Loader and Typed Wrapper
 *
 * Dynamically imports the WASM module from /wasm/elid.js,
 * calls init() once, and exports typed wrapper functions.
 */

export interface SimilarityOptions {
  case_sensitive: boolean;
  trim_whitespace: boolean;
  prefix_scale: number;
}

export interface MatchResult {
  index: number;
  score: number;
}

/** Encoding profile for embedding vectors */
export enum ElidProfile {
  /** 128-bit SimHash (cosine similarity via Hamming distance) */
  Mini128 = 0,
  /** Morton/Z-order curve encoding (10 dims, 10 bits each) */
  Morton10x10 = 1,
  /** Hilbert curve encoding (10 dims, 10 bits each) */
  Hilbert10x10 = 2,
}

/** Precision options for full vector encoding */
export enum ElidVectorPrecision {
  /** Full 32-bit float (lossless, 4 bytes per dimension) */
  Full32 = 0,
  /** 16-bit half-precision float (2 bytes per dimension) */
  Half16 = 1,
  /** 8-bit quantized (1 byte per dimension, ~1% error) */
  Quant8 = 2,
}

/** Dimension handling mode for full vector encoding */
export enum ElidDimensionMode {
  /** Preserve all original dimensions (no projection) */
  Preserve = 0,
  /** Reduce dimensions using random projection */
  Reduce = 1,
  /** Project to common space for cross-dimensional comparison */
  Common = 2,
}

/** Metadata about a FullVector ELID */
export interface ElidMetadata {
  originalDims: number;
  encodedDims: number;
  isLossless: boolean;
  hasDimensionReduction: boolean;
  precision: "Full32" | "Half16" | "Quant8" | "Bits";
  precisionBits?: number;
  dimensionMode: "Preserve" | "Reduce" | "Common";
}

interface ElidWasmModule {
  default: () => Promise<void>;
  levenshtein: (a: string, b: string) => number;
  osaDistance: (a: string, b: string) => number;
  hamming: (a: string, b: string) => number | null;
  normalizedLevenshtein: (a: string, b: string) => number;
  jaro: (a: string, b: string) => number;
  jaroWinkler: (a: string, b: string) => number;
  bestMatch: (a: string, b: string) => number;
  simhash: (text: string) => number;
  simhashDistance: (hash1: number, hash2: number) => number;
  simhashSimilarity: (a: string, b: string) => number;
  findBestMatch: (
    query: string,
    candidates: string[]
  ) => MatchResult;
  findMatchesAboveThreshold: (
    query: string,
    candidates: string[],
    threshold: number
  ) => MatchResult[];
  findSimilarHashes: (
    queryHash: number,
    candidateHashes: number[],
    maxDistance: number
  ) => number[];
  levenshteinWithOpts: (
    a: string,
    b: string,
    opts: SimilarityOptions
  ) => number;
  // Embedding functions (feature-gated in WASM)
  encodeElid?: (embedding: Float64Array, profile: ElidProfile) => string;
  encodeElidLossless?: (embedding: Float64Array) => string;
  encodeElidCompressed?: (embedding: Float64Array, retentionPct: number) => string;
  encodeElidMaxLength?: (embedding: Float64Array, maxChars: number) => string;
  decodeElidToEmbedding?: (elid: string) => Float64Array | null;
  isElidReversible?: (elid: string) => boolean;
  encodeElidCrossDimensional?: (embedding: Float64Array, commonDims: number) => string;
  elidHammingDistance?: (elid1: string, elid2: string) => number;
  getElidMetadata?: (elid: string) => ElidMetadata | null;
}

let wasmModule: ElidWasmModule | null = null;
let initPromise: Promise<boolean> | null = null;
let initError: string | null = null;

async function loadWasm(): Promise<boolean> {
  try {
    // Construct the URL at runtime so Vite/Rollup does NOT try to resolve
    // this import at build time (the WASM files may not exist yet).
    const base = (import.meta.env.BASE_URL ?? "/").replace(/\/?$/, "/");
    const wasmPath = `${base}wasm/elid.js`;
    const mod = (await import(/* @vite-ignore */ wasmPath)) as ElidWasmModule;
    await mod.default();
    wasmModule = mod;
    return true;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    initError = `Failed to load WASM module: ${message}`;
    console.warn(initError);
    return false;
  }
}

export async function initElid(): Promise<boolean> {
  if (wasmModule) return true;
  if (!initPromise) {
    initPromise = loadWasm();
  }
  return initPromise;
}

export function isReady(): boolean {
  return wasmModule !== null;
}

export function getError(): string | null {
  return initError;
}

/** Check if embedding features are available in the WASM module */
export function hasEmbeddingSupport(): boolean {
  return wasmModule !== null && typeof wasmModule.encodeElid === "function";
}

// --- Distance Functions ---

export function levenshtein(a: string, b: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.levenshtein(a, b);
}

export function osaDistance(a: string, b: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.osaDistance(a, b);
}

export function hamming(a: string, b: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.hamming(a, b);
}

// --- Similarity Functions ---

export function normalizedLevenshtein(a: string, b: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.normalizedLevenshtein(a, b);
}

export function jaro(a: string, b: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.jaro(a, b);
}

export function jaroWinkler(a: string, b: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.jaroWinkler(a, b);
}

export function bestMatch(a: string, b: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.bestMatch(a, b);
}

// --- SimHash ---

export function simhash(text: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.simhash(text);
}

export function simhashDistance(hash1: number, hash2: number): number | null {
  if (!wasmModule) return null;
  return wasmModule.simhashDistance(hash1, hash2);
}

export function simhashSimilarity(a: string, b: string): number | null {
  if (!wasmModule) return null;
  return wasmModule.simhashSimilarity(a, b);
}

// --- Search Functions ---

export function findBestMatch(
  query: string,
  candidates: string[]
): MatchResult | null {
  if (!wasmModule) return null;
  return wasmModule.findBestMatch(query, candidates);
}

export function findMatchesAboveThreshold(
  query: string,
  candidates: string[],
  threshold: number
): MatchResult[] | null {
  if (!wasmModule) return null;
  return wasmModule.findMatchesAboveThreshold(query, candidates, threshold);
}

export function findSimilarHashes(
  queryHash: number,
  candidateHashes: number[],
  maxDistance: number
): number[] | null {
  if (!wasmModule) return null;
  return wasmModule.findSimilarHashes(queryHash, candidateHashes, maxDistance);
}

// --- With Options ---

export function levenshteinWithOpts(
  a: string,
  b: string,
  opts: SimilarityOptions
): number | null {
  if (!wasmModule) return null;
  return wasmModule.levenshteinWithOpts(a, b, opts);
}

// --- Embedding Functions ---

/**
 * Encode an embedding vector to an ELID string using a specified profile.
 * @param embedding Float64Array of embedding values (64-2048 dimensions)
 * @param profile Encoding profile (Mini128, Morton10x10, or Hilbert10x10)
 * @returns Base32hex-encoded ELID string, or null if not available
 */
export function encodeElid(
  embedding: Float64Array,
  profile: ElidProfile
): string | null {
  if (!wasmModule?.encodeElid) return null;
  try {
    return wasmModule.encodeElid(embedding, profile);
  } catch (e) {
    console.error("encodeElid error:", e);
    return null;
  }
}

/**
 * Encode an embedding using lossless full vector encoding.
 * Preserves exact values (32-bit float precision) and all dimensions.
 * @param embedding Float64Array of embedding values
 * @returns Base32hex-encoded ELID string that can be decoded back
 */
export function encodeElidLossless(embedding: Float64Array): string | null {
  if (!wasmModule?.encodeElidLossless) return null;
  try {
    return wasmModule.encodeElidLossless(embedding);
  } catch (e) {
    console.error("encodeElidLossless error:", e);
    return null;
  }
}

/**
 * Encode an embedding with percentage-based compression.
 * @param embedding Float64Array of embedding values
 * @param retentionPct Information retention percentage (0.0-1.0)
 * @returns Base32hex-encoded ELID string
 */
export function encodeElidCompressed(
  embedding: Float64Array,
  retentionPct: number
): string | null {
  if (!wasmModule?.encodeElidCompressed) return null;
  try {
    return wasmModule.encodeElidCompressed(embedding, retentionPct);
  } catch (e) {
    console.error("encodeElidCompressed error:", e);
    return null;
  }
}

/**
 * Encode an embedding with a maximum output string length constraint.
 * @param embedding Float64Array of embedding values
 * @param maxChars Maximum output string length in characters
 * @returns Base32hex-encoded ELID string guaranteed to be <= maxChars
 */
export function encodeElidMaxLength(
  embedding: Float64Array,
  maxChars: number
): string | null {
  if (!wasmModule?.encodeElidMaxLength) return null;
  try {
    return wasmModule.encodeElidMaxLength(embedding, maxChars);
  } catch (e) {
    console.error("encodeElidMaxLength error:", e);
    return null;
  }
}

/**
 * Decode an ELID string back to an embedding vector.
 * Only works for ELIDs encoded with a FullVector profile (lossless, compressed, or max_length).
 * @param elid A valid ELID string (base32hex encoded)
 * @returns Float64Array containing the decoded embedding, or null if not reversible
 */
export function decodeElidToEmbedding(elid: string): Float64Array | null {
  if (!wasmModule?.decodeElidToEmbedding) return null;
  try {
    return wasmModule.decodeElidToEmbedding(elid);
  } catch (e) {
    console.error("decodeElidToEmbedding error:", e);
    return null;
  }
}

/**
 * Check if an ELID can be decoded back to an embedding.
 * @param elid A valid ELID string (base32hex encoded)
 * @returns true if decodeElidToEmbedding will return an embedding
 */
export function isElidReversible(elid: string): boolean | null {
  if (!wasmModule?.isElidReversible) return null;
  try {
    return wasmModule.isElidReversible(elid);
  } catch (e) {
    console.error("isElidReversible error:", e);
    return null;
  }
}

/**
 * Encode an embedding for cross-dimensional comparison.
 * Projects the embedding to a common dimension space.
 * @param embedding Float64Array of embedding values
 * @param commonDims Target dimension space (all vectors projected here)
 * @returns Base32hex-encoded ELID string
 */
export function encodeElidCrossDimensional(
  embedding: Float64Array,
  commonDims: number
): string | null {
  if (!wasmModule?.encodeElidCrossDimensional) return null;
  try {
    return wasmModule.encodeElidCrossDimensional(embedding, commonDims);
  } catch (e) {
    console.error("encodeElidCrossDimensional error:", e);
    return null;
  }
}

/**
 * Compute the Hamming distance between two ELID strings.
 * @param elid1 First ELID string
 * @param elid2 Second ELID string
 * @returns Hamming distance (0-128 for Mini128), or null if error
 */
export function elidHammingDistance(elid1: string, elid2: string): number | null {
  if (!wasmModule?.elidHammingDistance) return null;
  try {
    return wasmModule.elidHammingDistance(elid1, elid2);
  } catch (e) {
    console.error("elidHammingDistance error:", e);
    return null;
  }
}

/**
 * Get metadata about a FullVector ELID.
 * @param elid A valid ELID string (base32hex encoded)
 * @returns Metadata object, or null if not a FullVector ELID
 */
export function getElidMetadata(elid: string): ElidMetadata | null {
  if (!wasmModule?.getElidMetadata) return null;
  try {
    return wasmModule.getElidMetadata(elid);
  } catch (e) {
    console.error("getElidMetadata error:", e);
    return null;
  }
}
