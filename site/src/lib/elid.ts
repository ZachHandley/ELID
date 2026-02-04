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
