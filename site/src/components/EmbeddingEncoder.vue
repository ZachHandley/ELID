<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { Icon } from "@iconify/vue";
import {
  initElid,
  isReady,
  getError,
  hasEmbeddingSupport,
  encodeElid,
  encodeElidLossless,
  encodeElidCompressed,
  encodeElidMaxLength,
  decodeElidToEmbedding,
  isElidReversible,
  getElidMetadata,
  ElidProfile,
  type ElidMetadata,
} from "../lib/elid";

type ProfileType = "Mini128" | "Morton10x10" | "Hilbert10x10" | "Lossless" | "Compressed" | "MaxLength";

interface ProfileInfo {
  name: ProfileType;
  description: string;
  reversible: boolean;
  icon: string;
}

const PROFILES: ProfileInfo[] = [
  {
    name: "Mini128",
    description: "128-bit SimHash for fast similarity via Hamming distance",
    reversible: false,
    icon: "mdi:lightning-bolt",
  },
  {
    name: "Morton10x10",
    description: "Z-order curve encoding, good for range queries",
    reversible: false,
    icon: "mdi:grid",
  },
  {
    name: "Hilbert10x10",
    description: "Hilbert curve encoding, best locality preservation",
    reversible: false,
    icon: "mdi:chart-line-variant",
  },
  {
    name: "Lossless",
    description: "Full 32-bit precision, all dimensions preserved",
    reversible: true,
    icon: "mdi:lock",
  },
  {
    name: "Compressed",
    description: "Percentage-based compression with dimension/precision trade-off",
    reversible: true,
    icon: "mdi:zip-box",
  },
  {
    name: "MaxLength",
    description: "Fit embedding into a maximum character limit",
    reversible: true,
    icon: "mdi:text-short",
  },
];

const wasmLoading = ref(true);
const wasmError = ref<string | null>(null);
const embeddingError = ref<string | null>(null);
const mounted = ref(false);
const hasEmbeddings = ref(false);

// Input state
const inputMode = ref<"paste" | "random">("random");
const pastedInput = ref("");
const randomDims = ref(768);
const selectedProfile = ref<ProfileType>("Mini128");
const compressionPct = ref(0.5);
const maxLength = ref(100);

// Output state
const encodedElid = ref<string | null>(null);
const metadata = ref<ElidMetadata | null>(null);
const decodedVector = ref<Float64Array | null>(null);
const showDecoded = ref(false);

// Computed
const currentEmbedding = computed<Float64Array | null>(() => {
  if (inputMode.value === "random") {
    return generateRandomEmbedding(randomDims.value);
  }

  const parsed = parseEmbeddingInput(pastedInput.value);
  if (parsed) {
    embeddingError.value = null;
    return parsed;
  }
  return null;
});

const currentProfileInfo = computed<ProfileInfo | undefined>(() => {
  return PROFILES.find((p) => p.name === selectedProfile.value);
});

const canDecode = computed<boolean>(() => {
  if (!encodedElid.value) return false;
  const reversible = isElidReversible(encodedElid.value);
  return reversible === true;
});

// Seeded random for consistent results per dimension
let randomSeed = 12345;
function seededRandom(): number {
  randomSeed = (randomSeed * 1103515245 + 12345) & 0x7fffffff;
  return randomSeed / 0x7fffffff;
}

function generateRandomEmbedding(dims: number): Float64Array {
  randomSeed = dims; // Reset seed based on dims for consistency
  const arr = new Float64Array(dims);
  for (let i = 0; i < dims; i++) {
    // Generate values roughly in [-1, 1] range like real embeddings
    arr[i] = (seededRandom() - 0.5) * 2;
  }
  // Normalize to unit vector
  let norm = 0;
  for (let i = 0; i < dims; i++) {
    norm += arr[i] * arr[i];
  }
  norm = Math.sqrt(norm);
  if (norm > 0) {
    for (let i = 0; i < dims; i++) {
      arr[i] /= norm;
    }
  }
  return arr;
}

function parseEmbeddingInput(input: string): Float64Array | null {
  if (!input.trim()) {
    embeddingError.value = null;
    return null;
  }

  try {
    // Handle JSON array format
    let values: number[];
    const trimmed = input.trim();
    if (trimmed.startsWith("[")) {
      values = JSON.parse(trimmed);
    } else {
      // Comma or whitespace separated
      values = trimmed
        .split(/[\s,]+/)
        .filter((v) => v.length > 0)
        .map((v) => {
          const num = parseFloat(v);
          if (isNaN(num)) throw new Error(`Invalid number: ${v}`);
          return num;
        });
    }

    if (!Array.isArray(values) || values.length === 0) {
      throw new Error("No valid numbers found");
    }
    if (values.length < 64) {
      throw new Error(`Need at least 64 dimensions, got ${values.length}`);
    }
    if (values.length > 2048) {
      throw new Error(`Maximum 2048 dimensions, got ${values.length}`);
    }

    embeddingError.value = null;
    return new Float64Array(values);
  } catch (e) {
    embeddingError.value = e instanceof Error ? e.message : "Failed to parse embedding";
    return null;
  }
}

