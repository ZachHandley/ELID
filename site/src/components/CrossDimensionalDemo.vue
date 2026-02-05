<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { Icon } from "@iconify/vue";
import {
  initElid,
  isReady,
  getError,
  hasEmbeddingSupport,
  encodeElidCrossDimensional,
  decodeElidToEmbedding,
  elidHammingDistance,
} from "../lib/elid";

interface VectorInfo {
  label: string;
  dims: number;
  color: string;
  embedding: Float64Array | null;
  elid: string | null;
  decodedDims: number | null;
}

const wasmLoading = ref(true);
const wasmError = ref<string | null>(null);
const mounted = ref(false);
const hasEmbeddings = ref(false);

// Model presets
const MODEL_PRESETS = [
  { name: "OpenAI ada-002", dims: 1536 },
  { name: "OpenAI text-3-small", dims: 1536 },
  { name: "OpenAI text-3-large", dims: 3072 },
  { name: "Cohere embed-v3", dims: 1024 },
  { name: "Voyage-2", dims: 1024 },
  { name: "BGE-small", dims: 384 },
  { name: "BGE-base", dims: 768 },
  { name: "BGE-large", dims: 1024 },
  { name: "E5-small", dims: 384 },
  { name: "E5-base", dims: 768 },
  { name: "E5-large", dims: 1024 },
  { name: "Sentence-BERT", dims: 768 },
  { name: "MiniLM-L6", dims: 384 },
  { name: "Custom", dims: 512 },
];

// State
const vectorA = ref<VectorInfo>({
  label: "Vector A",
  dims: 256,
  color: "elid-accent",
  embedding: null,
  elid: null,
  decodedDims: null,
});

const vectorB = ref<VectorInfo>({
  label: "Vector B",
  dims: 768,
  color: "elid-green",
  embedding: null,
  elid: null,
  decodedDims: null,
});

const commonDims = ref(128);
const hammingDist = ref<number | null>(null);
const similarityScore = ref<number | null>(null);

// Seeded random for reproducible vectors
const BASE_DIMS = 2048; // Match ELID's MAX_COMMON_SOURCE_DIMS
let baseSeed = 42;
let cachedBaseEmbedding: Float64Array | null = null;

function seededRandom(seed: number): { value: number; nextSeed: number } {
  const newSeed = (seed * 1103515245 + 12345) & 0x7fffffff;
  return { value: newSeed / 0x7fffffff, nextSeed: newSeed };
}

function generateBaseEmbedding(seed: number): Float64Array {
  const arr = new Float64Array(BASE_DIMS);
  let currentSeed = seed;

  for (let i = 0; i < BASE_DIMS; i++) {
    const { value, nextSeed } = seededRandom(currentSeed);
    arr[i] = (value - 0.5) * 2;
    currentSeed = nextSeed;
  }

  // Normalize to unit length
  let norm = 0;
  for (let i = 0; i < BASE_DIMS; i++) norm += arr[i] * arr[i];
  norm = Math.sqrt(norm);
  for (let i = 0; i < BASE_DIMS; i++) arr[i] /= norm;

  return arr;
}

function deriveVector(baseEmbedding: Float64Array, targetDims: number): Float64Array {
  // Truncate to first targetDims (Matryoshka-style)
  const truncated = new Float64Array(targetDims);
  for (let i = 0; i < targetDims; i++) {
    truncated[i] = baseEmbedding[i];
  }

  // Renormalize (critical for correct similarity)
  let norm = 0;
  for (let i = 0; i < targetDims; i++) norm += truncated[i] * truncated[i];
  norm = Math.sqrt(norm);
  if (norm > 0) {
    for (let i = 0; i < targetDims; i++) truncated[i] /= norm;
  }

  return truncated;
}

