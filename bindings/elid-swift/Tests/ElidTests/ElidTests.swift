import XCTest
@testable import Elid

// Test vector structure matching test-vectors.json
struct TestVector: Codable {
    let embedding: [Float]
    let mini128: String
    let morton10x10: String?
    let hilbert10x10: String?

    enum CodingKeys: String, CodingKey {
        case embedding
        case mini128 = "Mini128"
        case morton10x10 = "Morton10x10"
        case hilbert10x10 = "Hilbert10x10"
    }
}

struct TestVectors: Codable {
    let version: String
    let vectors: [TestVector]
}

final class ElidTests: XCTestCase {

    // MARK: - Test Vector Loading

    func loadTestVectors() throws -> TestVectors {
        let bundle = Bundle.module
        guard let url = bundle.url(forResource: "test-vectors", withExtension: "json") else {
            XCTFail("Could not find test-vectors.json in test bundle")
            throw NSError(domain: "ElidTests", code: 1, userInfo: [NSLocalizedDescriptionKey: "test-vectors.json not found"])
        }

        let data = try Data(contentsOf: url)
        let decoder = JSONDecoder()
        return try decoder.decode(TestVectors.self, from: data)
    }

    // MARK: - Basic Encoding Tests

    func testEncodeBasic() throws {
        // Create a simple 128-dimensional embedding
        let embedding = (0..<128).map { Float($0) / 128.0 }

        // Encode with Mini128 profile
        let elid = try uniEncode(embedding: embedding, profile: .mini128)

        // Verify it's a valid string (should be 29 characters for base32hex: 2-byte header + 128-bit payload)
        XCTAssertFalse(elid.isEmpty, "ELID should not be empty")
        XCTAssertEqual(elid.count, 29, "Mini128 ELID should be 29 characters")

        // Verify it only contains valid base32hex characters
        let validChars = CharacterSet(charactersIn: "0123456789ABCDEFGHIJKLMNOPQRSTUV")
        let elidChars = CharacterSet(charactersIn: elid)
        XCTAssertTrue(validChars.isSuperset(of: elidChars), "ELID should only contain base32hex characters")
    }

    func testEncodeMorton() throws {
        // Create a 10-dimensional embedding for Morton encoding
        let embedding: [Float] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]

        // Encode with Morton profile
        let elid = try uniEncode(embedding: embedding, profile: .morton10x10)

