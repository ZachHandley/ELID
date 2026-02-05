"""Type stubs for the ELID string similarity library."""

from typing import Optional, TypedDict
from enum import Enum

import numpy as np
import numpy.typing as npt

__version__: str


class ElidMetadata(TypedDict, total=False):
    """Metadata about a FullVector ELID encoding."""

    original_dims: int
    """Original embedding dimension count"""
    encoded_dims: int
    """Number of dimensions in encoded representation"""
    is_lossless: bool
    """Whether exact reconstruction is possible"""
    has_dimension_reduction: bool
    """Whether dimensions were reduced"""
    precision: str
    """Precision type: 'Full32', 'Half16', 'Quant8', or 'Bits'"""
    precision_bits: int
    """Bit count if precision is 'Bits' (optional)"""
    dimension_mode: str
    """Dimension mode: 'Preserve', 'Reduce', or 'Common'"""

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


# ============================================================================
# FullVector encoding types and functions
# ============================================================================


class VectorPrecision(Enum):
    """Precision options for full vector encoding.

    Controls how many bits are used to represent each dimension value.
    Higher precision means more accurate reconstruction but larger output.

    Variants:
        Full32: Full 32-bit float (lossless, 4 bytes per dimension)
        Half16: 16-bit half-precision float (2 bytes per dimension)
        Quant8: 8-bit quantized (1 byte per dimension, ~1% error)
    """

    Full32 = ...
    """Full 32-bit float (lossless)"""
    Half16 = ...
    """16-bit half-precision float"""
    Quant8 = ...
    """8-bit quantized (~1% error)"""


class DimensionMode(Enum):
    """Dimension handling mode for full vector encoding.

    Controls whether to preserve original dimensions, reduce them,
    or project to a common space for cross-dimensional comparison.

    Variants:
        Preserve: Keep all original dimensions (no projection)
        Reduce: Reduce dimensions using random projection
        Common: Project to common space for cross-dimensional comparison
    """

    Preserve = ...
    """Preserve all original dimensions"""
    Reduce = ...
    """Reduce dimensions using random projection"""
    Common = ...
    """Project to common space for cross-dimensional comparison"""


def encode_lossless(embedding: npt.NDArray[np.float32]) -> str:
    """Encode an embedding using lossless full vector encoding.

    Preserves the exact embedding values (32-bit float precision) and all dimensions.
    This produces the largest output but allows exact reconstruction.

    Args:
        embedding: Input vector (f32, 64-2048 dimensions)

    Returns:
        Encoded ELID string that can be decoded back to the original embedding

    Raises:
        ValueError: If embedding dimensions are invalid or values contain NaN/Inf

    Example:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.random.randn(768).astype(np.float32)
        >>> elid_str = elid.encode_lossless(embedding)
        >>> recovered = elid.decode_to_embedding(elid_str)
        >>> np.allclose(embedding, recovered)  # True
    """
    ...


def encode_compressed(
    embedding: npt.NDArray[np.float32], retention_pct: float
) -> str:
    """Encode an embedding with percentage-based compression.

    The retention percentage (0.0-1.0) controls how much information is preserved:
    - 1.0 = lossless (Full32 precision, all dimensions)
    - 0.5 = half precision and/or half dimensions
    - 0.25 = quarter precision and/or quarter dimensions

    The algorithm optimizes for dimension reduction first (which preserves
    more geometric relationships) before reducing precision.

    Args:
        embedding: Input vector (f32, 64-2048 dimensions)
        retention_pct: Information retention percentage (0.0-1.0)

    Returns:
        Encoded ELID string

    Raises:
        ValueError: If embedding dimensions are invalid or values contain NaN/Inf

    Example:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.random.randn(768).astype(np.float32)
        >>> elid_50 = elid.encode_compressed(embedding, 0.5)   # 50% retention
        >>> elid_25 = elid.encode_compressed(embedding, 0.25)  # 25% retention
        >>> len(elid_25) < len(elid_50)  # True (smaller output)
    """
    ...


def encode_max_length(
    embedding: npt.NDArray[np.float32], max_chars: int
) -> str:
    """Encode an embedding with a maximum output string length constraint.

    Calculates the optimal precision and dimension settings to fit within
    the specified character limit while maximizing fidelity.

    Args:
        embedding: Input vector (f32, 64-2048 dimensions)
        max_chars: Maximum output string length in characters

    Returns:
        Encoded ELID string guaranteed to be <= max_chars in length

    Raises:
        ValueError: If embedding dimensions are invalid or values contain NaN/Inf

    Example:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.random.randn(768).astype(np.float32)
        >>> elid_str = elid.encode_max_length(embedding, 100)
        >>> len(elid_str) <= 100  # True
    """
    ...


