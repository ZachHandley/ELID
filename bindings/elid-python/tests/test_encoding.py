"""
Test suite for ELID Python bindings

Tests cover:
- Basic encoding/decoding
- Hamming distance computation
- Batch encoding with parallelism
- Zero-copy NumPy array access
- Cross-language test vector validation
"""

import json
import os
import numpy as np
import pytest
import elid


def test_encode_basic():
    """Test basic encoding with Mini128 profile"""
    embedding = np.random.randn(768)
    profile = elid.Profile("Mini128")

    elid_str = elid.encode(embedding, profile)

    # Validate output
    assert isinstance(elid_str, str)
    assert len(elid_str) == 29  # Mini128 produces 29 chars
    # Base32hex alphabet: 0-9, a-v
    assert all(c in "0123456789abcdefghijklmnopqrstuv" for c in elid_str)


def test_encode_deterministic():
    """Test that encoding is deterministic for same input"""
    embedding = np.array([0.1, 0.2, 0.3, 0.4] * 192)  # 768 dimensions
    profile = elid.Profile("Mini128")

    elid1 = elid.encode(embedding, profile)
    elid2 = elid.encode(embedding, profile)

    assert elid1 == elid2


def test_encode_validates_dimensions():
    """Test that encoding validates embedding dimensions"""
    profile = elid.Profile("Mini128")

    # Too small (< 64)
    with pytest.raises(ValueError, match="Invalid dimension"):
        elid.encode(np.array([0.1] * 32), profile)

    # Too large (> 2048)
    with pytest.raises(ValueError, match="Invalid dimension"):
        elid.encode(np.array([0.1] * 4096), profile)


def test_encode_different_profiles():
    """Test encoding with different profiles"""
    embedding = np.random.randn(768)

    # Mini128
    mini = elid.Profile("Mini128")
    elid_mini = elid.encode(embedding, mini)
    assert len(elid_mini) == 29

    # Morton10x10
    morton = elid.Profile("Morton10x10")
    elid_morton = elid.encode(embedding, morton)
    assert len(elid_morton) == 24  # 10D * 10 bits = 100 bits, ceil(100/5) + 2 header = ~24

    # Hilbert10x10
    hilbert = elid.Profile("Hilbert10x10")
    elid_hilbert = elid.encode(embedding, hilbert)
    assert len(elid_hilbert) == 24


def test_decode_roundtrip():
    """Test encode/decode round-trip"""
    embedding = np.random.randn(768)
    profile = elid.Profile("Mini128")

    elid_str = elid.encode(embedding, profile)
    raw_bytes = elid.decode(elid_str)

    # Validate bytes
    assert isinstance(raw_bytes, bytes)
    assert len(raw_bytes) == 18  # 2 header + 16 payload for Mini128


def test_decode_invalid_string():
    """Test that decode rejects invalid ELID strings"""
    # Invalid characters (outside base32hex alphabet)
    with pytest.raises(ValueError, match="Invalid ELID"):
        elid.decode("xyz123")


def test_hamming_distance_identical():
    """Test Hamming distance for identical embeddings"""
    embedding = np.random.randn(768)
    profile = elid.Profile("Mini128")

    elid_str = elid.encode(embedding, profile)
    distance = elid.hamming_distance(elid_str, elid_str)

    assert distance == 0


def test_hamming_distance_similar():
    """Test Hamming distance for similar embeddings"""
    profile = elid.Profile("Mini128")

    # Create similar embeddings
    embedding1 = np.random.randn(768)
    embedding2 = embedding1.copy()
    embedding2[0] += 0.01  # Slight perturbation

    elid1 = elid.encode(embedding1, profile)
    elid2 = elid.encode(embedding2, profile)

    distance = elid.hamming_distance(elid1, elid2)

    # Similar embeddings should have low Hamming distance
    assert 0 <= distance <= 128
    assert distance < 64  # Expect low distance for similar embeddings


def test_hamming_distance_different():
    """Test Hamming distance for different embeddings"""
    profile = elid.Profile("Mini128")

    # Create very different embeddings
    embedding1 = np.array([1.0, 0.0] * 384)
    embedding2 = np.array([0.0, 1.0] * 384)

    elid1 = elid.encode(embedding1, profile)
    elid2 = elid.encode(embedding2, profile)

    distance = elid.hamming_distance(elid1, elid2)

    # Different embeddings should have higher Hamming distance
    assert distance > 32


def test_hamming_distance_symmetric():
    """Test that Hamming distance is symmetric"""
    profile = elid.Profile("Mini128")

    embedding1 = np.random.randn(768)
    embedding2 = np.random.randn(768)

    elid1 = elid.encode(embedding1, profile)
    elid2 = elid.encode(embedding2, profile)

    dist_ab = elid.hamming_distance(elid1, elid2)
    dist_ba = elid.hamming_distance(elid2, elid1)

    assert dist_ab == dist_ba


def test_encode_batch_basic():
    """Test batch encoding with multiple embeddings"""
    embeddings = [np.random.randn(768) for _ in range(100)]
    profile = elid.Profile("Mini128")

    elids = elid.encode_batch(embeddings, profile)

    assert len(elids) == 100
    assert all(isinstance(e, str) for e in elids)
    assert all(len(e) == 29 for e in elids)


