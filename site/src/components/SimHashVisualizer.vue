<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { Icon } from "@iconify/vue";
import {
  initElid,
  isReady,
  getError,
  simhash,
  simhashDistance,
} from "../lib/elid";

interface HashEntry {
  id: number;
  text: string;
  hash: number | null;
  bits: boolean[];
}

const DEFAULT_STRINGS = ["iPhone 14", "iPhone 15", "Galaxy S23"];

const wasmLoading = ref(true);
const wasmError = ref<string | null>(null);
const mounted = ref(false);
const entries = ref<HashEntry[]>([]);
const newText = ref("");
const compareA = ref<number | null>(null);
const compareB = ref<number | null>(null);
let nextId = 0;

/**
 * Convert a float64 number (representing a 64-bit integer) to an array of 64 booleans.
 * JavaScript can only safely represent integers up to 2^53, so we use BigInt via
 * reinterpretation of the float64 bits.
 */
function numberToBits(n: number): boolean[] {
  // The WASM returns the simhash as an f64.
  // We treat it as an unsigned 64-bit integer via BigInt.
  const big = BigInt(Math.round(n));
  const bits: boolean[] = [];
  for (let i = 63; i >= 0; i--) {
    bits.push(((big >> BigInt(i)) & 1n) === 1n);
  }
  return bits;
}

function computeHash(text: string): { hash: number | null; bits: boolean[] } {
  if (!isReady() || !text.trim()) {
    return { hash: null, bits: Array(64).fill(false) };
  }
  const h = simhash(text);
  if (h === null) return { hash: null, bits: Array(64).fill(false) };
  return { hash: h, bits: numberToBits(h) };
}

function addEntry(text: string): void {
  if (!text.trim()) return;
  const { hash, bits } = computeHash(text);
  entries.value.push({ id: nextId++, text: text.trim(), hash, bits });
  newText.value = "";
}

function removeEntry(id: number): void {
  entries.value = entries.value.filter((e) => e.id !== id);
  if (compareA.value === id) compareA.value = null;
  if (compareB.value === id) compareB.value = null;
}

function recomputeAll(): void {
  entries.value = entries.value.map((entry) => {
    const { hash, bits } = computeHash(entry.text);
    return { ...entry, hash, bits };
  });
}

const comparisonDistance = computed<number | null>(() => {
  if (compareA.value === null || compareB.value === null) return null;
  const entryA = entries.value.find((e) => e.id === compareA.value);
  const entryB = entries.value.find((e) => e.id === compareB.value);
  if (!entryA?.hash || !entryB?.hash) return null;
  return simhashDistance(entryA.hash, entryB.hash);
});

const comparisonSimilarity = computed<number | null>(() => {
  const dist = comparisonDistance.value;
  if (dist === null) return null;
  return 1 - dist / 64;
});

const differingBits = computed<Set<number>>(() => {
  if (compareA.value === null || compareB.value === null) return new Set();
  const entryA = entries.value.find((e) => e.id === compareA.value);
  const entryB = entries.value.find((e) => e.id === compareB.value);
  if (!entryA || !entryB) return new Set();

  const diff = new Set<number>();
  for (let i = 0; i < 64; i++) {
    if (entryA.bits[i] !== entryB.bits[i]) {
      diff.add(i);
    }
  }
  return diff;
});

function isCompared(id: number): boolean {
  return id === compareA.value || id === compareB.value;
}

function toggleCompare(id: number): void {
  if (compareA.value === id) {
    compareA.value = null;
  } else if (compareB.value === id) {
    compareB.value = null;
  } else if (compareA.value === null) {
    compareA.value = id;
  } else if (compareB.value === null) {
    compareB.value = id;
  } else {
    // Both slots full -- replace B
    compareB.value = id;
  }
}

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === "Enter") {
    addEntry(newText.value);
  }
}

function scoreColor(score: number | null): string {
  if (score === null) return "text-elid-text-muted";
  if (score >= 0.8) return "text-elid-green";
  if (score >= 0.5) return "text-elid-yellow";
  return "text-elid-red";
}

onMounted(async () => {
  mounted.value = true;
  wasmLoading.value = true;
  const success = await initElid();
  wasmLoading.value = false;
  if (!success) {
    wasmError.value = getError();
    // Still add entries with null hashes
    for (const text of DEFAULT_STRINGS) {
      entries.value.push({
        id: nextId++,
        text,
        hash: null,
        bits: Array(64).fill(false),
      });
    }
    return;
  }
  for (const text of DEFAULT_STRINGS) {
    addEntry(text);
  }
  // Auto-select first two for comparison
  if (entries.value.length >= 2) {
    compareA.value = entries.value[0].id;
    compareB.value = entries.value[1].id;
  }
});
</script>

