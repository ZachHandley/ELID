<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { Icon } from "@iconify/vue";
import {
  initElid,
  isReady,
  getError,
  levenshtein,
  normalizedLevenshtein,
  osaDistance,
  hamming,
  jaro,
  jaroWinkler,
  bestMatch,
  simhashSimilarity,
} from "../lib/elid";

interface ExamplePair {
  a: string;
  b: string;
  label: string;
  note: string;
}

const examplePairs: ExamplePair[] = [
  {
    a: "kitten",
    b: "sitting",
    label: "Classic",
    note: "The classic edit distance example",
  },
  {
    a: "martha",
    b: "marhta",
    label: "Jaro-Winkler",
    note: "Transposition shows Jaro-Winkler strength",
  },
  {
    a: "iPhone 14 Pro",
    b: "iPhone 15 Pro",
    label: "SimHash",
    note: "Near-identical strings show SimHash strength",
  },
  {
    a: "receive",
    b: "recieve",
    label: "Typo",
    note: "Common spelling mistake detection",
  },
  {
    a: "ca",
    b: "ac",
    label: "Transpose",
    note: "OSA vs Levenshtein difference",
  },
];

const stringA = ref("kitten");
const stringB = ref("sitting");
const caseSensitive = ref(true);
const trimWhitespace = ref(false);
const wasmLoading = ref(true);
const wasmError = ref<string | null>(null);
const mounted = ref(false);

interface AlgorithmResult {
  name: string;
  icon: string;
  value: number | string | null;
  normalized: number | null;
  description: string;
  extra?: string;
}

const effectiveA = computed(() => {
  let val = stringA.value;
  if (!caseSensitive.value) val = val.toLowerCase();
  if (trimWhitespace.value) val = val.trim();
  return val;
});

const effectiveB = computed(() => {
  let val = stringB.value;
  if (!caseSensitive.value) val = val.toLowerCase();
  if (trimWhitespace.value) val = val.trim();
  return val;
});

const results = ref<AlgorithmResult[]>([]);

function computeResults(): void {
  if (!isReady()) {
    results.value = [];
    return;
  }

  const a = effectiveA.value;
  const b = effectiveB.value;

  if (!a && !b) {
    results.value = [];
    return;
  }

  const levDist = levenshtein(a, b);
  const normLev = normalizedLevenshtein(a, b);
  const osaDist = osaDistance(a, b);
  const hammDist = hamming(a, b);
  const jaroScore = jaro(a, b);
  const jwScore = jaroWinkler(a, b);
  const simScore = simhashSimilarity(a, b);
  const bestScore = bestMatch(a, b);

  const transpositionDetected =
    levDist !== null && osaDist !== null && osaDist < levDist;

  results.value = [
    {
      name: "Best Match",
      icon: "mdi:trophy",
      value: bestScore !== null ? bestScore.toFixed(4) : null,
      normalized: bestScore,
      description: "Composite maximum of normalizedLevenshtein and jaroWinkler",
    },
    {
      name: "Jaro-Winkler",
      icon: "mdi:swap-horizontal-bold",
      value: jwScore !== null ? jwScore.toFixed(4) : null,
      normalized: jwScore,
      description:
        "Jaro with prefix bonus -- favors strings that match from the start",
    },
    {
      name: "Jaro",
      icon: "mdi:swap-horizontal",
      value: jaroScore !== null ? jaroScore.toFixed(4) : null,
      normalized: jaroScore,
      description: "Based on matching characters and transpositions",
    },
    {
      name: "Normalized Levenshtein",
      icon: "mdi:percent",
      value: normLev !== null ? normLev.toFixed(4) : null,
      normalized: normLev,
      description: "Levenshtein distance normalized to 0-1 similarity range",
    },
    {
      name: "Levenshtein Distance",
      icon: "mdi:pencil-ruler",
      value: levDist,
      normalized: normLev,
      description: "Minimum insertions, deletions, substitutions to transform",
    },
    {
      name: "OSA Distance",
      icon: "mdi:rotate-left",
      value: osaDist,
      normalized: normLev,
      description: "Optimal String Alignment -- also counts transpositions",
      extra: transpositionDetected
        ? "Transposition detected! OSA < Levenshtein"
        : osaDist !== null && levDist !== null && osaDist === levDist
          ? "No transposition benefit"
          : undefined,
    },
    {
      name: "Hamming Distance",
      icon: "mdi:ab-testing",
      value:
        hammDist !== null
          ? hammDist
          : a.length !== b.length
            ? "N/A (different lengths)"
            : null,
      normalized:
        hammDist !== null && Math.max(a.length, b.length) > 0
          ? 1 - hammDist / Math.max(a.length, b.length)
          : null,
      description:
        "Number of positions where characters differ (same-length strings only)",
    },
    {
      name: "SimHash Similarity",
      icon: "mdi:fingerprint",
      value: simScore !== null ? simScore.toFixed(4) : null,
      normalized: simScore,
      description:
        "Locality-sensitive hashing: similar strings produce similar hashes",
    },
  ];
}

