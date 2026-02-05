<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { Icon } from "@iconify/vue";
import {
  initElid,
  isReady,
  getError,
  hasEmbeddingSupport,
  embeddingToBands,
} from "../lib/elid";

interface ImageData {
  url: string;
  embedding: Float64Array;
  bands: string[];
}

const wasmLoading = ref(true);
const wasmError = ref<string | null>(null);
const mounted = ref(false);
const hasEmbeddings = ref(false);

const images = ref<ImageData[]>([]);
const selectedIndex = ref<number | null>(null);
const numBands = ref(4);

// Generate a simple hash code from a string
function hashCode(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) - hash) + str.charCodeAt(i);
    hash |= 0;
  }
  return hash;
}

// Generate a deterministic fake embedding from an image URL
function generateFakeEmbedding(imageUrl: string): Float64Array {
  const seed = Math.abs(hashCode(imageUrl));
  const embedding = new Float64Array(128);
  let s = seed;
  for (let i = 0; i < 128; i++) {
    s = (s * 1103515245 + 12345) & 0x7fffffff;
    embedding[i] = (s / 0x7fffffff - 0.5) * 2;
  }
  // Normalize to unit vector
  let norm = 0;
  for (const v of embedding) norm += v * v;
  norm = Math.sqrt(norm);
  for (let i = 0; i < embedding.length; i++) embedding[i] /= norm;
  return embedding;
}

// Generate bands locally as fallback when WASM function not available
function generateBandsFallback(embedding: Float64Array, bands: number): string[] {
  // Simple local band generation using hash of embedding chunks
  const result: string[] = [];
  const chunkSize = Math.floor(embedding.length / bands);

  for (let b = 0; b < bands; b++) {
    let hash = 0;
    for (let i = b * chunkSize; i < (b + 1) * chunkSize && i < embedding.length; i++) {
      // Simple hash combining position and sign
      hash = ((hash << 1) | (embedding[i] > 0 ? 1 : 0)) >>> 0;
    }
    // Convert to base32hex-like string
    result.push(hash.toString(16).padStart(8, "0"));
  }
  return result;
}

// Get bands for an embedding, using WASM if available
function getBands(embedding: Float64Array): string[] {
  if (hasEmbeddings.value) {
    const wasmBands = embeddingToBands(embedding, numBands.value);
    if (wasmBands) return wasmBands;
  }
  return generateBandsFallback(embedding, numBands.value);
}

// Initialize images with embeddings and bands
function initializeImages(): void {
  const imageList: ImageData[] = [];
  for (let i = 1; i <= 12; i++) {
    const url = `/demo-images/img${i}.jpg`;
    const embedding = generateFakeEmbedding(url);
    const bands = getBands(embedding);
    imageList.push({ url, embedding, bands });
  }
  images.value = imageList;
}

// Refresh bands when numBands changes or WASM loads
function refreshBands(): void {
  for (const img of images.value) {
    img.bands = getBands(img.embedding);
  }
}

// Select an image
function selectImage(index: number): void {
  if (selectedIndex.value === index) {
    selectedIndex.value = null;
  } else {
    selectedIndex.value = index;
  }
}

// Get selected image's bands
const selectedBands = computed<string[]>(() => {
  if (selectedIndex.value === null) return [];
  return images.value[selectedIndex.value]?.bands ?? [];
});

// Check if an image is similar (shares at least one band with selected)
function isSimilar(index: number): boolean {
  if (selectedIndex.value === null || selectedIndex.value === index) return false;
  const selectedImg = images.value[selectedIndex.value];
  const img = images.value[index];
  if (!selectedImg || !img) return false;

  return selectedImg.bands.some(band => img.bands.includes(band));
}

// Get number of shared bands with selected image
function getSharedBands(index: number): number {
  if (selectedIndex.value === null || selectedIndex.value === index) return 0;
  const selectedImg = images.value[selectedIndex.value];
  const img = images.value[index];
  if (!selectedImg || !img) return 0;

  return selectedImg.bands.filter(band => img.bands.includes(band)).length;
}

// Count of similar images
const similarCount = computed<number>(() => {
  if (selectedIndex.value === null) return 0;
  let count = 0;
  for (let i = 0; i < images.value.length; i++) {
    if (isSimilar(i)) count++;
  }
  return count;
});

// Get CSS classes for an image based on selection state
function getImageClasses(index: number): string {
  if (selectedIndex.value === index) {
    return "ring-2 ring-elid-accent scale-[1.02]";
  }
  if (isSimilar(index)) {
    return "ring-2 ring-elid-green";
  }
  return "hover:ring-2 hover:ring-elid-border/50";
}

// Check if a band is shared with selected image
function isBandShared(band: string): boolean {
  if (selectedIndex.value === null) return false;
  // Check if any other image has this band
  for (let i = 0; i < images.value.length; i++) {
    if (i === selectedIndex.value) continue;
    if (images.value[i].bands.includes(band)) return true;
  }
  return false;
}