def decode_to_embedding(elid_str: str) -> Optional[npt.NDArray[np.float32]]:
    """Decode an ELID string back to an embedding vector.

    Only works for ELIDs encoded with a FullVector profile (lossless,
    compressed, or max_length). Returns None for non-reversible profiles
    like Mini128, Morton, or Hilbert.

    Args:
        elid_str: A valid ELID string (base32hex encoded)

    Returns:
        Decoded embedding as f32 array, or None if not reversible

    Note:
        If dimension reduction was used during encoding, the decoded embedding
        will be in the reduced dimension space, not the original.

    Example:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.random.randn(768).astype(np.float32)
        >>> elid_str = elid.encode_lossless(embedding)
        >>> recovered = elid.decode_to_embedding(elid_str)
        >>> recovered is not None  # True
        >>> np.allclose(embedding, recovered)  # True
    """
    ...


def is_reversible(elid_str: str) -> bool:
    """Check if an ELID can be decoded back to an embedding.

    Returns True if the ELID was encoded with a FullVector profile
    (lossless, compressed, or max_length), False otherwise.

    Args:
        elid_str: A valid ELID string (base32hex encoded)

    Returns:
        True if decode_to_embedding will return an embedding

    Raises:
        ValueError: If the ELID string is invalid

    Example:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.random.randn(768).astype(np.float32)
        >>>
        >>> # Mini128 is NOT reversible
        >>> mini_elid = elid.encode(embedding, elid.Profile.Mini128)
        >>> elid.is_reversible(mini_elid)  # False
        >>>
        >>> # Lossless IS reversible
        >>> lossless_elid = elid.encode_lossless(embedding)
        >>> elid.is_reversible(lossless_elid)  # True
    """
    ...


def encode_cross_dimensional(
    embedding: npt.NDArray[np.float32], common_dims: int
) -> str:
    """Encode an embedding for cross-dimensional comparison.

    Projects the embedding to a common dimension space, allowing comparison
    between embeddings of different original dimensions (e.g., 256d vs 768d).

    Args:
        embedding: Input vector (f32, 64-2048 dimensions)
        common_dims: Target dimension space (all vectors projected here)

    Returns:
        Encoded ELID string

    Raises:
        ValueError: If embedding dimensions are invalid or values contain NaN/Inf

    Example:
        >>> import elid
        >>> import numpy as np
        >>> # Different sized embeddings from different models
        >>> emb_256 = np.random.randn(256).astype(np.float32)
        >>> emb_768 = np.random.randn(768).astype(np.float32)
        >>>
        >>> # Project both to 128-dim common space
        >>> elid1 = elid.encode_cross_dimensional(emb_256, 128)
        >>> elid2 = elid.encode_cross_dimensional(emb_768, 128)
        >>>
        >>> # Now they can be compared directly
        >>> dec1 = elid.decode_to_embedding(elid1)
        >>> dec2 = elid.decode_to_embedding(elid2)
        >>> dec1.shape == dec2.shape  # True (both 128,)
    """
    ...


def get_metadata(elid_str: str) -> Optional[ElidMetadata]:
    """Get metadata about a FullVector ELID.

    Returns a dictionary containing information about how the ELID was encoded,
    including original dimensions, precision, and dimension mode.

    Args:
        elid_str: A valid ELID string (base32hex encoded)

    Returns:
        Metadata dictionary, or None if not a FullVector ELID.
        The dictionary contains:
        - original_dims (int): Original embedding dimension count
        - encoded_dims (int): Number of dimensions in encoded representation
        - is_lossless (bool): Whether exact reconstruction is possible
        - has_dimension_reduction (bool): Whether dimensions were reduced
        - precision (str): "Full32", "Half16", "Quant8", or "Bits"
        - precision_bits (int, optional): Bit count if precision is "Bits"
        - dimension_mode (str): "Preserve", "Reduce", or "Common"

    Raises:
        ValueError: If the ELID string is invalid

    Example:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.random.randn(768).astype(np.float32)
        >>> elid_str = elid.encode_compressed(embedding, 0.5)
        >>> meta = elid.get_metadata(elid_str)
        >>> print(meta['original_dims'])  # 768
        >>> print(meta['is_lossless'])    # False
    """
    ...
