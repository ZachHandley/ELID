<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { Icon } from "@iconify/vue";
import {
  initElid,
  isReady,
  getError,
  findMatchesAboveThreshold,
  jaroWinkler,
  normalizedLevenshtein,
  simhashSimilarity,
  type MatchResult,
} from "../lib/elid";

const CANDIDATES: string[] = [
  "iPhone 14 Pro",
  "iPhone 14",
  "iPhone 13 Pro Max",
  "Samsung Galaxy S23",
  "Samsung Galaxy S24 Ultra",
  "Google Pixel 8",
  "MacBook Pro 16",
  "MacBook Air M3",
  "iPad Pro 12.9",
  "Microsoft Surface Pro",
  "John Smith",
  "Jon Smith",
  "Jonathan Smithe",
  "Jane Doe",
  "Janet Doe",
  "New York",
  "New York City",
  "Newark",
  "New Orleans",
  "Newport",
];

interface SearchResult {
  text: string;
  score: number;
  bestAlgorithm: string;
  scores: {
    jaroWinkler: number;
    normalizedLevenshtein: number;
    simhash: number;
  };
}

const query = ref("");
const threshold = ref(0.3);
const wasmLoading = ref(true);
const wasmError = ref<string | null>(null);
const mounted = ref(false);
const searchResults = ref<SearchResult[]>([]);

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

function performSearch(): void {
  if (!isReady() || !query.value.trim()) {
    searchResults.value = [];
    return;
  }

  const q = query.value.trim();
  const results: SearchResult[] = [];

  for (let i = 0; i < CANDIDATES.length; i++) {
    const candidate = CANDIDATES[i];
    const jwScore = jaroWinkler(q, candidate) ?? 0;
    const nlScore = normalizedLevenshtein(q, candidate) ?? 0;
    const shScore = simhashSimilarity(q, candidate) ?? 0;

    const maxScore = Math.max(jwScore, nlScore, shScore);

    let bestAlgorithm = "Jaro-Winkler";
    if (nlScore >= jwScore && nlScore >= shScore)
      bestAlgorithm = "Normalized Levenshtein";
    else if (shScore >= jwScore && shScore >= nlScore)
      bestAlgorithm = "SimHash";

    if (maxScore >= threshold.value) {
      results.push({
        text: candidate,
        score: maxScore,
        bestAlgorithm,
        scores: {
          jaroWinkler: jwScore,
          normalizedLevenshtein: nlScore,
          simhash: shScore,
        },
      });
    }
  }

  results.sort((a, b) => b.score - a.score);
  searchResults.value = results;
}

function scoreColor(score: number): string {
  if (score >= 0.8) return "text-elid-green";
  if (score >= 0.5) return "text-elid-yellow";
  return "text-elid-red";
}

function scoreBgColor(score: number): string {
  if (score >= 0.8) return "bg-elid-green";
  if (score >= 0.5) return "bg-elid-yellow";
  return "bg-elid-red";
}

function algorithmIcon(algo: string): string {
  switch (algo) {
    case "Jaro-Winkler":
      return "mdi:swap-horizontal-bold";
    case "Normalized Levenshtein":
      return "mdi:percent";
    case "SimHash":
      return "mdi:fingerprint";
    default:
      return "mdi:help-circle";
  }
}

watch([query, threshold], () => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    performSearch();
  }, 100);
});

onMounted(async () => {
  mounted.value = true;
  wasmLoading.value = true;
  const success = await initElid();
  wasmLoading.value = false;
  if (!success) {
    wasmError.value = getError();
  }
  performSearch();
});
</script>

