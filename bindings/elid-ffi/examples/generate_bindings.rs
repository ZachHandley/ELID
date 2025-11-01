// Generate UniFFI bindings for Swift, Kotlin, and Ruby
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let udl_file = crate_root.join("src/elid.udl");
    let config_file = crate_root.join("uniffi.toml");
    let out_dir = crate_root.join("generated");

    // Ensure output directories exist
    fs::create_dir_all(out_dir.join("swift"))?;
    fs::create_dir_all(out_dir.join("kotlin"))?;
    fs::create_dir_all(out_dir.join("ruby"))?;

    println!("UDL file: {}", udl_file.display());
    println!("Config file: {}", config_file.display());
    println!("Output directory: {}", out_dir.display());

    // Use uniffi_bindgen library to generate bindings
    use uniffi_bindgen::bindings::{swift, kotlin, ruby};

    // Load the UDL file
    let udl_content = fs::read_to_string(&udl_file)?;

    println!("\nNote: For actual binding generation, you need to use the uniffi-bindgen CLI:");
    println!("  cargo install uniffi-bindgen --version 0.28  # or use the CLI from uniffi crate");
    println!("\nOr generate manually with:");
    println!("  uniffi-bindgen generate {} --language swift --out-dir {}/swift",
             udl_file.display(), out_dir.display());
    println!("  uniffi-bindgen generate {} --language kotlin --out-dir {}/kotlin",
             udl_file.display(), out_dir.display());
    println!("  uniffi-bindgen generate {} --language ruby --out-dir {}/ruby",
             udl_file.display(), out_dir.display());

    Ok(())
}
