//! Hamming Ball Neighbor Search Example
//!
//! Demonstrates how to use the Hamming ball iterator to find all ELIDs
//! within a specified Hamming distance from a center ELID.
//!
//! This is useful for approximate nearest neighbor search in databases
//! indexed by ELID.

use elid_core::{encode, hamming_distance, hamming_neighbors, Profile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hamming Ball Neighbor Search Demo ===\n");

    // Create a sample embedding (simulating a 768-dim BERT embedding)
    let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
    let embedding: Vec<f32> = embedding.into_iter().cycle().take(768).collect();

    // Encode with Mini128 profile
    let profile = Profile::default();
    let center = encode(&embedding, &profile)?;

    println!("Center ELID: {}\n", center);

    // Demonstrate different radii
    for radius in 0..=3 {
        println!("--- Radius {} ---", radius);

        // Expected neighbor counts
        let expected = match radius {
            0 => 1,
            1 => 1 + 128,
            2 => 1 + 128 + 8128,
            3 => 1 + 128 + 8128 + 341376,
            _ => unreachable!(),
        };

        println!("Expected neighbors: {}", expected);

        // Generate neighbors (for radius 3, only show first 10 for brevity)
        let limit = if radius == 3 { 10 } else { usize::MAX };
        let mut count = 0;

        for neighbor in hamming_neighbors(&center, radius)? {
            count += 1;

            // Show first few neighbors
            if count <= 5 {
                let dist = hamming_distance(&center, &neighbor)?;
                println!("  Neighbor {}: {} (distance: {})", count, neighbor, dist);
            }

            if count >= limit {
                println!("  ... (showing first {} of {} total)", limit, expected);
                break;
            }
        }

        if limit == usize::MAX {
            println!("  Total generated: {}", count);
            assert_eq!(count, expected, "Mismatch in neighbor count!");
        }

        println!();
    }

    // Demonstrate radius validation
    println!("--- Radius Validation ---");
    match hamming_neighbors(&center, 4) {
        Ok(_) => println!("ERROR: Should have rejected radius 4"),
        Err(e) => println!("Correctly rejected radius 4: {}", e),
    }

    println!("\n=== Database Query Pattern ===");
    println!("To find approximate neighbors in a database:\n");
    println!("1. Generate neighbor ELIDs with hamming_neighbors()");
    println!("2. Query database using WHERE elid IN (...) clause");
    println!("3. Results will include all IDs within Hamming distance\n");

    println!("Example SQL:");
    println!("```sql");
    println!("SELECT * FROM embeddings");
    println!("WHERE elid IN (");
    print!("  ");
    for (i, neighbor) in hamming_neighbors(&center, 1)?.take(3).enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("'{}'", neighbor);
    }
    println!(", ...");
    println!(")");
    println!("ORDER BY created_at DESC");
    println!("LIMIT 100;");
    println!("```");

    Ok(())
}