function encodeVector(vector: VectorInfo): void {
  if (!isReady() || !hasEmbeddings.value) return;

  // Generate base once, derive both vectors from it
  if (!cachedBaseEmbedding) {
    cachedBaseEmbedding = generateBaseEmbedding(baseSeed);
  }

  const derived = deriveVector(cachedBaseEmbedding, vector.dims);
  vector.embedding = derived;

  const elid = encodeElidCrossDimensional(derived, commonDims.value);
  vector.elid = elid;

  if (elid) {
    const decoded = decodeElidToEmbedding(elid);
    vector.decodedDims = decoded ? decoded.length : null;
  }
}

function computeComparison(): void {
  if (!vectorA.value.elid || !vectorB.value.elid) {
    hammingDist.value = null;
    similarityScore.value = null;
    return;
  }

  const dist = elidHammingDistance(vectorA.value.elid, vectorB.value.elid);
  hammingDist.value = dist;

  // For cross-dimensional comparison, similarity is based on the common dimension space
  // We use cosine similarity of the decoded vectors
  if (vectorA.value.elid && vectorB.value.elid) {
    const decodedA = decodeElidToEmbedding(vectorA.value.elid);
    const decodedB = decodeElidToEmbedding(vectorB.value.elid);

    if (decodedA && decodedB && decodedA.length === decodedB.length) {
      let dotProduct = 0;
      let normA = 0;
      let normB = 0;
      for (let i = 0; i < decodedA.length; i++) {
        dotProduct += decodedA[i] * decodedB[i];
        normA += decodedA[i] * decodedA[i];
        normB += decodedB[i] * decodedB[i];
      }
      const cosineSim = dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
      similarityScore.value = Math.max(0, Math.min(1, (cosineSim + 1) / 2)); // Normalize to 0-1
    }
  }
}

function regenerateVectors(): void {
  baseSeed = Date.now();
  cachedBaseEmbedding = null; // Clear cache to regenerate
  updateVectors();
}

function updateVectors(): void {
  cachedBaseEmbedding = null; // Clear when dimensions change
  encodeVector(vectorA.value);
  encodeVector(vectorB.value);
  computeComparison();
}

function scoreColor(score: number | null): string {
  if (score === null) return "text-elid-text-muted";
  if (score >= 0.7) return "text-elid-green";
  if (score >= 0.4) return "text-elid-yellow";
  return "text-elid-red";
}

function scoreBgColor(score: number | null): string {
  if (score === null) return "bg-elid-surface-alt";
  if (score >= 0.7) return "bg-elid-green";
  if (score >= 0.4) return "bg-elid-yellow";
  return "bg-elid-red";
}

// Watch for dimension changes
watch([() => vectorA.value.dims, () => vectorB.value.dims, commonDims], () => {
  updateVectors();
});

onMounted(async () => {
  mounted.value = true;
  wasmLoading.value = true;
  const success = await initElid();
  wasmLoading.value = false;

  if (!success) {
    wasmError.value = getError();
    return;
  }

  hasEmbeddings.value = hasEmbeddingSupport();
  if (hasEmbeddings.value) {
    updateVectors();
  }
});
</script>

