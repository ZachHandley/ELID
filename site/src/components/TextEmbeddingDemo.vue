<template>
  <div class="text-embedding-demo">
    <div v-if="!modelSupported" class="notice">
      <p>Text embedding requires WASM built with <code>models-text</code> feature.</p>
    </div>
    <div v-else>
      <div class="model-status">
        <button
          @click="loadModel"
          :disabled="modelLoading || modelReady"
          class="load-btn"
        >
          {{ modelReady ? 'Model Ready' : modelLoading ? 'Downloading...' : 'Load Text Model (~30MB)' }}
        </button>
        <span v-if="modelError" class="error">{{ modelError }}</span>
      </div>

      <div v-if="modelReady" class="embed-section">
        <div class="input-row">
          <textarea
            v-model="text1"
            placeholder="Enter first text..."
            rows="2"
          />
          <textarea
            v-model="text2"
            placeholder="Enter second text..."
            rows="2"
          />
        </div>
        <button @click="computeEmbeddings" :disabled="!text1 || !text2" class="embed-btn">
          Compute Embeddings &amp; Compare
        </button>

        <div v-if="result" class="results">
          <div class="result-row">
            <span class="label">Text 1 embedding:</span>
            <span class="value">{{ result.dims1 }}-dim, first 5: [{{ result.preview1 }}]</span>
          </div>
          <div class="result-row">
            <span class="label">Text 2 embedding:</span>
            <span class="value">{{ result.dims2 }}-dim, first 5: [{{ result.preview2 }}]</span>
          </div>
          <div class="result-row highlight">
            <span class="label">Cosine similarity:</span>
            <span class="value" :class="similarityClass">{{ result.cosine.toFixed(4) }}</span>
          </div>
          <div class="result-row">
            <span class="label">Inference time:</span>
            <span class="value">{{ result.timeMs.toFixed(1) }}ms</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  initElid,
  hasTextModelSupport,
  initTextModel,
  embedText,
} from '../lib/elid';

const modelSupported = ref(false);
const modelLoading = ref(false);
const modelReady = ref(false);
const modelError = ref<string | null>(null);

const text1 = ref('The quick brown fox jumps over the lazy dog');
const text2 = ref('The fast brown fox leaps over the sleepy dog');

interface EmbedResult {
  dims1: number;
  dims2: number;
  preview1: string;
  preview2: string;
  cosine: number;
  timeMs: number;
}

const result = ref<EmbedResult | null>(null);

const similarityClass = computed(() => {
  if (!result.value) return '';
  const c = result.value.cosine;
  if (c > 0.8) return 'high';
  if (c > 0.5) return 'medium';
  return 'low';
});

onMounted(async () => {
  await initElid();
  modelSupported.value = hasTextModelSupport();
});

async function loadModel() {
  modelLoading.value = true;
  modelError.value = null;
  try {
    const ok = await initTextModel();
    if (!ok) {
      modelError.value = 'Failed to load model';
    } else {
      modelReady.value = true;
    }
  } catch (e) {
    modelError.value = String(e);
  } finally {
    modelLoading.value = false;
  }
}

function computeEmbeddings() {
  const start = performance.now();
  const emb1 = embedText(text1.value);
  const emb2 = embedText(text2.value);
  const elapsed = performance.now() - start;

  if (!emb1 || !emb2) {
    modelError.value = 'Embedding failed';
    return;
  }

  let dot = 0;
  for (let i = 0; i < emb1.length; i++) {
    dot += emb1[i] * emb2[i];
  }

  result.value = {
    dims1: emb1.length,
    dims2: emb2.length,
    preview1: Array.from(emb1.slice(0, 5)).map(v => v.toFixed(3)).join(', '),
    preview2: Array.from(emb2.slice(0, 5)).map(v => v.toFixed(3)).join(', '),
    cosine: dot,
    timeMs: elapsed,
  };
}
</script>

<style scoped>
.text-embedding-demo {
  padding: 1rem;
}

.notice {
  padding: 1rem;
  background: #2a2a3e;
  border-radius: 8px;
  color: #aaa;
}

.model-status {
  margin-bottom: 1rem;
}

.load-btn, .embed-btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
}

.load-btn {
  background: #4a9eff;
  color: white;
}

.load-btn:disabled {
  background: #555;
  cursor: default;
}

.embed-btn {
  background: #5a5aff;
  color: white;
  margin-top: 0.5rem;
}

.embed-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.error {
  color: #ff5555;
  margin-left: 0.5rem;
}

.input-row {
  display: flex;
  gap: 0.5rem;
}

.input-row textarea {
  flex: 1;
  padding: 0.5rem;
  background: #1a1a2e;
  border: 1px solid #333;
  border-radius: 6px;
  color: #eee;
  font-family: inherit;
  resize: vertical;
}

.results {
  margin-top: 1rem;
  background: #1a1a2e;
  border-radius: 8px;
  padding: 1rem;
}

.result-row {
  display: flex;
  justify-content: space-between;
  padding: 0.3rem 0;
  border-bottom: 1px solid #2a2a3e;
}

.result-row:last-child {
  border-bottom: none;
}

.result-row.highlight {
  font-weight: bold;
  font-size: 1.1rem;
}

.label {
  color: #888;
}

.value {
  font-family: monospace;
}

.value.high { color: #4ade80; }
.value.medium { color: #fbbf24; }
.value.low { color: #f87171; }
</style>
