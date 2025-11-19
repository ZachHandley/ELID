use std::env;

fn main() {
    // Only generate bindings when building with the ffi feature
    if env::var("CARGO_FEATURE_FFI").is_ok() {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        cbindgen::Builder::new()
            .with_crate(crate_dir)
            .with_language(cbindgen::Language::C)
            .with_include_guard("ELID_H")
            .with_documentation(true)
            .with_pragma_once(true)
            .generate()
            .expect("Unable to generate C bindings")
            .write_to_file("elid.h");
    }
}