<template>
  <div class="space-y-6">
    <!-- WASM Loading -->
    <div
      v-if="wasmLoading && mounted"
      class="flex items-center justify-center gap-3 py-12 text-elid-text-muted"
    >
      <Icon icon="mdi:loading" class="w-6 h-6 animate-spin" />
      <span class="text-lg">Loading WASM module...</span>
    </div>

    <!-- WASM Error -->
    <div
      v-else-if="wasmError && mounted"
      class="bg-elid-red/10 border border-elid-red/30 rounded-lg p-6 text-center"
    >
      <Icon icon="mdi:alert-circle" class="w-8 h-8 text-elid-red mx-auto mb-3" />
      <p class="text-elid-red font-medium mb-2">WASM module not available</p>
    </div>

    <!-- No Embedding Support -->
    <div
      v-else-if="mounted && !hasEmbeddings"
      class="bg-elid-yellow/10 border border-elid-yellow/30 rounded-lg p-6 text-center"
    >
      <Icon icon="mdi:package-variant" class="w-8 h-8 text-elid-yellow mx-auto mb-3" />
      <p class="text-elid-yellow font-medium mb-2">Embedding features not available</p>
      <p class="text-sm text-elid-text-muted">
        Rebuild WASM with <code class="font-mono bg-elid-surface px-1.5 py-0.5 rounded">--features embeddings</code>
      </p>
    </div>

    <!-- Main Content -->
    <template v-else-if="mounted">
      <!-- Common Dimension Control -->
      <div class="bg-elid-surface border border-elid-border/50 rounded-lg p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="font-medium flex items-center gap-2">
            <Icon icon="mdi:resize" class="w-5 h-5 text-elid-accent" />
            Common Dimension Space
          </h3>
          <button
            @click="regenerateVectors"
            class="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-elid-surface-alt border border-elid-border rounded-lg hover:border-elid-accent transition-colors"
          >
            <Icon icon="mdi:refresh" class="w-4 h-4" />
            Regenerate
          </button>
        </div>
        <div class="flex items-center gap-4">
          <label class="text-sm text-elid-text-muted whitespace-nowrap">
            Project to:
            <span class="font-mono text-elid-accent">{{ commonDims }}</span> dimensions
          </label>
          <input
            v-model.number="commonDims"
            type="range"
            min="32"
            max="512"
            step="32"
            class="flex-1 h-2 bg-elid-bg rounded-full appearance-none cursor-pointer accent-elid-accent"
          />
        </div>
        <p class="mt-2 text-xs text-elid-text-muted">
          Both vectors are projected to this common dimension space, enabling direct comparison
          regardless of their original sizes.
        </p>
      </div>

      <!-- Vector Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Vector A -->
        <div class="bg-elid-surface border border-elid-accent/30 rounded-lg p-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="font-medium flex items-center gap-2">
              <div class="w-3 h-3 rounded-full bg-elid-accent"></div>
              {{ vectorA.label }}
            </h3>
            <span class="text-sm font-mono text-elid-accent">{{ vectorA.dims }}d</span>
          </div>

          <div class="mb-3">
            <label class="text-xs text-elid-text-muted mb-1 block">Original dimensions:</label>
            <select
              v-model.number="vectorA.dims"
              class="w-full px-3 py-2 bg-elid-bg border border-elid-border rounded-lg text-sm focus:outline-none focus:border-elid-accent"
            >
              <option v-for="preset in MODEL_PRESETS" :key="preset.name + '-a'" :value="preset.dims">
                {{ preset.name }} ({{ preset.dims }}d)
              </option>
            </select>
          </div>

          <div v-if="vectorA.elid" class="space-y-2">
            <div class="text-xs text-elid-text-muted">
              Encoded to <span class="font-mono text-elid-accent">{{ vectorA.decodedDims }}d</span> common space
            </div>
            <code class="block p-2 bg-elid-bg border border-elid-border/30 rounded text-[10px] font-mono text-elid-text-muted truncate">
              {{ vectorA.elid.substring(0, 50) }}...
            </code>
          </div>
        </div>

        <!-- Vector B -->
        <div class="bg-elid-surface border border-elid-green/30 rounded-lg p-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="font-medium flex items-center gap-2">
              <div class="w-3 h-3 rounded-full bg-elid-green"></div>
              {{ vectorB.label }}
            </h3>
            <span class="text-sm font-mono text-elid-green">{{ vectorB.dims }}d</span>
          </div>

          <div class="mb-3">
            <label class="text-xs text-elid-text-muted mb-1 block">Original dimensions:</label>
            <select
              v-model.number="vectorB.dims"
              class="w-full px-3 py-2 bg-elid-bg border border-elid-border rounded-lg text-sm focus:outline-none focus:border-elid-accent"
            >
              <option v-for="preset in MODEL_PRESETS" :key="preset.name + '-b'" :value="preset.dims">
                {{ preset.name }} ({{ preset.dims }}d)
              </option>
            </select>
          </div>

          <div v-if="vectorB.elid" class="space-y-2">
            <div class="text-xs text-elid-text-muted">
              Encoded to <span class="font-mono text-elid-green">{{ vectorB.decodedDims }}d</span> common space
            </div>
            <code class="block p-2 bg-elid-bg border border-elid-border/30 rounded text-[10px] font-mono text-elid-text-muted truncate">
              {{ vectorB.elid.substring(0, 50) }}...
            </code>
          </div>
        </div>
      </div>

      <!-- Comparison Result -->
      <div class="bg-elid-accent/5 border border-elid-accent/30 rounded-lg p-5">
        <div class="flex items-center gap-3 mb-4">
          <Icon icon="mdi:compare-horizontal" class="w-6 h-6 text-elid-accent" />
          <h3 class="font-semibold text-elid-accent">Cross-Dimensional Comparison</h3>
        </div>

        <div class="grid grid-cols-2 gap-6">
          <div>
            <div class="text-sm text-elid-text-muted mb-1">Cosine Similarity</div>
            <div class="flex items-baseline gap-2">
              <span
                class="text-3xl font-bold font-mono"
                :class="scoreColor(similarityScore)"
              >
                {{ similarityScore !== null ? (similarityScore * 100).toFixed(1) + "%" : "--" }}
              </span>
            </div>
            <div class="mt-2 h-2 bg-elid-bg rounded-full overflow-hidden">
              <div
                class="h-full rounded-full transition-all duration-300"
                :class="scoreBgColor(similarityScore)"
                :style="{ width: `${(similarityScore ?? 0) * 100}%` }"
              ></div>
            </div>
          </div>

          <div>
            <div class="text-sm text-elid-text-muted mb-1">Dimension Comparison</div>
            <div class="flex items-center gap-3 mt-2">
              <div class="flex items-center gap-1.5">
                <div class="w-2.5 h-2.5 rounded-full bg-elid-accent"></div>
                <span class="font-mono text-sm">{{ vectorA.dims }}d</span>
              </div>
              <Icon icon="mdi:arrow-right" class="w-4 h-4 text-elid-text-muted" />
              <span class="font-mono text-sm text-elid-accent">{{ commonDims }}d</span>
              <Icon icon="mdi:arrow-left" class="w-4 h-4 text-elid-text-muted" />
              <div class="flex items-center gap-1.5">
                <div class="w-2.5 h-2.5 rounded-full bg-elid-green"></div>
                <span class="font-mono text-sm">{{ vectorB.dims }}d</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Explanation -->
      <div class="bg-elid-surface border border-elid-border/50 rounded-lg p-4">
        <h4 class="font-medium mb-2 flex items-center gap-2">
          <Icon icon="mdi:lightbulb-outline" class="w-5 h-5 text-elid-yellow" />
          How Cross-Dimensional Comparison Works
        </h4>
        <div class="text-sm text-elid-text-muted space-y-2">
          <p>
            This demo simulates <strong class="text-elid-text">Matryoshka Representation Learning</strong> -
            modern embedding models where early dimensions encode the most semantic information.
            Both vectors are derived from the same 2048-dimensional base by truncation.
          </p>
          <p>
            ELID's cross-dimensional encoding projects both vectors to a common space
            using a <strong class="text-elid-text">Johnson-Lindenstrauss random projection</strong>, enabling
            direct comparison regardless of original dimensions.
          </p>
          <p class="text-elid-accent">
            Use case: Compare embeddings from OpenAI (1536d) and Cohere (1024d) in the same index,
            or reduce dimensions for storage while preserving searchability.
          </p>
        </div>
      </div>
    </template>
  </div>
</template>