function encodeCurrentEmbedding(): void {
  const embedding = currentEmbedding.value;
  if (!embedding || !isReady() || !hasEmbeddings.value) {
    encodedElid.value = null;
    metadata.value = null;
    decodedVector.value = null;
    return;
  }

  let result: string | null = null;

  switch (selectedProfile.value) {
    case "Mini128":
      result = encodeElid(embedding, ElidProfile.Mini128);
      break;
    case "Morton10x10":
      result = encodeElid(embedding, ElidProfile.Morton10x10);
      break;
    case "Hilbert10x10":
      result = encodeElid(embedding, ElidProfile.Hilbert10x10);
      break;
    case "Lossless":
      result = encodeElidLossless(embedding);
      break;
    case "Compressed":
      result = encodeElidCompressed(embedding, compressionPct.value);
      break;
    case "MaxLength":
      result = encodeElidMaxLength(embedding, maxLength.value);
      break;
  }

  encodedElid.value = result;
  showDecoded.value = false;
  decodedVector.value = null;

  // Get metadata for reversible profiles
  if (result) {
    metadata.value = getElidMetadata(result);
  } else {
    metadata.value = null;
  }
}

function decodeElid(): void {
  if (!encodedElid.value || !canDecode.value) return;
  decodedVector.value = decodeElidToEmbedding(encodedElid.value);
  showDecoded.value = true;
}

function copyToClipboard(): void {
  if (encodedElid.value) {
    navigator.clipboard.writeText(encodedElid.value);
  }
}

function regenerateRandom(): void {
  // Change seed to get different values
  randomSeed = Date.now();
  encodeCurrentEmbedding();
}

// Watch for changes and re-encode
watch(
  [currentEmbedding, selectedProfile, compressionPct, maxLength],
  () => {
    encodeCurrentEmbedding();
  },
  { immediate: false }
);

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
    encodeCurrentEmbedding();
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
      <p class="text-sm text-elid-text-muted">
        Build the library with <code class="font-mono bg-elid-surface px-1.5 py-0.5 rounded">wasm-pack build</code> first.
      </p>
    </div>

    <!-- No Embedding Support -->
    <div
      v-else-if="mounted && !hasEmbeddings"
      class="bg-elid-yellow/10 border border-elid-yellow/30 rounded-lg p-6 text-center"
    >
      <Icon icon="mdi:package-variant" class="w-8 h-8 text-elid-yellow mx-auto mb-3" />
      <p class="text-elid-yellow font-medium mb-2">Embedding features not available</p>
      <p class="text-sm text-elid-text-muted">
        The WASM module was built without the <code class="font-mono bg-elid-surface px-1.5 py-0.5 rounded">embeddings</code> feature.
        <br />
        Rebuild with: <code class="font-mono bg-elid-surface px-1.5 py-0.5 rounded">wasm-pack build --features embeddings</code>
      </p>
    </div>

    <!-- Main Content -->
    <template v-else-if="mounted">
      <!-- Input Section -->
      <div class="space-y-4">
        <!-- Input Mode Toggle -->
        <div class="flex items-center gap-4">
          <span class="text-sm text-elid-text-muted">Input:</span>
          <div class="flex rounded-lg overflow-hidden border border-elid-border">
            <button
              @click="inputMode = 'random'"
              class="px-4 py-2 text-sm font-medium transition-colors"
              :class="
                inputMode === 'random'
                  ? 'bg-elid-accent text-elid-bg'
                  : 'bg-elid-surface text-elid-text-muted hover:text-elid-text'
              "
            >
              Random Vector
            </button>
            <button
              @click="inputMode = 'paste'"
              class="px-4 py-2 text-sm font-medium transition-colors"
              :class="
                inputMode === 'paste'
                  ? 'bg-elid-accent text-elid-bg'
                  : 'bg-elid-surface text-elid-text-muted hover:text-elid-text'
              "
            >
              Paste Embedding
            </button>
          </div>
        </div>

        <!-- Random Vector Controls -->
        <div v-if="inputMode === 'random'" class="flex items-center gap-4">
          <label class="text-sm text-elid-text-muted">
            Dimensions:
            <span class="font-mono text-elid-accent ml-1">{{ randomDims }}</span>
          </label>
          <input
            v-model.number="randomDims"
            type="range"
            min="64"
            max="2048"
            step="64"
            class="flex-1 h-2 bg-elid-bg rounded-full appearance-none cursor-pointer accent-elid-accent"
          />
          <button
            @click="regenerateRandom"
            class="flex items-center gap-2 px-3 py-1.5 bg-elid-surface border border-elid-border rounded-lg hover:border-elid-accent transition-colors text-sm"
          >
            <Icon icon="mdi:refresh" class="w-4 h-4" />
            New Vector
          </button>
        </div>

        <!-- Paste Input -->
        <div v-else>
          <textarea
            v-model="pastedInput"
            class="w-full h-32 px-4 py-3 bg-elid-bg border border-elid-border rounded-lg text-elid-text font-mono text-sm focus:outline-none focus:border-elid-accent focus:ring-1 focus:ring-elid-accent transition-colors resize-y"
            placeholder="Paste comma-separated floats or JSON array, e.g.:
