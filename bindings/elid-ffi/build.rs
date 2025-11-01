fn main() {
    // Generate UniFFI scaffolding from the UDL file
    uniffi::generate_scaffolding("src/elid.udl").unwrap();
}
