"""
ELID: Embedding Locality IDentifier

Compact, sortable identifiers for high-dimensional embeddings.
"""

from .elid import (
    Profile,
    encode,
    decode,
    hamming_distance,
    encode_batch,
    __version__,
)

__all__ = [
    "Profile",
    "encode",
    "decode",
    "hamming_distance",
    "encode_batch",
    "__version__",
]