        XCTAssertFalse(elid.isEmpty, "Morton ELID should not be empty")
    }

    func testEncodeHilbert() throws {
        // Create a 10-dimensional embedding for Hilbert encoding
        let embedding: [Float] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]

        // Encode with Hilbert profile
        let elid = try uniEncode(embedding: embedding, profile: .hilbert10x10)

        XCTAssertFalse(elid.isEmpty, "Hilbert ELID should not be empty")
    }

    // MARK: - Decoding Tests

    func testDecodeBasic() throws {
        // First encode an embedding
        let embedding = (0..<128).map { Float($0) / 128.0 }
        let elid = try uniEncode(embedding: embedding, profile: .mini128)

        // Then decode it
        let decoded = try uniDecode(elid: elid)

        // Verify we got bytes back (should be 18 bytes for Mini128: 2-byte header + 16-byte payload)
        XCTAssertEqual(decoded.count, 18, "Mini128 should decode to 18 bytes (2 header + 16 payload)")
    }

    func testRoundTrip() throws {
        let embedding = (0..<128).map { Float($0) / 128.0 }

        // Encode
        let elid = try uniEncode(embedding: embedding, profile: .mini128)

        // Decode
        let bytes = try uniDecode(elid: elid)

        // Re-encode the same embedding
        let elid2 = try uniEncode(embedding: embedding, profile: .mini128)

        // Should be identical (deterministic)
        XCTAssertEqual(elid, elid2, "Encoding should be deterministic")
    }

    // MARK: - Hamming Distance Tests

    func testHammingDistance() throws {
        let embedding1 = (0..<128).map { Float($0) / 128.0 }
        let embedding2 = (0..<128).map { Float($0 + 1) / 128.0 }

        let elid1 = try uniEncode(embedding: embedding1, profile: .mini128)
        let elid2 = try uniEncode(embedding: embedding2, profile: .mini128)

        let distance = try uniHammingDistance(elid1: elid1, elid2: elid2)

        // Distance should be between 0 and 128
        XCTAssertGreaterThanOrEqual(distance, 0, "Hamming distance cannot be negative")
        XCTAssertLessThanOrEqual(distance, 128, "Hamming distance cannot exceed 128 for Mini128")
    }

    func testHammingDistanceSameELID() throws {
        let embedding = (0..<128).map { Float($0) / 128.0 }
        let elid = try uniEncode(embedding: embedding, profile: .mini128)

        let distance = try uniHammingDistance(elid1: elid, elid2: elid)

        // Same ELID should have distance 0
        XCTAssertEqual(distance, 0, "Hamming distance of identical ELIDs should be 0")
    }

    // MARK: - Batch Encoding Tests

    func testEncodeBatch() throws {
        let embeddings = (0..<5).map { i in
            (0..<128).map { Float($0 + i) / 128.0 }
        }

        let elids = try uniEncodeBatch(embeddings: embeddings, profile: .mini128)

        XCTAssertEqual(elids.count, 5, "Batch encoding should return 5 ELIDs")

        // All ELIDs should be valid
        for elid in elids {
            XCTAssertEqual(elid.count, 29, "Each ELID should be 29 characters")
        }
    }

    // MARK: - Error Handling Tests

    func testInvalidDimensionError() {
        // Too few dimensions (< 64) should fail
        let embedding = [Float](repeating: 0.5, count: 32)

        XCTAssertThrowsError(try uniEncode(embedding: embedding, profile: .mini128)) { error in
            if case UniElidError.InvalidDimension = error {
                // Expected error
            } else {
                XCTFail("Expected InvalidDimension error, got \(error)")
            }
        }
    }

    func testInvalidEncodingError() {
        // Try to decode an invalid ELID string
        let invalidElid = "INVALID_ELID_STRING_123"

        XCTAssertThrowsError(try uniDecode(elid: invalidElid)) { error in
            if case UniElidError.InvalidEncoding = error {
                // Expected error
            } else if case UniElidError.InvalidHeader = error {
                // Also acceptable
            } else {
                XCTFail("Expected InvalidEncoding or InvalidHeader error, got \(error)")
            }
        }
    }

    func testProfileMismatchError() {
        // Encode with Mini128
        let embedding = (0..<128).map { Float($0) / 128.0 }
        let elid = try! uniEncode(embedding: embedding, profile: .mini128)

        // Try to compute Hamming distance with a Morton ELID
        let embedding2 = (0..<10).map { Float($0) / 10.0 }
        let mortonElid = try! uniEncode(embedding: embedding2, profile: .morton10x10)

        XCTAssertThrowsError(try uniHammingDistance(elid1: elid, elid2: mortonElid)) { error in
            if case UniElidError.ProfileMismatch = error {
                // Expected error
            } else {
                XCTFail("Expected ProfileMismatch error, got \(error)")
            }
        }
    }

    // MARK: - Test Vector Validation

    func testVectorsAgainstReference() throws {
        let vectors = try loadTestVectors()

        print("Testing against \(vectors.vectors.count) reference vectors (version \(vectors.version))")

        for (index, vector) in vectors.vectors.enumerated() {
            // Test Mini128 encoding
            let mini128Elid = try uniEncode(embedding: vector.embedding, profile: .mini128)
            XCTAssertEqual(mini128Elid, vector.mini128,
                "Vector \(index): Mini128 ELID mismatch. Expected \(vector.mini128), got \(mini128Elid)")

            // Test Morton10x10 if available
            if let expectedMorton = vector.morton10x10, vector.embedding.count == 10 {
                let mortonElid = try uniEncode(embedding: vector.embedding, profile: .morton10x10)
                XCTAssertEqual(mortonElid, expectedMorton,
                    "Vector \(index): Morton10x10 ELID mismatch. Expected \(expectedMorton), got \(mortonElid)")
            }

            // Test Hilbert10x10 if available
            if let expectedHilbert = vector.hilbert10x10, vector.embedding.count == 10 {
                let hilbertElid = try uniEncode(embedding: vector.embedding, profile: .hilbert10x10)
                XCTAssertEqual(hilbertElid, expectedHilbert,
                    "Vector \(index): Hilbert10x10 ELID mismatch. Expected \(expectedHilbert), got \(hilbertElid)")
            }

            // Test decode
            let decodedBytes = try uniDecode(elid: mini128Elid)
            XCTAssertFalse(decodedBytes.isEmpty, "Vector \(index): Decoded bytes should not be empty")
        }

        print("✅ All \(vectors.vectors.count) reference vectors validated successfully")
    }

    // MARK: - Performance Tests

    func testEncodingPerformance() throws {
        let embedding = (0..<128).map { Float($0) / 128.0 }

        measure {
            for _ in 0..<1000 {
                _ = try! uniEncode(embedding: embedding, profile: .mini128)
            }
        }
    }

    func testBatchEncodingPerformance() throws {
        let embeddings = (0..<100).map { i in
            (0..<128).map { Float($0 + i) / 128.0 }
        }

        measure {
            _ = try! uniEncodeBatch(embeddings: embeddings, profile: .mini128)
        }
    }

    func testHammingDistancePerformance() throws {
        let embedding1 = (0..<128).map { Float($0) / 128.0 }
        let embedding2 = (0..<128).map { Float($0 + 1) / 128.0 }

        let elid1 = try uniEncode(embedding: embedding1, profile: .mini128)
        let elid2 = try uniEncode(embedding: embedding2, profile: .mini128)

        measure {
            for _ in 0..<1000 {
                _ = try! uniHammingDistance(elid1: elid1, elid2: elid2)
            }
        }
    }
}
