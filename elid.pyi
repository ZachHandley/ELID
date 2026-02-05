"""Type stubs for the ELID string similarity library."""

from typing import Optional
from enum import Enum

import numpy as np
import numpy.typing as npt

__version__: str

class SimilarityOpts:
    """Options for configuring string similarity algorithms.

    Attributes:
        case_sensitive: Case-sensitive comparison (default: True)
        trim_whitespace: Trim whitespace before comparison (default: False)
        prefix_scale: Prefix scale for Jaro-Winkler (default: 0.1, max: 0.25)
    """

    case_sensitive: bool
    trim_whitespace: bool
    prefix_scale: float

    def __init__(
        self,
        case_sensitive: bool = True,
        trim_whitespace: bool = False,
        prefix_scale: float = 0.1,
    ) -> None: ...
    def __repr__(self) -> str: ...

class MatchResult:
    """Result from a match operation."""

    index: int
    score: float

def levenshtein(a: str, b: str) -> int:
    """Compute the Levenshtein distance between two strings.

    Args:
        a: First string
        b: Second string

    Returns:
        The minimum number of single-character edits needed to transform one string into another.
    """
    ...

def normalized_levenshtein(a: str, b: str) -> float:
    """Compute the normalized Levenshtein similarity between two strings.

    Args:
        a: First string
        b: Second string

    Returns:
        Similarity score between 0.0 (completely different) and 1.0 (identical).
    """
    ...

def jaro(a: str, b: str) -> float:
    """Compute the Jaro similarity between two strings.

    Args:
        a: First string
        b: Second string

    Returns:
        Similarity score between 0.0 and 1.0. Particularly effective for short strings.
    """
    ...

def jaro_winkler(a: str, b: str) -> float:
    """Compute the Jaro-Winkler similarity between two strings.

    Gives more favorable ratings to strings with common prefixes.

    Args:
        a: First string
        b: Second string

    Returns:
        Similarity score between 0.0 and 1.0.
    """
    ...

def hamming(a: str, b: str) -> Optional[int]:
    """Compute the Hamming distance between two strings.

    Args:
        a: First string
        b: Second string

    Returns:
        Number of positions at which characters differ, or None if lengths differ.
    """
    ...

def osa_distance(a: str, b: str) -> int:
    """Compute the OSA (Optimal String Alignment) distance between two strings.

    Similar to Levenshtein but also considers transpositions as a single operation.

    Args:
        a: First string
        b: Second string

    Returns:
        OSA distance.
    """
    ...

def best_match(a: str, b: str) -> float:
    """Compute the best matching similarity between two strings.

    Runs multiple algorithms and returns the highest score.

    Args:
        a: First string
        b: Second string

    Returns:
        Best similarity score between 0.0 and 1.0.
    """
    ...

def find_best_match(query: str, candidates: list[str]) -> MatchResult:
    """Find the best match for a query string in a list of candidates.

    Args:
        query: Query string
        candidates: List of candidate strings

    Returns:
        Dict with 'index' and 'score' keys.
    """
    ...

def find_matches_above_threshold(
    query: str, candidates: list[str], threshold: float
) -> list[MatchResult]:
    """Find all matches above a threshold score.

    Args:
        query: Query string
        candidates: List of candidate strings
        threshold: Minimum similarity score (0.0 to 1.0)

    Returns:
        List of dicts with 'index' and 'score' keys.
    """
    ...

def levenshtein_with_opts(a: str, b: str, opts: SimilarityOpts) -> int:
    """Compute Levenshtein distance with custom options.

    Args:
        a: First string
        b: Second string
        opts: Configuration options

    Returns:
        Levenshtein distance.
    """
    ...

def simhash(text: str) -> int:
    """Compute the SimHash fingerprint of a string.

    Returns a 64-bit integer hash where similar strings produce similar hashes.

    Args:
        text: Input string

    Returns:
        64-bit hash value.
    """
    ...

