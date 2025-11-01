"""Type stubs for ELID Python bindings"""

from enum import Enum
from typing import List
import numpy as np
import numpy.typing as npt

__version__: str

class Profile:
    """ELID encoding profile"""
    Mini128: Profile
    Morton10x10: Profile
    Hilbert10x10: Profile

    def __init__(self, name: str) -> None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

def encode(
    embedding: npt.NDArray[np.float32],
    profile: Profile
) -> str:
    """
    Encode a high-dimensional embedding to a sortable string identifier.

    Args:
        embedding: NumPy array of shape (N,) where 64 ≤ N ≤ 2048
        profile: Encoding profile (Mini128, Morton10x10, or Hilbert10x10)

    Returns:
        Sortable string identifier (29 chars for Mini128, ~24 for others)

    Raises:
        ValueError: If embedding dimensions are invalid or encoding fails

    Examples:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.random.randn(768)
        >>> elid_id = elid.encode(embedding, elid.Profile.Mini128)
        >>> len(elid_id)
        29
    """
    ...

def decode(elid_str: str) -> bytes:
    """
    Decode an ELID string back to raw bytes.

    Args:
        elid_str: ELID string identifier (base32hex encoded)

    Returns:
        Raw bytes (18 bytes for Mini128: 2 header + 16 payload)

    Raises:
        ValueError: If ELID string is malformed or invalid encoding

    Examples:
        >>> import elid
        >>> raw_bytes = elid.decode("0123456789abcdefghijklmnop")
        >>> len(raw_bytes)
        18
    """
    ...

def hamming_distance(elid1: str, elid2: str) -> int:
    """
    Calculate Hamming distance between two ELIDs.

    Args:
        elid1: First ELID string
        elid2: Second ELID string (must use Mini128 profile)

    Returns:
        Hamming distance (0-128 for Mini128)
        Lower distance indicates higher similarity.

    Raises:
        ValueError: If ELIDs use different profiles or are malformed

    Examples:
        >>> import elid
        >>> import numpy as np
        >>> embedding = np.ones(768)
        >>> elid1 = elid.encode(embedding, elid.Profile.Mini128)
        >>> elid2 = elid.encode(embedding, elid.Profile.Mini128)
        >>> elid.hamming_distance(elid1, elid2)
        0
    """
    ...

def encode_batch(
    embeddings: List[npt.NDArray[np.float32]],
    profile: Profile
) -> List[str]:
    """
    Encode multiple embeddings in parallel using Rayon.

    Args:
        embeddings: List of NumPy arrays, each shape (N,) where 64 ≤ N ≤ 2048
        profile: Encoding profile for all embeddings

    Returns:
        List of ELID strings, same order as input

    Raises:
        ValueError: If any embedding has invalid dimensions

    Examples:
        >>> import elid
        >>> import numpy as np
        >>> embeddings = [np.random.randn(768) for _ in range(1000)]
        >>> elids = elid.encode_batch(embeddings, elid.Profile.Mini128)
        >>> len(elids)
        1000
    """
    ...
