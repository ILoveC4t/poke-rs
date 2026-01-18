//! Generation 5 (Black/White, Black 2/White 2) mechanics.

use super::GenMechanics;

/// Generation 5 mechanics (Pokémon Black/White/B2W2).
///
/// Key differences from Gen 6:
/// - Critical hits are 2.0x (not 1.5x)
/// - No Mega Evolution
/// - No Terrain
#[derive(Clone, Copy, Debug, Default)]
pub struct Gen5;

impl GenMechanics for Gen5 {
    const GEN: u8 = 5;
    
    // 2.0x crit multiplier
    fn crit_multiplier(&self) -> u16 {
        8192 // 2.0x
    }
    
    // STAB without Tera (obviously)
    fn stab_multiplier(&self, has_adaptability: bool, _is_tera_stab: bool) -> u16 {
        if has_adaptability { 8192 } else { 6144 }
    }
    
    // No terrain in Gen 5
    fn terrain_modifier(&self, _terrain: super::Terrain, _move_type: crate::types::Type, _is_grounded: bool) -> Option<u16> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gen5_crit() {
        let gen = Gen5;
        // Gen 5 has 2.0x crit
        assert_eq!(gen.crit_multiplier(), 8192);
    }
    
    #[test]
    fn test_gen5_features() {
        let gen = Gen5;
        
        assert!(gen.has_abilities());
        assert!(gen.has_held_items());
        assert!(gen.uses_physical_special_split());
        assert!(!gen.has_terastallization());
        assert!(!gen.has_mega_evolution());
        assert!(!gen.has_z_moves());
        assert!(!gen.has_dynamax());
    }
}
