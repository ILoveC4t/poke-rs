//! Helper functions for code generation.

use heck::ToPascalCase;

use crate::models::MoveData;

/// Convert a key to a valid Rust identifier in PascalCase.
/// Handles keys starting with digits by prefixing with underscore.
pub fn to_valid_ident(key: &str) -> String {
    let pascal = key.to_pascal_case();
    if pascal.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("_{}", pascal)
    } else {
        pascal
    }
}

/// Check if a move has secondary effects for Sheer Force boost criteria.
/// Returns true if the move has secondary or secondaries fields (that are not null),
/// or if it has the explicit has_sheer_force flag set.
pub fn has_secondary_effects(data: &MoveData) -> bool {
    data.secondary.as_ref().is_some_and(|v| !v.is_null())
        || data.secondaries.as_ref().is_some_and(|v| !v.is_null())
        || data.has_sheer_force.unwrap_or(false)
}
