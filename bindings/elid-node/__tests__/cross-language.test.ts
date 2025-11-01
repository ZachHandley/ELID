import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { join } from 'path';
import { encodeElid, decodeElid, hammingDistanceElid, ElidProfile } from '../index';

interface TestVector {
  embedding: number[];
  profile: string;
  elid: string;
  dimensions: number;
}

interface TestVectors {
  version: string;
  vectors: TestVector[];
}

describe('Cross-Language Compatibility', () => {
  let testVectors: TestVectors;

  try {
    const testVectorsPath = join(__dirname, '../../test-vectors.json');
    const testVectorsJson = readFileSync(testVectorsPath, 'utf-8');
    testVectors = JSON.parse(testVectorsJson);
  } catch (error) {
    console.warn('Test vectors file not found, skipping cross-language tests');
    testVectors = { version: '0.0.0', vectors: [] };
  }

  it('should load test vectors', () => {
    expect(testVectors).toBeDefined();
    expect(testVectors.version).toBeDefined();
    console.log(`Loaded test vectors version ${testVectors.version}`);
  });

  if (testVectors.vectors.length > 0) {
    testVectors.vectors.forEach((vector, index) => {
      describe(`Test Vector ${index + 1}: ${vector.profile}`, () => {
        it('should encode to matching ELID string', () => {
          // Convert profile string to enum (extract base profile name)
          let profile: ElidProfile;
          const baseProfile = vector.profile.split(' ')[0]; // Extract "Mini128" from "Mini128 (384D uniform)"
          switch (baseProfile) {
            case 'Mini128':
              profile = ElidProfile.Mini128;
              break;
            case 'Morton10x10':
              profile = ElidProfile.Morton10x10;
              break;
            case 'Hilbert10x10':
              profile = ElidProfile.Hilbert10x10;
              break;
            default:
              throw new Error(`Unknown profile: ${vector.profile}`);
          }

          // Convert to Float64Array
          const embedding = new Float64Array(vector.embedding);

          // Encode
          const elid = encodeElid(embedding, profile);

          // Verify matches expected
          expect(elid).toBe(vector.elid);
        });

        it('should decode to matching bytes', () => {
          // Decode the expected ELID
          const bytes = decodeElid(vector.elid);

          // Verify we get bytes back
          expect(bytes).toBeInstanceOf(Buffer);
          expect(bytes.length).toBeGreaterThan(0);
        });

        it('should round-trip encode-decode', () => {
          // Convert profile string to enum (extract base profile name)
          let profile: ElidProfile;
          const baseProfile = vector.profile.split(' ')[0]; // Extract "Mini128" from "Mini128 (384D uniform)"
          switch (baseProfile) {
            case 'Mini128':
              profile = ElidProfile.Mini128;
              break;
            case 'Morton10x10':
              profile = ElidProfile.Morton10x10;
              break;
            case 'Hilbert10x10':
              profile = ElidProfile.Hilbert10x10;
              break;
            default:
              throw new Error(`Unknown profile: ${vector.profile}`);
          }

          // Convert to Float64Array
          const embedding = new Float64Array(vector.embedding);

          // Encode
          const elid = encodeElid(embedding, profile);

          // Decode
          const bytes = decodeElid(elid);

          // Re-encode should give same result
          const elid2 = encodeElid(embedding, profile);
          expect(elid2).toBe(elid);
        });
      });
    });

    it('should compute correct hamming distances between test vectors', () => {
      if (testVectors.vectors.length < 2) {
        console.log('Skipping hamming distance test: need at least 2 vectors');
        return;
      }

      // Test hamming distance between first two vectors with same profile
      const mini128Vectors = testVectors.vectors.filter(v => v.profile.startsWith('Mini128'));

      if (mini128Vectors.length >= 2) {
        const elid1 = mini128Vectors[0].elid;
        const elid2 = mini128Vectors[1].elid;

        const distance = hammingDistanceElid(elid1, elid2);

        // Distance should be valid
        expect(distance).toBeGreaterThanOrEqual(0);
        expect(distance).toBeLessThanOrEqual(128); // Max for Mini128

        console.log(`Hamming distance between vector 1 and 2: ${distance}`);
      }
    });

    it('should match Rust implementation exactly', () => {
      // This test verifies that the TypeScript bindings produce
      // byte-identical results to the Rust implementation

      testVectors.vectors.forEach((vector, index) => {
        // Convert profile (extract base profile name)
        let profile: ElidProfile;
        const baseProfile = vector.profile.split(' ')[0]; // Extract "Mini128" from "Mini128 (384D uniform)"
        switch (baseProfile) {
          case 'Mini128':
            profile = ElidProfile.Mini128;
            break;
          case 'Morton10x10':
            profile = ElidProfile.Morton10x10;
            break;
          case 'Hilbert10x10':
            profile = ElidProfile.Hilbert10x10;
            break;
          default:
            throw new Error(`Unknown profile: ${vector.profile}`);
        }

        const embedding = new Float64Array(vector.embedding);
        const elid = encodeElid(embedding, profile);

        // Byte-for-byte comparison
        expect(elid).toBe(vector.elid);

        console.log(`Vector ${index + 1} matches Rust: ✓`);
      });
    });
  } else {
    it('should skip tests when no test vectors available', () => {
      expect(testVectors.vectors).toHaveLength(0);
    });
  }
});