onMounted(async () => {
  mounted.value = true;
  wasmLoading.value = true;

  const success = await initElid();
  wasmLoading.value = false;

  if (!success) {
    wasmError.value = getError();
  }

  hasEmbeddings.value = hasEmbeddingSupport();
  initializeImages();
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

    <!-- Main Content (shown even if WASM fails - we have fallback) -->
    <template v-else-if="mounted">
      <!-- Info banner -->
      <div class="bg-elid-accent/10 border border-elid-accent/30 rounded-lg p-4">
        <p class="text-sm text-elid-text-muted">
          <strong class="text-elid-text">ELID works with any embedding type</strong> -
          text, images, audio, code. Click an image to see which others share LSH bands.
        </p>
      </div>

      <!-- Band count control -->
      <div class="flex items-center gap-4">
        <label class="text-sm text-elid-text-muted whitespace-nowrap">
          LSH Bands:
          <span class="font-mono text-elid-accent">{{ numBands }}</span>
        </label>
        <input
          v-model.number="numBands"
          type="range"
          min="1"
          max="8"
          step="1"
          class="flex-1 max-w-48 h-2 bg-elid-bg rounded-full appearance-none cursor-pointer accent-elid-accent"
          @change="refreshBands"
        />
        <span class="text-xs text-elid-text-muted">
          More bands = higher collision chance
        </span>
      </div>

      <!-- Image grid -->
      <div class="grid grid-cols-3 sm:grid-cols-4 gap-3">
        <div
          v-for="(img, idx) in images"
          :key="idx"
          @click="selectImage(idx)"
          class="relative aspect-square rounded-lg overflow-hidden cursor-pointer transition-all duration-200"
          :class="getImageClasses(idx)"
        >
          <img
            :src="img.url"
            :alt="`Demo image ${idx + 1}`"
            class="w-full h-full object-cover"
            loading="lazy"
          />
          <!-- Selected overlay -->
          <div
            v-if="selectedIndex === idx"
            class="absolute inset-0 bg-elid-accent/20 pointer-events-none"
          >
            <span class="absolute top-2 left-2 bg-elid-accent text-elid-bg text-xs font-medium px-2 py-0.5 rounded">
              Selected
            </span>
          </div>
          <!-- Similar indicator -->
          <div
            v-else-if="isSimilar(idx)"
            class="absolute inset-0 pointer-events-none"
          >
            <span class="absolute bottom-1 right-1 bg-elid-green text-elid-bg text-xs font-medium px-1.5 py-0.5 rounded">
              {{ getSharedBands(idx) }} band{{ getSharedBands(idx) > 1 ? 's' : '' }}
            </span>
          </div>
        </div>
      </div>

      <!-- Selected image info -->
      <div
        v-if="selectedIndex !== null"
        class="bg-elid-surface border border-elid-border/50 rounded-lg p-4"
      >
        <h4 class="font-medium mb-3 flex items-center gap-2">
          <Icon icon="mdi:image-outline" class="w-5 h-5 text-elid-accent" />
          Image {{ selectedIndex + 1 }} - LSH Bands
        </h4>
        <div class="flex flex-wrap gap-2 mb-4">
          <code
            v-for="(band, i) in selectedBands"
            :key="i"
            class="px-2 py-1 border rounded text-xs font-mono transition-colors"
            :class="isBandShared(band)
              ? 'bg-elid-green/10 border-elid-green/30 text-elid-green'
              : 'bg-elid-bg border-elid-border/30 text-elid-text-muted'"
            :title="isBandShared(band) ? 'Shared with similar images' : 'Unique band'"
          >
            {{ band }}
          </code>
        </div>
        <p class="text-sm text-elid-text-muted">
          <span class="text-elid-green font-medium">{{ similarCount }} similar image{{ similarCount !== 1 ? 's' : '' }}</span>
          found (sharing at least 1 band)
        </p>
      </div>

      <!-- No selection prompt -->
      <div
        v-else
        class="bg-elid-surface/50 border border-elid-border/30 rounded-lg p-6 text-center"
      >
        <Icon icon="mdi:cursor-default-click" class="w-8 h-8 mx-auto mb-2 text-elid-text-muted/50" />
        <p class="text-sm text-elid-text-muted">
          Click an image to see its LSH bands and find similar images
        </p>
      </div>

      <!-- Explanation -->
      <div class="text-xs text-elid-text-muted/70 space-y-2">
        <p>
          This demo uses simulated embeddings generated from image URLs. In production,
          you would use real image embeddings from models like CLIP, MobileNet, or ResNet.
        </p>
        <p v-if="!hasEmbeddings" class="text-elid-yellow/80">
          <Icon icon="mdi:information" class="w-3.5 h-3.5 inline -mt-0.5 mr-1" />
          WASM embedding features not available - using local fallback for band generation.
        </p>
      </div>
    </template>
  </div>
</template>
