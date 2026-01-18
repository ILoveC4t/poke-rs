//! Generation 4 (Diamond/Pearl/Platinum, HeartGold/SoulSilver) mechanics.

use super::GenMechanics;

/// Generation 4 mechanics (Pokémon DPPt/HGSS).
///
/// Key features:
/// - Physical/Special split introduced (moves have their own category)
/// - 2.0x crit multiplier
/// - No terrain, no Mega Evolution
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen4;

impl GenMechanics for Gen4 {
    const GEN: u8 = 4;
    
    // 2.0x crit multiplier
    fn crit_multiplier(&self) -> u16 {
        8192
    }
    
    // STAB without Tera
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> u16 {
        if has_adaptability { 8192 } else { 6144 }
    }
    
    // No terrain
    fn terrain_modifier(&self, _terrain: super::Terrain, _move_type: crate::types::Type, _is_grounded: bool) -> Option<u16> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gen4_split() {
        let gen = Gen4;
        // Gen 4 introduced the split
        assert!(gen.uses_physical_special_split());
    }
    
    #[test]
    fn test_gen4_crit() {
        let gen = Gen4;
        assert_eq!(gen.crit_multiplier(), 8192); // 2.0x
    }
}
