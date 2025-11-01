<?php

declare(strict_types=1);

namespace Elid;

use Elid\Exception\ElidException;
use FFI;

/**
 * ELID (Embedding Locality-preserving Identifiers) PHP Bindings
 *
 * Provides PHP interface to the ELID core library via FFI.
 */
class Elid
{
    /** Mini128 profile - SimHash-based 128-bit encoding */
    public const MINI128 = 0;

    /** Morton10x10 profile - Z-order curve encoding */
    public const MORTON10X10 = 1;

    /** Hilbert10x10 profile - Hilbert curve encoding */
    public const HILBERT10X10 = 2;

    /** @var FFI|null */
    private static $ffi = null;

    /** @var string|null */
    private static $libraryPath = null;

    /**
     * Encode an embedding vector into an ELID string
     *
     * @param array<float> $embedding Array of floats (64-2048 dimensions)
     * @param int $profile Profile type (MINI128, MORTON10X10, or HILBERT10X10)
     * @return string The encoded ELID string
     * @throws ElidException If encoding fails
     */
    public static function encode(array $embedding, int $profile = self::MINI128): string
    {
        $ffi = self::getFFI();

        // Validate profile
        if (!in_array($profile, [self::MINI128, self::MORTON10X10, self::HILBERT10X10], true)) {
            throw new ElidException("Invalid profile type: {$profile}");
        }

        // Validate embedding
        $len = count($embedding);
        if ($len < 64 || $len > 2048) {
            throw new ElidException("Invalid embedding dimension: {$len} (must be 64-2048)");
        }

        // Convert PHP array to C float array
        $arr = $ffi->new("float[{$len}]");
        for ($i = 0; $i < $len; $i++) {
            $arr[$i] = (float)$embedding[$i];
        }

        // Call C function
        $result = $ffi->elid_encode($arr, $len, $profile);
        if (FFI::isNull($result)) {
            throw new ElidException("Encoding failed");
        }

        // Convert C string to PHP string
        $elid = FFI::string($result);

        // Free C memory
        $ffi->elid_free_string($result);

        return $elid;
    }

    /**
     * Decode an ELID string to raw bytes
     *
     * @param string $elid The ELID string to decode
     * @return string Binary string containing the decoded bytes
     * @throws ElidException If decoding fails
     */
    public static function decode(string $elid): string
    {
        $ffi = self::getFFI();

        // Create C string
        $c_elid = $ffi->new("char[" . (strlen($elid) + 1) . "]");
        FFI::memcpy($c_elid, $elid, strlen($elid));
        $c_elid[strlen($elid)] = "\0";

        // Call C function
        $out_len = $ffi->new("size_t");
        $result = $ffi->elid_decode($c_elid, FFI::addr($out_len));

        if ($result === null || FFI::isNull($result)) {
            throw new ElidException("Decoding failed");
        }

        $len = $out_len->cdata;

        // Convert C bytes to PHP string
        $bytes = FFI::string($result, $len);

        // Free C memory
        $ffi->elid_free_bytes($result, $len);

        return $bytes;
    }

    /**
     * Compute Hamming distance between two Mini128 ELIDs
     *
     * @param string $elid1 First ELID string
     * @param string $elid2 Second ELID string
     * @return int Hamming distance (0-128)
     * @throws ElidException If computation fails or ELIDs are not Mini128
     */
    public static function hammingDistance(string $elid1, string $elid2): int
    {
        $ffi = self::getFFI();

        // Create C strings
        $c_elid1 = $ffi->new("char[" . (strlen($elid1) + 1) . "]");
        FFI::memcpy($c_elid1, $elid1, strlen($elid1));
        $c_elid1[strlen($elid1)] = "\0";

        $c_elid2 = $ffi->new("char[" . (strlen($elid2) + 1) . "]");
        FFI::memcpy($c_elid2, $elid2, strlen($elid2));
        $c_elid2[strlen($elid2)] = "\0";

        // Call C function
        $distance = $ffi->elid_hamming_distance($c_elid1, $c_elid2);

        // Check for error (u32::MAX)
        if ($distance === 0xFFFFFFFF) {
            throw new ElidException("Hamming distance computation failed (profiles must both be Mini128)");
        }

        return $distance;
    }

    /**
     * Encode a batch of embeddings
     *
     * @param array<array<float>> $embeddings Array of embedding arrays
     * @param int $profile Profile type (MINI128, MORTON10X10, or HILBERT10X10)
     * @return array<string> Array of encoded ELID strings
     * @throws ElidException If any encoding fails
     */
    public static function encodeBatch(array $embeddings, int $profile = self::MINI128): array
    {
        $elids = [];
        foreach ($embeddings as $embedding) {
            $elids[] = self::encode($embedding, $profile);
        }
        return $elids;
    }

    /**
     * Get or initialize the FFI instance
     *
     * @return FFI The FFI instance
     * @throws ElidException If FFI is not available or library cannot be loaded
     */
    private static function getFFI(): FFI
    {
        if (self::$ffi !== null) {
            return self::$ffi;
        }

        if (!extension_loaded('ffi')) {
            throw new ElidException('PHP FFI extension is not enabled');
        }

        $libPath = self::findLibrary();

        self::$ffi = FFI::cdef("
            char* elid_encode(const float* embedding, size_t len, uint8_t profile_type);
            uint8_t* elid_decode(const char* elid_str, size_t* out_len);
            uint32_t elid_hamming_distance(const char* elid1, const char* elid2);
            void elid_free_string(char* s);
            void elid_free_bytes(uint8_t* bytes, size_t len);
        ", $libPath);

        return self::$ffi;
    }

    /**
     * Find the ELID native library for the current platform
     *
     * @return string Path to the library
     * @throws ElidException If library cannot be found
     */
    private static function findLibrary(): string
    {
        if (self::$libraryPath !== null) {
            return self::$libraryPath;
        }

        // Allow override via environment variable
        if (($envPath = getenv('ELID_LIBRARY_PATH')) !== false) {
            if (file_exists($envPath)) {
                self::$libraryPath = $envPath;
                return $envPath;
            }
        }

        // Determine library name based on platform
        $libName = self::getLibraryName();

        // Search locations
        $searchPaths = [
            // Relative to this file (development)
            dirname(__DIR__, 3) . '/elid-ffi/target/release/' . $libName,
            dirname(__DIR__, 3) . '/elid-ffi/target/debug/' . $libName,
            // Installed location
            '/usr/local/lib/' . $libName,
            '/usr/lib/' . $libName,
            // Current directory
            getcwd() . '/' . $libName,
        ];

        foreach ($searchPaths as $path) {
            if (file_exists($path)) {
                self::$libraryPath = $path;
                return $path;
            }
        }

        throw new ElidException(
            "ELID native library not found. Please build elid-ffi or set ELID_LIBRARY_PATH environment variable.\n" .
            "Searched locations:\n" . implode("\n", $searchPaths)
        );
    }

    /**
     * Get the platform-specific library name
     *
     * @return string Library filename
     */
    private static function getLibraryName(): string
    {
        return match (PHP_OS_FAMILY) {
            'Windows' => 'elid_ffi.dll',
            'Darwin'  => 'libelid_ffi.dylib',
            'Linux'   => 'libelid_ffi.so',
            default   => throw new ElidException('Unsupported operating system: ' . PHP_OS_FAMILY)
        };
    }

    /**
     * Set custom library path (useful for testing)
     *
     * @param string $path Path to the library
     * @return void
     */
    public static function setLibraryPath(string $path): void
    {
        self::$libraryPath = $path;
        self::$ffi = null; // Reset FFI to reload
    }
}
