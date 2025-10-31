//! Command implementations for the ELID CLI.

pub mod decode;
pub mod encode;

// Re-export the command handlers for easy access
pub use decode::handle_decode;
pub use encode::handle_encode;