<template>
  <div class="space-y-5">
    <!-- Add String Input -->
    <div class="flex gap-3">
      <div class="relative flex-1">
        <Icon
          icon="mdi:plus"
          class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-elid-text-muted"
        />
        <input
          v-model="newText"
          type="text"
          @keydown="handleKeydown"
          class="w-full pl-12 pr-4 py-3 bg-elid-bg border border-elid-border rounded-lg text-elid-text font-mono focus:outline-none focus:border-elid-accent focus:ring-1 focus:ring-elid-accent transition-colors"
          placeholder="Add a string to hash..."
        />
      </div>
      <button
        @click="addEntry(newText)"
        :disabled="!newText.trim()"
        class="px-5 py-3 bg-elid-accent text-elid-bg font-semibold rounded-lg hover:bg-elid-accent/80 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
      >
        Add
      </button>
    </div>

    <!-- Instruction -->
    <p class="text-xs text-elid-text-muted">
      Click on two entries to compare their bit patterns. Differing bits are
      highlighted in red.
    </p>

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

    <!-- Entries -->
    <div v-else-if="mounted && !wasmLoading" class="space-y-3">
      <div
        v-for="entry in entries"
        :key="entry.id"
        class="bg-elid-surface border rounded-lg p-4 cursor-pointer transition-all"
        :class="
          isCompared(entry.id)
            ? 'border-elid-accent ring-1 ring-elid-accent/30'
            : 'border-elid-border/50 hover:border-elid-border'
        "
        @click="toggleCompare(entry.id)"
      >
        <!-- Header -->
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <div
              v-if="entry.id === compareA"
              class="w-5 h-5 rounded-full bg-elid-accent flex items-center justify-center text-[10px] font-bold text-elid-bg"
            >
              A
            </div>
            <div
              v-else-if="entry.id === compareB"
              class="w-5 h-5 rounded-full bg-elid-accent flex items-center justify-center text-[10px] font-bold text-elid-bg"
            >
              B
            </div>
            <span class="font-mono font-medium">"{{ entry.text }}"</span>
          </div>
          <button
            @click.stop="removeEntry(entry.id)"
            class="p-1 text-elid-text-muted hover:text-elid-red transition-colors"
            title="Remove"
          >
            <Icon icon="mdi:close" class="w-4 h-4" />
          </button>
        </div>

        <!-- Bit Visualization -->
        <div class="flex flex-wrap gap-[2px]">
          <div
            v-for="(bit, bitIndex) in entry.bits"
            :key="bitIndex"
            class="w-[calc(100%/64-2px)] min-w-[5px] max-w-[12px] aspect-square rounded-[2px] transition-colors duration-200"
            :class="
              isCompared(entry.id) && differingBits.has(bitIndex)
                ? 'bg-elid-red'
                : bit
                  ? 'bg-elid-accent'
                  : 'bg-elid-surface-alt'
            "
            :title="`Bit ${bitIndex}: ${bit ? '1' : '0'}`"
          ></div>
        </div>

        <!-- Hash value -->
        <div class="mt-2 text-[10px] font-mono text-elid-text-muted truncate">
          Hash: {{ entry.hash !== null ? entry.hash.toFixed(0) : "N/A" }}
        </div>
      </div>

      <!-- Comparison Result -->
      <div
        v-if="compareA !== null && compareB !== null"
        class="bg-elid-accent/5 border border-elid-accent/30 rounded-lg p-4"
      >
        <div class="flex items-center gap-3 mb-2">
          <Icon icon="mdi:compare" class="w-5 h-5 text-elid-accent" />
          <span class="font-semibold text-sm text-elid-accent">Comparison</span>
        </div>
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-elid-text-muted">Hamming Distance:</span>
            <span class="ml-2 font-mono font-bold">
              {{
                comparisonDistance !== null ? comparisonDistance : "--"
              }}
            </span>
            <span class="text-xs text-elid-text-muted ml-1">/ 64 bits</span>
          </div>
          <div>
            <span class="text-elid-text-muted">Similarity:</span>
            <span
              class="ml-2 font-mono font-bold"
              :class="scoreColor(comparisonSimilarity)"
            >
              {{
                comparisonSimilarity !== null
                  ? (comparisonSimilarity * 100).toFixed(1) + "%"
                  : "--"
              }}
            </span>
          </div>
        </div>

        <!-- Visual Comparison Bar -->
        <div
          v-if="comparisonSimilarity !== null"
          class="mt-3 h-2 bg-elid-bg rounded-full overflow-hidden"
        >
          <div
            class="h-full rounded-full transition-all duration-300"
            :class="
              comparisonSimilarity >= 0.8
                ? 'bg-elid-green'
                : comparisonSimilarity >= 0.5
                  ? 'bg-elid-yellow'
                  : 'bg-elid-red'
            "
            :style="{
              width: `${Math.max(comparisonSimilarity * 100, 1)}%`,
            }"
          ></div>
        </div>

        <!-- Bit Legend -->
        <div class="mt-3 flex items-center gap-4 text-[10px] text-elid-text-muted">
          <div class="flex items-center gap-1.5">
            <div class="w-3 h-3 rounded-sm bg-elid-accent"></div>
            <span>Bit = 1 (matching)</span>
          </div>
          <div class="flex items-center gap-1.5">
            <div class="w-3 h-3 rounded-sm bg-elid-surface-alt"></div>
            <span>Bit = 0 (matching)</span>
          </div>
          <div class="flex items-center gap-1.5">
            <div class="w-3 h-3 rounded-sm bg-elid-red"></div>
            <span>Differing bit</span>
          </div>
        </div>
      </div>

      <!-- Empty hint -->
      <div
        v-else-if="entries.length >= 2"
        class="text-center py-4 text-xs text-elid-text-muted"
      >
        Click two entries above to compare their hash bit patterns
      </div>

      <!-- No entries -->
      <div v-if="entries.length === 0" class="text-center py-8 text-elid-text-muted">
        <Icon icon="mdi:fingerprint" class="w-10 h-10 mx-auto mb-3 opacity-40" />
        <p class="text-sm">Add strings above to visualize their SimHash fingerprints</p>
      </div>
    </div>
  </div>
</template>