0.123, -0.456, 0.789, ...
or
[0.123, -0.456, 0.789, ...]"
          ></textarea>
          <p v-if="embeddingError" class="mt-1 text-sm text-elid-red">
            <Icon icon="mdi:alert" class="w-4 h-4 inline -mt-0.5 mr-1" />
            {{ embeddingError }}
          </p>
          <p v-else-if="currentEmbedding" class="mt-1 text-sm text-elid-green">
            <Icon icon="mdi:check" class="w-4 h-4 inline -mt-0.5 mr-1" />
            Parsed {{ currentEmbedding.length }} dimensions
          </p>
        </div>
      </div>

      <!-- Profile Selection -->
      <div class="space-y-3">
        <label class="text-sm font-medium text-elid-text-muted">Encoding Profile:</label>
        <div class="grid grid-cols-2 lg:grid-cols-3 gap-2">
          <button
            v-for="profile in PROFILES"
            :key="profile.name"
            @click="selectedProfile = profile.name"
            class="relative flex flex-col items-start p-3 rounded-lg border transition-all text-left"
            :class="
              selectedProfile === profile.name
                ? 'bg-elid-accent/10 border-elid-accent'
                : 'bg-elid-surface border-elid-border/50 hover:border-elid-border'
            "
          >
            <div class="flex items-center gap-2 mb-1">
              <Icon :icon="profile.icon" class="w-4 h-4" :class="selectedProfile === profile.name ? 'text-elid-accent' : 'text-elid-text-muted'" />
              <span class="font-medium text-sm" :class="selectedProfile === profile.name ? 'text-elid-accent' : ''">
                {{ profile.name }}
              </span>
            </div>
            <p class="text-[11px] text-elid-text-muted leading-tight">{{ profile.description }}</p>
            <span
              v-if="profile.reversible"
              class="absolute top-2 right-2 w-5 h-5 rounded-full bg-elid-green/20 flex items-center justify-center"
              title="Reversible - can decode back to embedding"
            >
              <Icon icon="mdi:undo" class="w-3 h-3 text-elid-green" />
            </span>
          </button>
        </div>
      </div>

      <!-- Profile-specific options -->
      <div v-if="selectedProfile === 'Compressed'" class="flex items-center gap-4">
        <label class="text-sm text-elid-text-muted whitespace-nowrap">
          Retention:
          <span class="font-mono text-elid-accent">{{ (compressionPct * 100).toFixed(0) }}%</span>
        </label>
        <input
          v-model.number="compressionPct"
          type="range"
          min="0.1"
          max="1"
          step="0.05"
          class="flex-1 h-2 bg-elid-bg rounded-full appearance-none cursor-pointer accent-elid-accent"
        />
        <div class="flex gap-2 text-xs text-elid-text-muted">
          <span>10%</span>
          <span>100%</span>
        </div>
      </div>

      <div v-if="selectedProfile === 'MaxLength'" class="flex items-center gap-4">
        <label class="text-sm text-elid-text-muted whitespace-nowrap">
          Max Length:
          <span class="font-mono text-elid-accent">{{ maxLength }} chars</span>
        </label>
        <input
          v-model.number="maxLength"
          type="range"
          min="20"
          max="500"
          step="10"
          class="flex-1 h-2 bg-elid-bg rounded-full appearance-none cursor-pointer accent-elid-accent"
        />
        <div class="flex gap-2 text-xs text-elid-text-muted">
          <span>20</span>
          <span>500</span>
        </div>
      </div>

      <!-- Output Section -->
      <div class="bg-elid-surface border border-elid-border/50 rounded-lg p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="font-medium flex items-center gap-2">
            <Icon icon="mdi:code-string" class="w-5 h-5 text-elid-accent" />
            Encoded ELID
          </h3>
          <div class="flex items-center gap-2">
            <button
              v-if="canDecode && !showDecoded"
              @click="decodeElid"
              class="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-elid-green/10 border border-elid-green/30 text-elid-green rounded-lg hover:bg-elid-green/20 transition-colors"
            >
              <Icon icon="mdi:undo" class="w-4 h-4" />
              Decode
            </button>
            <button
              v-if="encodedElid"
              @click="copyToClipboard"
              class="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-elid-surface-alt border border-elid-border rounded-lg hover:border-elid-accent transition-colors"
            >
              <Icon icon="mdi:content-copy" class="w-4 h-4" />
              Copy
            </button>
          </div>
        </div>

        <!-- ELID Output -->
        <div v-if="encodedElid" class="relative">
          <code
            class="block w-full p-3 bg-elid-bg border border-elid-border/30 rounded-lg font-mono text-sm text-elid-text break-all max-h-32 overflow-y-auto"
          >
            {{ encodedElid }}
          </code>
          <div class="mt-2 text-xs text-elid-text-muted">
            Length: <span class="font-mono text-elid-accent">{{ encodedElid.length }}</span> characters
          </div>
        </div>
        <div v-else class="text-center py-6 text-elid-text-muted">
          <Icon icon="mdi:code-braces" class="w-8 h-8 mx-auto mb-2 opacity-40" />
          <p class="text-sm">No output yet</p>
        </div>

        <!-- Metadata -->
        <div v-if="metadata" class="mt-4 pt-4 border-t border-elid-border/30">
          <h4 class="text-xs font-medium text-elid-text-muted uppercase tracking-wider mb-2">Metadata</h4>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-sm">
            <div>
              <span class="text-elid-text-muted">Original:</span>
              <span class="ml-1 font-mono">{{ metadata.originalDims }}d</span>
            </div>
            <div>
              <span class="text-elid-text-muted">Encoded:</span>
              <span class="ml-1 font-mono">{{ metadata.encodedDims }}d</span>
            </div>
            <div>
              <span class="text-elid-text-muted">Precision:</span>
              <span class="ml-1 font-mono">{{ metadata.precision }}</span>
            </div>
            <div>
              <span class="text-elid-text-muted">Mode:</span>
              <span class="ml-1 font-mono">{{ metadata.dimensionMode }}</span>
            </div>
          </div>
        </div>

        <!-- Decoded Vector -->
        <div v-if="showDecoded && decodedVector" class="mt-4 pt-4 border-t border-elid-border/30">
          <div class="flex items-center justify-between mb-2">
            <h4 class="text-xs font-medium text-elid-text-muted uppercase tracking-wider">
              Decoded Vector ({{ decodedVector.length }} dimensions)
            </h4>
            <button
              @click="showDecoded = false"
              class="text-xs text-elid-text-muted hover:text-elid-text transition-colors"
            >
              Hide
            </button>
          </div>
          <code class="block w-full p-3 bg-elid-bg border border-elid-border/30 rounded-lg font-mono text-xs text-elid-text-muted break-all max-h-24 overflow-y-auto">
            [{{ Array.from(decodedVector).slice(0, 20).map(v => v.toFixed(6)).join(", ") }}{{ decodedVector.length > 20 ? ", ..." : "" }}]
          </code>
        </div>
      </div>

      <!-- Info Box -->
      <div class="bg-elid-accent/5 border border-elid-accent/20 rounded-lg p-4">
        <div class="flex items-start gap-3">
          <Icon icon="mdi:information" class="w-5 h-5 text-elid-accent flex-shrink-0 mt-0.5" />
          <div class="text-sm text-elid-text-muted">
            <p class="mb-2">
              <strong class="text-elid-text">ELID</strong> (Embedding Locality IDentifier) encodes high-dimensional
              embedding vectors into compact, sortable strings. Similar embeddings produce lexicographically
              similar ELIDs, enabling vector search using standard string indexes.
            </p>
            <p>
              <strong class="text-elid-green">Reversible profiles</strong> (Lossless, Compressed, MaxLength)
              can decode back to the original embedding. Non-reversible profiles (Mini128, Morton, Hilbert)
              are optimized for similarity comparison via Hamming distance.
            </p>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