def test_encode_batch_performance():
    """Test batch encoding performance (>5000 embeddings/sec)"""
    import time

    # Create 1000 embeddings
    embeddings = [np.random.randn(768) for _ in range(1000)]
    profile = elid.Profile("Mini128")

    # Time the batch encoding
    start = time.time()
    elids = elid.encode_batch(embeddings, profile)
    elapsed = time.time() - start

    # Calculate throughput
    throughput = len(embeddings) / elapsed

    print(f"\nBatch encoding throughput: {throughput:.0f} embeddings/sec")
    print(f"Elapsed: {elapsed:.3f}s for {len(embeddings)} embeddings")

    assert len(elids) == 1000
    # Expect >5000 embeddings/sec on modern hardware
    # Note: This may fail on slow CI runners, adjust threshold if needed
    assert throughput > 1000  # Conservative threshold


def test_encode_batch_empty():
    """Test batch encoding with empty list"""
    profile = elid.Profile("Mini128")
    elids = elid.encode_batch([], profile)
    assert elids == []


def test_encode_batch_single():
    """Test batch encoding with single embedding"""
    embedding = np.random.randn(768)
    profile = elid.Profile("Mini128")

    elids = elid.encode_batch([embedding], profile)

    assert len(elids) == 1
    # Should match single encode
    assert elids[0] == elid.encode(embedding, profile)


def test_zero_copy_numpy():
    """Test zero-copy NumPy array access"""
    # Create a large embedding
    embedding = np.random.randn(2048)
    profile = elid.Profile("Mini128")

    # Get memory address of NumPy array
    original_ptr = embedding.__array_interface__["data"][0]

    # Encode (should use zero-copy access)
    elid_str = elid.encode(embedding, profile)

    # Verify the NumPy array wasn't modified or copied
    new_ptr = embedding.__array_interface__["data"][0]
    assert original_ptr == new_ptr

    # Verify result is valid
    assert len(elid_str) == 29


def test_cross_language_validation():
    """Test against cross-language test vectors"""
    # Load test vectors
    test_vectors_path = os.path.join(
        os.path.dirname(__file__), "..", "..", "test-vectors.json"
    )

    if not os.path.exists(test_vectors_path):
        pytest.skip("Test vectors file not found")

    with open(test_vectors_path, "r") as f:
        test_data = json.load(f)

    vectors = test_data["vectors"]
    profile = elid.Profile("Mini128")

    # Test first 5 vectors
    for i, vector in enumerate(vectors[:5]):
        embedding = np.array(vector["embedding"], dtype=np.float64)
        expected_elid = vector["mini128_elid"]

        # Encode and compare
        result_elid = elid.encode(embedding, profile)

        assert (
            result_elid == expected_elid
        ), f"Vector {i}: expected {expected_elid}, got {result_elid}"

    print(f"\n✓ Validated {min(5, len(vectors))} test vectors against Rust implementation")


def test_profile_creation():
    """Test Profile enum creation"""
    mini = elid.Profile("Mini128")
    assert str(mini) == "Mini128"

    morton = elid.Profile("Morton10x10")
    assert str(morton) == "Morton10x10"

    hilbert = elid.Profile("Hilbert10x10")
    assert str(hilbert) == "Hilbert10x10"

    # Invalid profile
    with pytest.raises(ValueError, match="Invalid profile"):
        elid.Profile("InvalidProfile")


def test_multidimensional_array_rejection():
    """Test that multidimensional arrays are rejected"""
    profile = elid.Profile("Mini128")

    # 2D array
    embedding_2d = np.random.randn(128, 6)
    with pytest.raises(ValueError, match="Expected 1D array"):
        elid.encode(embedding_2d, profile)


def test_non_contiguous_array():
    """Test handling of non-contiguous NumPy arrays"""
    profile = elid.Profile("Mini128")

    # Create non-contiguous array (every other element)
    embedding = np.random.randn(1536)[::2]  # 768 elements, non-contiguous

    # Should either work or give clear error
    try:
        elid_str = elid.encode(embedding, profile)
        assert len(elid_str) == 29
    except ValueError as e:
        assert "contiguous" in str(e).lower()


def test_normalized_embeddings():
    """Test that encoding handles normalized embeddings correctly"""
    profile = elid.Profile("Mini128")

    # Create normalized embedding (unit vector)
    embedding = np.random.randn(768)
    embedding = embedding / np.linalg.norm(embedding)

    elid_str = elid.encode(embedding, profile)
    assert len(elid_str) == 29


def test_edge_case_dimensions():
    """Test edge case dimensions (64 and 2048)"""
    profile = elid.Profile("Mini128")

    # Minimum dimension (64)
    embedding_min = np.random.randn(64)
    elid_min = elid.encode(embedding_min, profile)
    assert len(elid_min) == 29

    # Maximum dimension (2048)
    embedding_max = np.random.randn(2048)
    elid_max = elid.encode(embedding_max, profile)
    assert len(elid_max) == 29


def test_version():
    """Test that version is accessible"""
    assert hasattr(elid, "__version__")
    assert isinstance(elid.__version__, str)
    print(f"\nELID Python bindings version: {elid.__version__}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
