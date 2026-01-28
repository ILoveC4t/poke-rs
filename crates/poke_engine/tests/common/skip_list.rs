//! Skip list for intentionally skipped test fixtures.
//!
//! Fixtures are skipped when they test incorrect behavior in smogon's calculator
//! that our engine correctly handles. For each skip, add a corresponding
//! correctness test in `tests/` to verify the engine's behavior.

/// Fixtures that are intentionally skipped because they test incorrect behavior.
/// 
/// When adding a fixture to this list:
/// 1. Document WHY it's skipped (what's wrong with the fixture)
/// 2. Create a correctness test in `tests/` that verifies correct behavior
/// 3. Reference the correctness test in the comment
pub const SKIPPED_FIXTURES: &[&str] = &[
    // =========================================================================
    // Arceus Multitype Tests
    // =========================================================================
    // Smogon doesn't apply Multitype's type change, so it calculates damage
    // without STAB. In actual games, Arceus holding a Plate changes type and
    // DOES get STAB on Judgment.
    // Correctness test: tests/multitype_correctness.rs
    "gen4-Arceus-Plate--gen-4--4",
    "gen5-Arceus-Plate--gen-5--5",
    "gen6-Arceus-Plate--gen-6--6",
    "gen7-Arceus-Plate--gen-7--7",
];

/// Check if a fixture should be skipped.
pub fn should_skip(fixture_id: &str) -> bool {
    SKIPPED_FIXTURES.contains(&fixture_id)
}

/// Get the reason a fixture is skipped, if any.
pub fn skip_reason(fixture_id: &str) -> Option<&'static str> {
    if fixture_id.contains("Arceus-Plate") {
        return Some("Smogon fixture doesn't apply Multitype type change (see multitype_correctness.rs)");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skip_check() {
        assert!(should_skip("gen4-Arceus-Plate--gen-4--4"));
        assert!(!should_skip("gen9-some-other-test"));
    }
}
