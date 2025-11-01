// Generate UniFFI bindings for Swift, Kotlin, and Ruby
use std::path::PathBuf;
use uniffi_bindgen::generate_bindings;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let udl_file = PathBuf::from("src/elid.udl");
    let config_file = Some(PathBuf::from("uniffi.toml"));
    let out_dir = PathBuf::from("generated");

    // Generate Swift bindings
    println!("Generating Swift bindings...");
    generate_bindings(
        &udl_file,
        config_file.as_deref(),
        vec!["swift".to_string()],
        Some(&out_dir.join("swift")),
        None,
        false,
    )?;

    // Generate Kotlin bindings
    println!("Generating Kotlin bindings...");
    generate_bindings(
        &udl_file,
        config_file.as_deref(),
        vec!["kotlin".to_string()],
        Some(&out_dir.join("kotlin")),
        None,
        false,
    )?;

    // Generate Ruby bindings
    println!("Generating Ruby bindings...");
    generate_bindings(
        &udl_file,
        config_file.as_deref(),
        vec!["ruby".to_string()],
        Some(&out_dir.join("ruby")),
        None,
        false,
    )?;

    println!("All bindings generated successfully!");
    Ok(())
}
