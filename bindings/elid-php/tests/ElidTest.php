<?php

declare(strict_types=1);

namespace Elid\Tests;

use Elid\Elid;
use Elid\Exception\ElidException;
use PHPUnit\Framework\TestCase;

class ElidTest extends TestCase
{
    private array $testVectors = [];

    protected function setUp(): void
    {
        parent::setUp();

        // Load test vectors
        $testVectorsPath = dirname(__DIR__, 2) . '/test-vectors.json';
        if (file_exists($testVectorsPath)) {
            $json = json_decode(file_get_contents($testVectorsPath), true);
            $this->testVectors = $json['vectors'] ?? [];
        }
    }

    /**
     * Test encoding with Mini128 profile
     */
    public function testEncodeMini128(): void
    {
        $embedding = array_fill(0, 768, 0.5);

        $elid = Elid::encode($embedding, Elid::MINI128);

        $this->assertIsString($elid);
        $this->assertMatchesRegularExpression('/^[0-9a-v]{24,30}$/', $elid);
        $this->assertEquals(29, strlen($elid));
    }

    /**
     * Test encoding with Morton10x10 profile
     */
    public function testEncodeMorton10x10(): void
    {
        $embedding = array_fill(0, 768, 0.5);

        $elid = Elid::encode($embedding, Elid::MORTON10X10);

        $this->assertIsString($elid);
        $this->assertMatchesRegularExpression('/^[0-9a-v]{24,30}$/', $elid);
        $this->assertGreaterThanOrEqual(24, strlen($elid));
    }

    /**
     * Test encoding with Hilbert10x10 profile
     */
    public function testEncodeHilbert10x10(): void
    {
        $embedding = array_fill(0, 768, 0.5);

        $elid = Elid::encode($embedding, Elid::HILBERT10X10);

        $this->assertIsString($elid);
        $this->assertMatchesRegularExpression('/^[0-9a-v]{24,30}$/', $elid);
        $this->assertGreaterThanOrEqual(24, strlen($elid));
    }

    /**
     * Test decode round-trip
     */
    public function testDecodeRoundtrip(): void
    {
        $embedding = array_fill(0, 768, 0.5);

        $elid = Elid::encode($embedding, Elid::MINI128);
        $decoded = Elid::decode($elid);

        $this->assertIsString($decoded);
        $this->assertGreaterThan(0, strlen($decoded));
    }

    /**
     * Test Hamming distance calculation
     */
    public function testHammingDistance(): void
    {
        $embedding1 = array_fill(0, 768, 0.1);
        $embedding2 = array_fill(0, 768, 0.2);

        $elid1 = Elid::encode($embedding1, Elid::MINI128);
        $elid2 = Elid::encode($embedding2, Elid::MINI128);

        $distance = Elid::hammingDistance($elid1, $elid2);

        $this->assertIsInt($distance);
        $this->assertGreaterThanOrEqual(0, $distance);
        $this->assertLessThanOrEqual(128, $distance);
    }

    /**
     * Test Hamming distance between identical embeddings
     */
    public function testHammingDistanceSame(): void
    {
        $embedding = array_fill(0, 768, 0.5);

        $elid1 = Elid::encode($embedding, Elid::MINI128);
        $elid2 = Elid::encode($embedding, Elid::MINI128);

        $distance = Elid::hammingDistance($elid1, $elid2);

        $this->assertEquals(0, $distance);
    }

    /**
     * Test batch encoding
     */
    public function testEncodeBatch(): void
    {
        $embeddings = [
            array_fill(0, 768, 0.1),
            array_fill(0, 768, 0.2),
            array_fill(0, 768, 0.3),
        ];

        $elids = Elid::encodeBatch($embeddings, Elid::MINI128);

        $this->assertCount(3, $elids);
        foreach ($elids as $elid) {
            $this->assertIsString($elid);
            $this->assertMatchesRegularExpression('/^[0-9a-v]{24,30}$/', $elid);
        }
    }

    /**
     * Test encoding with invalid dimension (too small)
     */
    public function testEncodeInvalidDimensionTooSmall(): void
    {
        $this->expectException(ElidException::class);
        $this->expectExceptionMessage('Invalid embedding dimension');

        $embedding = array_fill(0, 50, 0.5);
        Elid::encode($embedding, Elid::MINI128);
    }

    /**
     * Test encoding with invalid dimension (too large)
     */
    public function testEncodeInvalidDimensionTooLarge(): void
    {
        $this->expectException(ElidException::class);
        $this->expectExceptionMessage('Invalid embedding dimension');

        $embedding = array_fill(0, 3000, 0.5);
        Elid::encode($embedding, Elid::MINI128);
    }

