let wasm;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

let cachedFloat64ArrayMemory0 = null;

function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
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
 * @param {Float64Array} embedding
 * @param {ElidProfile} profile
 * @returns {string}
 */
export function encodeElid(embedding, profile) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passArrayF64ToWasm0(embedding, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.encodeElid(ptr0, len0, profile);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

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
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function normalizedLevenshtein(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.normalizedLevenshtein(ptr0, len0, ptr1, len1);
    return ret;
}

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
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function jaroWinkler(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.jaroWinkler(ptr0, len0, ptr1, len1);
    return ret;
}

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
 * @param {number} hash1
 * @param {number} hash2
 * @returns {number}
 */
export function simhashDistance(hash1, hash2) {
    const ret = wasm.simhashDistance(hash1, hash2);
    return ret >>> 0;
}

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
 * @param {Float64Array} embedding
 * @param {number} retention_pct
 * @returns {string}
 */
export function encodeElidCompressed(embedding, retention_pct) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passArrayF64ToWasm0(embedding, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.encodeElidCompressed(ptr0, len0, retention_pct);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

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
 * @param {string} text
 * @returns {number}
 */
export function simhash(text) {
    const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.simhash(ptr0, len0);
    return ret;
}

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
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function jaro(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.jaro(ptr0, len0, ptr1, len1);
    return ret;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}
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
 * @param {string} a
 * @param {string} b
 * @param {SimilarityOptions} opts
 * @returns {number}
 */
export function levenshteinWithOpts(a, b, opts) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    _assertClass(opts, SimilarityOptions);
    var ptr2 = opts.__destroy_into_raw();
    const ret = wasm.levenshteinWithOpts(ptr0, len0, ptr1, len1, ptr2);
    return ret >>> 0;
}

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
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function osaDistance(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.osaDistance(ptr0, len0, ptr1, len1);
    return ret >>> 0;
}

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
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function bestMatch(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bestMatch(ptr0, len0, ptr1, len1);
    return ret;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
        const add = addToExternrefTable0(array[i]);
        getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}
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
 * @param {string} query
 * @param {string[]} candidates
 * @param {number} threshold
 * @returns {any}
 */
export function findMatchesAboveThreshold(query, candidates, threshold) {
    const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.findMatchesAboveThreshold(ptr0, len0, ptr1, len1, threshold);
    return ret;
}

let cachedUint32ArrayMemory0 = null;

function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}
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
 * @param {number} query_hash
 * @param {Float64Array} candidate_hashes
 * @param {number} max_distance
 * @returns {Uint32Array}
 */