function scoreColor(score: number | null): string {
  if (score === null) return "bg-elid-surface-alt";
  if (score >= 0.8) return "bg-elid-green";
  if (score >= 0.5) return "bg-elid-yellow";
  return "bg-elid-red";
}

function scoreTextColor(score: number | null): string {
  if (score === null) return "text-elid-text-muted";
  if (score >= 0.8) return "text-elid-green";
  if (score >= 0.5) return "text-elid-yellow";
  return "text-elid-red";
}

function loadExample(pair: ExamplePair): void {
  stringA.value = pair.a;
  stringB.value = pair.b;
}

function swapStrings(): void {
  const tmp = stringA.value;
  stringA.value = stringB.value;
  stringB.value = tmp;
}

watch([effectiveA, effectiveB], () => {
  computeResults();
});

onMounted(async () => {
  mounted.value = true;
  wasmLoading.value = true;
  const success = await initElid();
  wasmLoading.value = false;
  if (!success) {
    wasmError.value = getError();
  }
  computeResults();
});
</script>

<template>
  <div class="space-y-6">
    <!-- Input Section -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label
          for="string-a"
          class="block text-sm font-medium text-elid-text-muted mb-2"
        >
          String A
        </label>
        <input
          id="string-a"
          v-model="stringA"
          type="text"
          class="w-full px-4 py-3 bg-elid-bg border border-elid-border rounded-lg text-elid-text font-mono text-lg focus:outline-none focus:border-elid-accent focus:ring-1 focus:ring-elid-accent transition-colors"
          placeholder="Enter first string..."
        />
      </div>
      <div>
        <label
          for="string-b"
          class="block text-sm font-medium text-elid-text-muted mb-2"
        >
          String B
        </label>
        <input
          id="string-b"
          v-model="stringB"
          type="text"
          class="w-full px-4 py-3 bg-elid-bg border border-elid-border rounded-lg text-elid-text font-mono text-lg focus:outline-none focus:border-elid-accent focus:ring-1 focus:ring-elid-accent transition-colors"
          placeholder="Enter second string..."
        />
      </div>
    </div>

    <!-- Controls Row -->
    <div class="flex flex-wrap items-center gap-4">
      <!-- Swap Button -->
      <button
        @click="swapStrings"
        class="flex items-center gap-2 px-4 py-2 bg-elid-surface border border-elid-border rounded-lg hover:border-elid-accent transition-colors text-sm"
        title="Swap strings"
      >
        <Icon icon="mdi:swap-horizontal" class="w-4 h-4" />
        Swap
      </button>

      <!-- Toggles -->
      <label class="flex items-center gap-2 text-sm cursor-pointer select-none">
        <input
          v-model="caseSensitive"
          type="checkbox"
          class="w-4 h-4 accent-elid-accent"
        />
        <span class="text-elid-text-muted">Case Sensitive</span>
      </label>
      <label class="flex items-center gap-2 text-sm cursor-pointer select-none">
        <input
          v-model="trimWhitespace"
          type="checkbox"
          class="w-4 h-4 accent-elid-accent"
        />
        <span class="text-elid-text-muted">Trim Whitespace</span>
      </label>

      <!-- Divider -->
      <div class="hidden md:block w-px h-6 bg-elid-border"></div>

      <!-- Example Pairs -->
      <div class="flex flex-wrap items-center gap-2">
        <span class="text-xs text-elid-text-muted uppercase tracking-wider"
          >Examples:</span
        >
        <button
          v-for="pair in examplePairs"
          :key="pair.label"
          @click="loadExample(pair)"
          class="px-3 py-1 text-xs bg-elid-surface-alt/50 border border-elid-border/50 rounded-full hover:border-elid-accent hover:text-elid-accent transition-colors"
          :title="pair.note"
        >
          {{ pair.label }}
        </button>
      </div>
    </div>

    <!-- WASM Loading State -->
    <div
      v-if="wasmLoading && mounted"
      class="flex items-center justify-center gap-3 py-12 text-elid-text-muted"
    >
      <Icon icon="mdi:loading" class="w-6 h-6 animate-spin" />
      <span class="text-lg">Loading WASM module...</span>
    </div>

    <!-- WASM Error State -->
    <div
      v-else-if="wasmError && mounted"
      class="bg-elid-red/10 border border-elid-red/30 rounded-lg p-6 text-center"
    >
      <Icon icon="mdi:alert-circle" class="w-8 h-8 text-elid-red mx-auto mb-3" />
      <p class="text-elid-red font-medium mb-2">WASM module not available</p>
      <p class="text-sm text-elid-text-muted">
        The WASM binary hasn't been built yet. Run the build pipeline or place
        the compiled files in
        <code class="font-mono bg-elid-surface px-1.5 py-0.5 rounded"
          >public/wasm/</code
        >.
      </p>
    </div>

    <!-- Results Grid -->
    <div
      v-else-if="results.length > 0"
      class="grid grid-cols-1 lg:grid-cols-2 gap-3"
    >
      <div
        v-for="result in results"
        :key="result.name"
        class="bg-elid-surface border border-elid-border/50 rounded-lg p-4 hover:border-elid-border transition-colors"
        :class="{
          'lg:col-span-2 border-elid-accent/30 bg-elid-accent/5':
            result.name === 'Best Match',
        }"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="flex items-center gap-3 min-w-0">
            <div
              class="flex-shrink-0 w-9 h-9 rounded-lg flex items-center justify-center"
              :class="
                result.name === 'Best Match'
                  ? 'bg-elid-accent/20 text-elid-accent'
                  : 'bg-elid-surface-alt text-elid-text-muted'
              "
            >
              <Icon :icon="result.icon" class="w-5 h-5" />
            </div>
            <div class="min-w-0">
              <h3
                class="font-semibold text-sm"
                :class="{
                  'text-elid-accent': result.name === 'Best Match',
                }"
              >
                {{ result.name }}
              </h3>
              <p class="text-xs text-elid-text-muted truncate">
                {{ result.description }}
              </p>
            </div>
          </div>
          <div class="flex-shrink-0 text-right">
            <span
              class="font-mono text-xl font-bold"
              :class="
                typeof result.value === 'string' &&
                result.value.includes('N/A')
                  ? 'text-elid-text-muted text-sm'
                  : scoreTextColor(result.normalized)
              "
            >
              {{ result.value ?? "--" }}
            </span>
          </div>
        </div>

        <!-- Score Bar -->
        <div
          v-if="result.normalized !== null"
          class="mt-3 h-2 bg-elid-bg rounded-full overflow-hidden"
        >
          <div
            class="h-full rounded-full transition-all duration-300 ease-out"
            :class="scoreColor(result.normalized)"
            :style="{ width: `${Math.max(result.normalized * 100, 1)}%` }"
          ></div>
        </div>

        <!-- Extra Info -->
        <p
          v-if="result.extra"
          class="mt-2 text-xs font-medium"
          :class="
            result.extra.includes('detected')
              ? 'text-elid-orange'
              : 'text-elid-text-muted'
          "
        >
          <Icon
            v-if="result.extra.includes('detected')"
            icon="mdi:information"
            class="w-3.5 h-3.5 inline-block mr-1 -mt-0.5"
          />
          {{ result.extra }}
        </p>
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-else-if="mounted && !wasmLoading && !wasmError"
      class="text-center py-12 text-elid-text-muted"
    >
      <Icon icon="mdi:text-search" class="w-12 h-12 mx-auto mb-3 opacity-50" />
      <p>Enter two strings to compare</p>
    </div>
  </div>
</template>