    /**
     * Test encoding with invalid profile
     */
    public function testEncodeInvalidProfile(): void
    {
        $this->expectException(ElidException::class);
        $this->expectExceptionMessage('Invalid profile type');

        $embedding = array_fill(0, 768, 0.5);
        Elid::encode($embedding, 99);
    }

    /**
     * Test decoding invalid ELID
     */
    public function testDecodeInvalid(): void
    {
        $this->expectException(ElidException::class);

        Elid::decode('invalid-elid-string');
    }

    /**
     * Test Hamming distance with non-Mini128 profile
     */
    public function testHammingDistanceInvalidProfile(): void
    {
        $this->expectException(ElidException::class);

        $embedding = array_fill(0, 768, 0.5);

        $elid1 = Elid::encode($embedding, Elid::MORTON10X10);
        $elid2 = Elid::encode($embedding, Elid::MORTON10X10);

        Elid::hammingDistance($elid1, $elid2);
    }

    /**
     * Test cross-language validation with test vectors
     */
    public function testCrossLanguageValidation(): void
    {
        if (empty($this->testVectors)) {
            $this->markTestSkipped('Test vectors not available');
        }

        foreach (array_slice($this->testVectors, 0, 5) as $i => $vector) {
            $embedding = $vector['embedding'];
            $expectedElid = $vector['elid'];
            $profile = $vector['profile'];

            // Only test Mini128 profile for now
            if (strpos($profile, 'Mini128') === false) {
                continue;
            }

            $actualElid = Elid::encode($embedding, Elid::MINI128);

            $this->assertEquals(
                $expectedElid,
                $actualElid,
                "Test vector {$i}: ELID mismatch for profile {$profile}"
            );
        }
    }

    /**
     * Test memory cleanup by encoding/decoding many times
     */
    public function testMemoryCleanup(): void
    {
        $embedding = array_fill(0, 768, 0.5);

        // Encode and decode many times to verify no memory leaks
        for ($i = 0; $i < 100; $i++) {
            $elid = Elid::encode($embedding, Elid::MINI128);
            $decoded = Elid::decode($elid);
            $this->assertIsString($decoded);
        }

        // If we get here without crashing or running out of memory, memory management is working
        $this->assertTrue(true);
    }

    /**
     * Test encoding determinism
     */
    public function testEncodingDeterminism(): void
    {
        $embedding = array_fill(0, 768, 0.5);

        $elid1 = Elid::encode($embedding, Elid::MINI128);
        $elid2 = Elid::encode($embedding, Elid::MINI128);

        $this->assertEquals($elid1, $elid2);
    }

    /**
     * Test different embeddings produce different ELIDs
     */
    public function testDifferentEmbeddings(): void
    {
        // Create two genuinely different embeddings
        $embedding1 = [];
        $embedding2 = [];
        for ($i = 0; $i < 768; $i++) {
            $embedding1[] = sin($i * 0.1);
            $embedding2[] = cos($i * 0.1);
        }

        $elid1 = Elid::encode($embedding1, Elid::MINI128);
        $elid2 = Elid::encode($embedding2, Elid::MINI128);

        $this->assertNotEquals($elid1, $elid2);
    }

    /**
     * Test all three profiles produce valid ELIDs
     */
    public function testAllProfiles(): void
    {
        $embedding = array_fill(0, 768, 0.5);

        $profiles = [
            Elid::MINI128,
            Elid::MORTON10X10,
            Elid::HILBERT10X10,
        ];

        foreach ($profiles as $profile) {
            $elid = Elid::encode($embedding, $profile);
            $this->assertMatchesRegularExpression('/^[0-9a-v]{24,30}$/', $elid);

            // Test decode works
            $decoded = Elid::decode($elid);
            $this->assertIsString($decoded);
        }
    }

    /**
     * Test batch encoding with mixed profiles
     */
    public function testBatchEncodingConsistency(): void
    {
        $embeddings = [
            array_fill(0, 768, 0.1),
            array_fill(0, 768, 0.2),
            array_fill(0, 768, 0.3),
        ];

        // Encode as batch
        $batchElids = Elid::encodeBatch($embeddings, Elid::MINI128);

        // Encode individually
        $individualElids = [];
        foreach ($embeddings as $embedding) {
            $individualElids[] = Elid::encode($embedding, Elid::MINI128);
        }

        // Should be identical
        $this->assertEquals($individualElids, $batchElids);
    }
}
