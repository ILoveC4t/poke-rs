//! Move system hooks and registry.
//!
//! This module extends the generated move identifiers with hook support
//! for conditional move logic (Knock Off, Venoshock, etc.).

// Include generated move identifiers
include!(concat!(env!("OUT_DIR"), "/moves.rs"));

pub mod hooks;
pub mod registry;
pub mod implementations;

pub use hooks::MoveHooks;
pub use registry::MOVE_REGISTRY;