def simhash_distance(hash1: int, hash2: int) -> int:
    """Compute the Hamming distance between two SimHash values.

    Args:
        hash1: First SimHash value
        hash2: Second SimHash value

    Returns:
        Hamming distance (0-64). Lower values indicate higher similarity.
    """
    ...

def simhash_similarity(a: str, b: str) -> float:
    """Compute the normalized SimHash similarity between two strings.

    Args:
        a: First string
        b: Second string

    Returns:
        Similarity score between 0.0 and 1.0.
    """
    ...

def find_similar_hashes(
    query_hash: int, candidate_hashes: list[int], max_distance: int
) -> list[int]:
    """Find all hashes within a given distance threshold.

    Args:
        query_hash: The query SimHash value
        candidate_hashes: List of candidate SimHash values
        max_distance: Maximum Hamming distance threshold

    Returns:
        Indices of candidates within the distance threshold.
    """
    ...

# ============================================================================
# Embedding functions (available when built with embeddings feature)
# ============================================================================

class Profile(Enum):
    """Encoding profile for embedding vectors.

    Profiles determine how embeddings are transformed into compact identifiers.

    Variants:
        Mini128: 128-bit SimHash (default, fast cosine similarity via Hamming distance)
        Morton10x10: Z-order curve encoding for database indexing
        Hilbert10x10: Hilbert curve encoding for maximum locality preservation
    """

    Mini128 = ...
    """128-bit SimHash encoding"""
    Morton10x10 = ...
    """Morton (Z-order) curve encoding with 10 dimensions x 10 bits"""
    Hilbert10x10 = ...
    """Hilbert curve encoding with 10 dimensions x 10 bits"""

def encode(embedding: npt.NDArray[np.float32], profile: Profile) -> str:
    """Encode an embedding vector to an ELID string.

    Converts a high-dimensional embedding vector into a compact, sortable identifier
    using the specified profile. The resulting ELID preserves locality properties
    for efficient similarity search.

    Args:
        embedding: Input vector (f32, 64-2048 dimensions)
        profile: Encoding strategy (Mini128, Morton10x10, or Hilbert10x10)

    Returns:
        Encoded ELID string

    Raises:
        ValueError: If embedding dimensions are invalid or values contain NaN/Inf

    Example:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.random.randn(768).astype(np.float32)
        >>> elid_str = elid.encode(embedding, elid.Profile.Mini128)
    """
    ...

def decode(elid_str: str) -> bytes:
    """Decode an ELID string to raw bytes.

    Decodes a base32hex-encoded ELID string back to its raw byte representation.
    This returns the header bytes + payload bytes.

    Args:
        elid_str: The ELID string to decode

    Returns:
        Raw bytes (header + payload)

    Raises:
        ValueError: If the ELID string contains invalid characters

    Example:
        >>> raw_bytes = elid.decode("01a2b3c4d5e6f7...")
        >>> len(raw_bytes)  # 18 for Mini128 (2 header + 16 payload)
    """
    ...

def elid_hamming_distance(elid1: str, elid2: str) -> int:
    """Compute Hamming distance between two ELID strings.

    Returns the number of differing bits in the SimHash payloads of two ELIDs.
    This distance is proportional to the angular distance between the original
    embeddings. Both ELIDs must use the Mini128 profile.

    Args:
        elid1: First ELID string
        elid2: Second ELID string

    Returns:
        Hamming distance (0-128)

    Raises:
        ValueError: If either ELID is invalid or uses a non-Mini128 profile

    Example:
        >>> import elid
        >>> import numpy as np
        >>> emb1 = np.random.randn(768).astype(np.float32)
        >>> emb2 = emb1 + np.random.randn(768).astype(np.float32) * 0.1
        >>> elid1 = elid.encode(emb1, elid.Profile.Mini128)
        >>> elid2 = elid.encode(emb2, elid.Profile.Mini128)
        >>> distance = elid.elid_hamming_distance(elid1, elid2)
    """
    ...