export function findSimilarHashes(query_hash, candidate_hashes, max_distance) {
    const ptr0 = passArrayF64ToWasm0(candidate_hashes, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.findSimilarHashes(query_hash, ptr0, len0, max_distance);
    var v2 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v2;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
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
 * @param {string} elid_str
 * @returns {Uint8Array}
 */
export function decodeElid(elid_str) {
    const ptr0 = passStringToWasm0(elid_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.decodeElid(ptr0, len0);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
    return v2;
}

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
 * @param {string} a
 * @param {string} b
 * @returns {number | undefined}
 */
export function hamming(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.hamming(ptr0, len0, ptr1, len1);
    return ret === 0x100000001 ? undefined : ret;
}

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
 * @param {Float64Array} embedding
 * @param {number} max_chars
 * @returns {string}
 */
export function encodeElidMaxLength(embedding, max_chars) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passArrayF64ToWasm0(embedding, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.encodeElidMaxLength(ptr0, len0, max_chars);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

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
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function levenshtein(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.levenshtein(ptr0, len0, ptr1, len1);
    return ret >>> 0;
}

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
 * @param {Float64Array} embedding
 * @returns {string}
 */
export function encodeElidLossless(embedding) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passArrayF64ToWasm0(embedding, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.encodeElidLossless(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

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
 * @param {string} elid_str
 * @returns {any}
 */
export function decodeElidToEmbedding(elid_str) {
    const ptr0 = passStringToWasm0(elid_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.decodeElidToEmbedding(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

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
 * @param {string} elid1
 * @param {string} elid2
 * @returns {number}
 */
export function elidHammingDistance(elid1, elid2) {
    const ptr0 = passStringToWasm0(elid1, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(elid2, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.elidHammingDistance(ptr0, len0, ptr1, len1);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0] >>> 0;
}

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
 * @param {string} elid_str
 * @returns {any}
 */
export function getElidMetadata(elid_str) {
    const ptr0 = passStringToWasm0(elid_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.getElidMetadata(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

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
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function simhashSimilarity(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.simhashSimilarity(ptr0, len0, ptr1, len1);
    return ret;
}

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
 * @param {string} query
 * @param {string[]} candidates
 * @returns {object}
 */
export function findBestMatch(query, candidates) {
    const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayJsValueToWasm0(candidates, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.findBestMatch(ptr0, len0, ptr1, len1);
    return ret;
}

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
 * @param {string} elid_str
 * @returns {boolean}
 */
export function isElidReversible(elid_str) {
    const ptr0 = passStringToWasm0(elid_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.isElidReversible(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0] !== 0;
}

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
 * @param {Float64Array} embedding
 * @param {number} common_dims
 * @returns {string}
 */
export function encodeElidCrossDimensional(embedding, common_dims) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passArrayF64ToWasm0(embedding, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.encodeElidCrossDimensional(ptr0, len0, common_dims);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

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
 * @enum {0 | 1 | 2}
 */
export const ElidDimensionMode = Object.freeze({
    /**
     * Preserve all original dimensions (no projection)
     */
    Preserve: 0, "0": "Preserve",
    /**
     * Reduce dimensions using random projection
     */
    Reduce: 1, "1": "Reduce",
    /**
     * Project to common space for cross-dimensional comparison
     */
    Common: 2, "2": "Common",
});
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
 * @enum {0 | 1 | 2}
 */
export const ElidProfile = Object.freeze({
    /**
     * 128-bit SimHash (cosine similarity via Hamming distance)
     */
    Mini128: 0, "0": "Mini128",
    /**
     * Morton/Z-order curve encoding (10 dims, 10 bits each)
     */
    Morton10x10: 1, "1": "Morton10x10",
    /**
     * Hilbert curve encoding (10 dims, 10 bits each)
     */
    Hilbert10x10: 2, "2": "Hilbert10x10",
});
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
 * @enum {0 | 1 | 2}
 */
export const ElidVectorPrecision = Object.freeze({
    /**
     * Full 32-bit float (lossless, 4 bytes per dimension)
     */
    Full32: 0, "0": "Full32",
    /**
     * 16-bit half-precision float (2 bytes per dimension)
     */
    Half16: 1, "1": "Half16",
    /**
     * 8-bit quantized (1 byte per dimension, ~1% error)
     */
    Quant8: 2, "2": "Quant8",
});

const SimilarityOptionsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_similarityoptions_free(ptr >>> 0, 1));
/**
 * Options for configuring string similarity algorithms
 */
export class SimilarityOptions {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SimilarityOptionsFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_similarityoptions_free(ptr, 0);
    }
    /**
     * Set prefix scale
     * @param {number} value
     */
    setPrefixScale(value) {
        wasm.similarityoptions_setPrefixScale(this.__wbg_ptr, value);
    }
    /**
     * Set case sensitivity
     * @param {boolean} value
     */
    setCaseSensitive(value) {
        wasm.similarityoptions_setCaseSensitive(this.__wbg_ptr, value);
    }
    /**
     * Set whitespace trimming
     * @param {boolean} value
     */
    setTrimWhitespace(value) {
        wasm.similarityoptions_setTrimWhitespace(this.__wbg_ptr, value);
    }
    /**
     * Create a new SimilarityOptions with default values
     */
    constructor() {
        const ret = wasm.similarityoptions_new();
        this.__wbg_ptr = ret >>> 0;
        SimilarityOptionsFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Case-sensitive comparison (default: true)
     * @returns {boolean}
     */
    get case_sensitive() {
        const ret = wasm.__wbg_get_similarityoptions_case_sensitive(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Case-sensitive comparison (default: true)
     * @param {boolean} arg0
     */
    set case_sensitive(arg0) {
        wasm.__wbg_set_similarityoptions_case_sensitive(this.__wbg_ptr, arg0);
    }
    /**
     * Trim whitespace before comparison (default: false)
     * @returns {boolean}
     */
    get trim_whitespace() {
        const ret = wasm.__wbg_get_similarityoptions_trim_whitespace(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Trim whitespace before comparison (default: false)
     * @param {boolean} arg0
     */
    set trim_whitespace(arg0) {
        wasm.__wbg_set_similarityoptions_trim_whitespace(this.__wbg_ptr, arg0);
    }
    /**
     * Prefix scale for Jaro-Winkler (default: 0.1, max: 0.25)
     * @returns {number}
     */
    get prefix_scale() {
        const ret = wasm.__wbg_get_similarityoptions_prefix_scale(this.__wbg_ptr);
        return ret;
    }
    /**
     * Prefix scale for Jaro-Winkler (default: 0.1, max: 0.25)
     * @param {number} arg0
     */
    set prefix_scale(arg0) {
        wasm.__wbg_set_similarityoptions_prefix_scale(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) SimilarityOptions.prototype[Symbol.dispose] = SimilarityOptions.prototype.free;

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_Error_e83987f665cf5504 = function(arg0, arg1) {
        const ret = Error(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg___wbindgen_debug_string_df47ffb5e35e6763 = function(arg0, arg1) {
        const ret = debugString(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg___wbindgen_is_string_fbb76cb2940daafd = function(arg0) {
        const ret = typeof(arg0) === 'string';
        return ret;
    };
    imports.wbg.__wbg___wbindgen_string_get_e4f06c90489ad01b = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg___wbindgen_throw_b855445ff6a94295 = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_new_1acc0b6eea89d040 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_68651c719dcda04e = function() {
        const ret = new Map();
        return ret;
    };
    imports.wbg.__wbg_new_e17d9f43105b08be = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_new_with_length_cd045ed0a87d4dd6 = function(arg0) {
        const ret = new Float64Array(arg0 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
        arg0[arg1] = arg2;
    };
    imports.wbg.__wbg_set_907fb406c34a251d = function(arg0, arg1, arg2) {
        const ret = arg0.set(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_set_c213c871859d6500 = function(arg0, arg1, arg2) {
        arg0[arg1 >>> 0] = arg2;
    };
    imports.wbg.__wbg_set_c2abbebe8b9ebee1 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_set_index_a0c01b257dd824f8 = function(arg0, arg1, arg2) {
        arg0[arg1 >>> 0] = arg2;
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_4625c577ab2ec9ee = function(arg0) {
        // Cast intrinsic for `U64 -> Externref`.
        const ret = BigInt.asUintN(64, arg0);
        return ret;
    };
    imports.wbg.__wbindgen_cast_9ae0607507abb057 = function(arg0) {
        // Cast intrinsic for `I64 -> Externref`.
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
        // Cast intrinsic for `F64 -> Externref`.
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('elid_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