<template>
  <div class="space-y-5">
    <!-- Search Input -->
    <div class="relative">
      <Icon
        icon="mdi:magnify"
        class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-elid-text-muted"
      />
      <input
        v-model="query"
        type="text"
        class="w-full pl-12 pr-4 py-3 bg-elid-bg border border-elid-border rounded-lg text-elid-text font-mono text-lg focus:outline-none focus:border-elid-accent focus:ring-1 focus:ring-elid-accent transition-colors"
        placeholder="Search for products, names, places..."
      />
    </div>

    <!-- Threshold Slider -->
    <div class="flex items-center gap-4">
      <label class="text-sm text-elid-text-muted whitespace-nowrap">
        Threshold:
        <span class="font-mono text-elid-accent">{{
          threshold.toFixed(2)
        }}</span>
      </label>
      <input
        v-model.number="threshold"
        type="range"
        min="0"
        max="1"
        step="0.05"
        class="flex-1 h-2 bg-elid-bg rounded-full appearance-none cursor-pointer accent-elid-accent"
      />
      <div class="flex gap-2 text-xs text-elid-text-muted">
        <span>0.0</span>
        <span>1.0</span>
      </div>
    </div>

    <!-- WASM Loading -->
    <div
      v-if="wasmLoading && mounted"
      class="flex items-center justify-center gap-3 py-8 text-elid-text-muted"
    >
      <Icon icon="mdi:loading" class="w-5 h-5 animate-spin" />
      <span>Loading WASM module...</span>
    </div>

    <!-- WASM Error -->
    <div
      v-else-if="wasmError && mounted"
      class="bg-elid-red/10 border border-elid-red/30 rounded-lg p-4 text-center"
    >
      <p class="text-elid-red text-sm">
        WASM module not available. Build the library first.
      </p>
    </div>

    <!-- Results -->
    <div v-else-if="mounted && !wasmLoading">
      <!-- Results Count -->
      <div
        v-if="query.trim()"
        class="flex items-center justify-between mb-3 text-sm text-elid-text-muted"
      >
        <span>
          {{ searchResults.length }} result{{
            searchResults.length !== 1 ? "s" : ""
          }}
          found
        </span>
        <span class="text-xs">
          Searching {{ CANDIDATES.length }} items
        </span>
      </div>

      <!-- Result Items -->
      <div class="space-y-2">
        <div
          v-for="(result, index) in searchResults"
          :key="result.text"
          class="bg-elid-surface border border-elid-border/50 rounded-lg p-4 hover:border-elid-border transition-colors"
        >
          <div class="flex items-center justify-between gap-4">
            <!-- Left: Rank + Name -->
            <div class="flex items-center gap-3 min-w-0">
              <span
                class="flex-shrink-0 w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold"
                :class="
                  index === 0
                    ? 'bg-elid-accent/20 text-elid-accent'
                    : 'bg-elid-surface-alt text-elid-text-muted'
                "
              >
                {{ index + 1 }}
              </span>
              <span class="font-medium truncate">{{ result.text }}</span>
            </div>

            <!-- Right: Score + Best Algorithm -->
            <div class="flex items-center gap-3 flex-shrink-0">
              <div class="flex items-center gap-1.5 text-xs text-elid-text-muted">
                <Icon :icon="algorithmIcon(result.bestAlgorithm)" class="w-3.5 h-3.5" />
                <span class="hidden sm:inline">{{ result.bestAlgorithm }}</span>
              </div>
              <span
                class="font-mono text-lg font-bold"
                :class="scoreColor(result.score)"
              >
                {{ result.score.toFixed(3) }}
              </span>
            </div>
          </div>

          <!-- Score Breakdown Bar -->
          <div class="mt-3 flex items-center gap-2">
            <div class="flex-1 grid grid-cols-3 gap-1 text-[10px] text-elid-text-muted">
              <div>
                <div class="flex justify-between mb-0.5">
                  <span>JW</span>
                  <span class="font-mono">{{
                    result.scores.jaroWinkler.toFixed(2)
                  }}</span>
                </div>
                <div class="h-1 bg-elid-bg rounded-full overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-300"
                    :class="scoreBgColor(result.scores.jaroWinkler)"
                    :style="{
                      width: `${Math.max(result.scores.jaroWinkler * 100, 1)}%`,
                    }"
                  ></div>
                </div>
              </div>
              <div>
                <div class="flex justify-between mb-0.5">
                  <span>NL</span>
                  <span class="font-mono">{{
                    result.scores.normalizedLevenshtein.toFixed(2)
                  }}</span>
                </div>
                <div class="h-1 bg-elid-bg rounded-full overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-300"
                    :class="scoreBgColor(result.scores.normalizedLevenshtein)"
                    :style="{
                      width: `${Math.max(result.scores.normalizedLevenshtein * 100, 1)}%`,
                    }"
                  ></div>
                </div>
              </div>
              <div>
                <div class="flex justify-between mb-0.5">
                  <span>SH</span>
                  <span class="font-mono">{{
                    result.scores.simhash.toFixed(2)
                  }}</span>
                </div>
                <div class="h-1 bg-elid-bg rounded-full overflow-hidden">
                  <div
                    class="h-full rounded-full transition-all duration-300"
                    :class="scoreBgColor(result.scores.simhash)"
                    :style="{
                      width: `${Math.max(result.scores.simhash * 100, 1)}%`,
                    }"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty States -->
      <div
        v-if="!query.trim()"
        class="text-center py-8 text-elid-text-muted"
      >
        <Icon icon="mdi:magnify" class="w-10 h-10 mx-auto mb-3 opacity-40" />
        <p class="text-sm">Start typing to search across {{ CANDIDATES.length }} items</p>
        <p class="text-xs mt-1 opacity-60">
          Try "iphone", "john", or "new york"
        </p>
      </div>
      <div
        v-else-if="searchResults.length === 0"
        class="text-center py-8 text-elid-text-muted"
      >
        <Icon icon="mdi:magnify-close" class="w-10 h-10 mx-auto mb-3 opacity-40" />
        <p class="text-sm">No matches above threshold {{ threshold.toFixed(2) }}</p>
        <p class="text-xs mt-1 opacity-60">
          Try lowering the threshold slider
        </p>
      </div>
    </div>
  </div>
</template>
