//! Item system hooks and registry.

// Include generated item identifiers
include!(concat!(env!("OUT_DIR"), "/items.rs"));

pub mod hooks;
pub mod implementations;
pub mod registry;

pub use hooks::ItemHooks;
pub use registry::ITEM_REGISTRY;
