"""
Example: Encoding transformer embeddings with ELID

This example demonstrates how to use ELID with sentence transformers
for semantic similarity search using Hamming distance.
"""

import elid
import numpy as np
from typing import List, Tuple


def simulate_transformer_embeddings(texts: List[str], dim: int = 768) -> List[np.ndarray]:
    """
    Simulate transformer embeddings (in production, use sentence-transformers or similar).

    Args:
        texts: List of text strings
        dim: Embedding dimension (768 for BERT, 1536 for OpenAI text-embedding-3-small)

    Returns:
        List of NumPy arrays representing embeddings
    """
    # In production, use:
    # from sentence_transformers import SentenceTransformer
    # model = SentenceTransformer('all-MiniLM-L6-v2')
    # return model.encode(texts)

    # For demo purposes, generate random embeddings
    embeddings = []
    for text in texts:
        # Simulate different embeddings for different texts
        seed = sum(ord(c) for c in text)
        np.random.seed(seed)
        embedding = np.random.randn(dim)
        # Normalize to unit length (typical for transformer embeddings)
        embedding = embedding / np.linalg.norm(embedding)
        embeddings.append(embedding)

    return embeddings


def main():
    print("=" * 60)
    print("ELID Transformer Encoding Example")
    print("=" * 60)

    # Sample documents
    documents = [
        "The quick brown fox jumps over the lazy dog",
        "A fast brown fox leaps over a sleepy dog",  # Similar to first
        "Python is a popular programming language",
        "Machine learning models process data",
        "Deep learning uses neural networks",  # Related to ML
        "The weather is sunny today",
        "Climate change affects global temperatures",  # Related to weather
    ]

    print(f"\nEncoding {len(documents)} documents...")

    # Generate embeddings (in production, use actual transformer model)
    embeddings = simulate_transformer_embeddings(documents, dim=768)
    print(f"✓ Generated {len(embeddings)} embeddings (768D)")

    # Encode with ELID using Mini128 profile (optimized for similarity search)
    profile = elid.Profile("Mini128")

    # Method 1: Batch encoding (faster for multiple embeddings)
    print(f"\nMethod 1: Batch encoding with encode_batch()...")
    elids = elid.encode_batch(embeddings, profile)
    print(f"✓ Encoded {len(elids)} ELIDs in batch")

    # Print results
    print("\n" + "=" * 60)
    print("Documents and their ELIDs:")
    print("=" * 60)
    for i, (doc, elid_str) in enumerate(zip(documents, elids)):
        print(f"{i+1}. {doc[:50]:50s} -> {elid_str}")

    # Find similar documents using Hamming distance
    print("\n" + "=" * 60)
    print("Similarity Search (using Hamming distance)")
    print("=" * 60)

    query_idx = 0  # Use first document as query
    query_elid = elids[query_idx]
    query_text = documents[query_idx]

    print(f"\nQuery: {query_text}")
    print(f"Query ELID: {query_elid}\n")

    # Compute distances to all other documents
    similarities: List[Tuple[int, str, int]] = []
    for i, (doc, doc_elid) in enumerate(zip(documents, elids)):
        if i == query_idx:
            continue

        distance = elid.hamming_distance(query_elid, doc_elid)
        similarities.append((i, doc, distance))

    # Sort by distance (ascending = most similar first)
    similarities.sort(key=lambda x: x[2])

    print("Top 3 most similar documents:")
    for rank, (idx, doc, distance) in enumerate(similarities[:3], 1):
        # Convert Hamming distance to approximate cosine similarity
        # cos(θ) ≈ cos(π * distance / 128)
        approx_cosine = np.cos(np.pi * distance / 128)

        print(f"\n{rank}. Document {idx+1} (Hamming distance: {distance}/128)")
        print(f"   Approximate cosine similarity: {approx_cosine:.3f}")
        print(f"   Text: {doc[:60]}...")

    # Demonstrate decode
    print("\n" + "=" * 60)
    print("Decoding Example")
    print("=" * 60)

    first_elid = elids[0]
    raw_bytes = elid.decode(first_elid)

    print(f"ELID: {first_elid}")
    print(f"Raw bytes: {raw_bytes.hex()}")
    print(f"Length: {len(raw_bytes)} bytes (2 header + 16 payload)")

    # Decode header
    version = (raw_bytes[0] & 0xF0) >> 4
    profile_type = raw_bytes[0] & 0x0F

    print(f"\nHeader:")
    print(f"  Version: {version}")
    print(f"  Profile type: 0x{profile_type:02x} (Mini128)")

    # Method 2: Single encoding (for comparison)
    print("\n" + "=" * 60)
    print("Method 2: Single encoding (for individual embeddings)")
    print("=" * 60)

    single_embedding = embeddings[0]
    single_elid = elid.encode(single_embedding, profile)

    print(f"Single encode result: {single_elid}")
    print(f"Batch encode result:  {elids[0]}")
    print(f"Match: {single_elid == elids[0]}")

    # Database use case example
    print("\n" + "=" * 60)
    print("Database Use Case Example")
    print("=" * 60)

    print("\nUsing ELIDs as sortable primary keys in a vector database:")
    print("```sql")
    print("CREATE TABLE documents (")
    print("    elid TEXT PRIMARY KEY,")
    print("    content TEXT,")
    print("    embedding BLOB")
    print(");")
    print("")
    print("-- Range query for approximate nearest neighbors")
    print("SELECT elid, content FROM documents")
    print(f"WHERE elid BETWEEN '{query_elid[:10]}' AND '{query_elid[:10]}z'")
    print("ORDER BY elid")
    print("LIMIT 100;")
    print("```")

    print("\n" + "=" * 60)
    print("Performance Tips")
    print("=" * 60)

    print("""
1. Use encode_batch() for multiple embeddings (>10x faster)
2. Pre-normalize embeddings to unit length for consistency
3. Use Mini128 profile for similarity search (Hamming distance)
4. Use Morton10x10 for database indexing (sortability)
5. Store ELIDs as PRIMARY KEY for efficient B-tree indexing
6. Use ELID prefixes for approximate nearest neighbor search
    """)


if __name__ == "__main__":
    main()
