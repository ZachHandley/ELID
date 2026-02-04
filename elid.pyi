"""Type stubs for the ELID string similarity library."""

from typing import Optional

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
