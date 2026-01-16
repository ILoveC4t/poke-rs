//! Entity blueprints for spawning Pokémon into battle.
//!
//! The `PokemonConfig` struct serves as a builder pattern for configuring
//! a Pokémon before injecting it into the `BattleState`.

use crate::abilities::AbilityId;
use crate::items::ItemId;
use crate::moves::MoveId;
use crate::natures::{BattleStat, NatureId};
use crate::species::{Species, SpeciesId};
use crate::state::{BattleState, MAX_MOVES};
use crate::types::Type;

/// Default IVs (perfect)
pub const DEFAULT_IVS: [u8; 6] = [31, 31, 31, 31, 31, 31];

/// Default EVs (none)
pub const DEFAULT_EVS: [u8; 6] = [0, 0, 0, 0, 0, 0];

/// Default level
pub const DEFAULT_LEVEL: u8 = 50;

/// Blueprint for spawning a Pokémon into battle.
/// 
/// Use builder methods to customize, then call `spawn()` to inject
/// into a `BattleState` at a specific slot.
#[derive(Clone, Debug)]
pub struct PokemonConfig {
    /// Species (determines base stats and types)
    pub species: SpeciesId,
    
    /// Level (1-100)
    pub level: u8,
    
    /// Individual Values [HP, Atk, Def, SpA, SpD, Spe] (0-31)
    pub ivs: [u8; 6],
    
    /// Effort Values [HP, Atk, Def, SpA, SpD, Spe] (0-255, max 510 total)
    pub evs: [u8; 6],
    
    /// Nature (affects stat growth)
    pub nature: NatureId,
    
    /// Ability (if None, uses species' first ability)
    pub ability: Option<AbilityId>,
    
    /// Held item
    pub item: ItemId,
    
    /// Move set
    pub moves: [MoveId; MAX_MOVES],
    
    /// Happiness (affects Return/Frustration power)
    /// FIXME: May not be needed if these moves are deprecated in Gen 9
    pub happiness: u8,
    
    /// Override types (for custom forms, etc.)
    pub types_override: Option<[Type; 2]>,
    
    /// Current HP (if less than max, e.g., for restoring a saved team)
    pub current_hp: Option<u16>,
}

impl Default for PokemonConfig {
    fn default() -> Self {
        Self {
            species: SpeciesId(0),
            level: DEFAULT_LEVEL,
            ivs: DEFAULT_IVS,
            evs: DEFAULT_EVS,
            nature: NatureId::default(),
            ability: None,
            item: ItemId::default(),
            moves: [MoveId::default(); MAX_MOVES],
            happiness: 255,
            types_override: None,
            current_hp: None,
        }
    }
}

impl PokemonConfig {
    /// Create a new config for a species
    pub fn new(species: SpeciesId) -> Self {
        Self {
            species,
            ..Default::default()
        }
    }
    
    /// Create from species string key
    pub fn from_str(species_key: &str) -> Option<Self> {
        SpeciesId::from_str(species_key).map(Self::new)
    }
    
    // ========================================================================
    // Builder methods
    // ========================================================================
    
    /// Set level
    pub fn level(mut self, level: u8) -> Self {
        self.level = level.clamp(1, 100);
        self
    }
    
    /// Set IVs
    pub fn ivs(mut self, ivs: [u8; 6]) -> Self {
        self.ivs = ivs.map(|v| v.min(31));
        self
    }
    
    /// Set EVs
    pub fn evs(mut self, evs: [u8; 6]) -> Self {
        // Clamp individual EVs to 252 and total to 510
        let mut total: u16 = 0;
        for ev in &mut self.evs.iter_mut().zip(evs.iter()) {
            let clamped = (*ev.1).min(252);
            let remaining = 510u16.saturating_sub(total);
            // Cap remaining at 255 for u8 cast, though clamped is max 252 anyway
            *ev.0 = clamped.min(remaining.min(255) as u8);
            total += *ev.0 as u16;
        }
        self
    }
    
    /// Set nature
    pub fn nature(mut self, nature: NatureId) -> Self {
        self.nature = nature;
        self
    }
    
    /// Set ability
    pub fn ability(mut self, ability: AbilityId) -> Self {
        self.ability = Some(ability);
        self
    }
    
    /// Set held item
    pub fn item(mut self, item: ItemId) -> Self {
        self.item = item;
        self
    }
    
    /// Set moves
    pub fn moves(mut self, moves: [MoveId; MAX_MOVES]) -> Self {
        self.moves = moves;
        self
    }
    
    /// Set a single move at a slot
    pub fn set_move(mut self, slot: usize, move_id: MoveId) -> Self {
        if slot < MAX_MOVES {
            self.moves[slot] = move_id;
        }
        self
    }
    
    /// Set current HP (for partially damaged Pokémon)
    pub fn current_hp(mut self, hp: u16) -> Self {
        self.current_hp = Some(hp);
        self
    }
    
    // ========================================================================
    // Stat Calculation
    // ========================================================================
    
    /// Calculate final stats based on base stats, IVs, EVs, level, and nature
    pub fn calculate_stats(&self) -> [u16; 6] {
        let species = self.species.data();
        let base = species.base_stats;
        let level = self.level as u32;
        
        let mut stats = [0u16; 6];
        
        // HP formula: floor((2 * Base + IV + floor(EV/4)) * Level / 100) + Level + 10
        stats[0] = self.calculate_hp(base[0] as u32, level);
        
        // Other stats: floor((floor((2 * Base + IV + floor(EV/4)) * Level / 100) + 5) * Nature)
        for i in 1..6 {
            stats[i] = self.calculate_stat(i, base[i] as u32, level);
        }
        
        stats
    }
    
    /// Calculate HP stat (special formula)
    fn calculate_hp(&self, base: u32, level: u32) -> u16 {
        // Shedinja always has 1 HP
        if self.species.data().flags & crate::species::FLAG_FORCE_1_HP != 0 {
            return 1;
        }

        let iv = self.ivs[0] as u32;
        let ev = self.evs[0] as u32;
        
        let hp = ((2 * base + iv + ev / 4) * level / 100) + level + 10;
        hp as u16
    }
    
    /// Calculate non-HP stat with nature modifier
    fn calculate_stat(&self, stat_index: usize, base: u32, level: u32) -> u16 {
        let iv = self.ivs[stat_index] as u32;
        let ev = self.evs[stat_index] as u32;
        
        let raw = ((2 * base + iv + ev / 4) * level / 100) + 5;
        
        // Apply nature modifier (9 = -10%, 10 = neutral, 11 = +10%)
        let nature_stat = match stat_index {
            1 => BattleStat::Atk,
            2 => BattleStat::Def,
            3 => BattleStat::SpA,
            4 => BattleStat::SpD,
            5 => BattleStat::Spe,
            _ => unreachable!(),
        };
        let modifier = self.nature.stat_modifier(nature_stat) as u32;
        
        ((raw * modifier) / 10) as u16
    }
    
    /// Get effective types (from override or species)
    fn get_types(&self) -> [Type; 2] {
        if let Some(types) = self.types_override {
            return types;
        }
        
        let species = self.species.data();
        let type1 = species.primary_type();
        let type2 = species.secondary_type().unwrap_or(type1);
        [type1, type2]
    }
    
    /// Get ability (from override or species default)
    fn get_ability(&self, _species: &Species) -> AbilityId {
        if let Some(ability) = self.ability {
            return ability;
        }
        
        // FIXME: Look up species' first ability from Species data
        // For now, return a placeholder
        AbilityId::default()
    }
    
    // ========================================================================
    // Spawning
    // ========================================================================
    
    /// Spawn this Pokémon into the battle state at the given player/slot
    pub fn spawn(&self, state: &mut BattleState, player: usize, slot: usize) {
        let index = BattleState::entity_index(player, slot);
        let species = self.species.data();
        
        // Calculate and set stats
        let stats = self.calculate_stats();
        state.stats[index] = stats;
        
        // Set HP
        let max_hp = stats[0];
        state.max_hp[index] = max_hp;
        state.hp[index] = self.current_hp.unwrap_or(max_hp).min(max_hp);
        
        // Set identity
        state.species[index] = self.species;
        state.level[index] = self.level;
        state.nature[index] = self.nature;
        
        // Set types
        state.types[index] = self.get_types();
        
        // Set ability
        state.abilities[index] = self.get_ability(species);
        
        // Set item
        state.items[index] = self.item;
        
        // Set moves and PP
        state.moves[index] = self.moves;
        for i in 0..MAX_MOVES {
            // FIXME: Look up move's base PP and calculate max PP with PP Ups
            state.pp[index][i] = 0;  // Placeholder
            state.max_pp[index][i] = 0;
        }
        
        // Reset volatile state
        state.boosts[index] = [0; 7];
        state.status[index] = crate::state::Status::NONE;
        state.volatiles[index] = crate::state::Volatiles::empty();
        state.status_counter[index] = 0;
        
        // Update team size if needed
        if slot >= state.team_sizes[player] as usize {
            state.team_sizes[player] = (slot + 1) as u8;
        }
    }
}

// ============================================================================
// Convenience functions for common Pokémon setups
// ============================================================================

/// Create a basic config with common competitive defaults
pub fn competitive_config(species: SpeciesId) -> PokemonConfig {
    PokemonConfig::new(species)
        .level(50)
        .ivs(DEFAULT_IVS)
}

// FIXME: Add preset factory functions for specific Pokémon (like pokedex::gengar())
// These would be generated or hand-written based on common sets.

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stat_calculation() {
        // Test cases derived from smogon/damage-calc stats.test.ts
        // Level 100, Base 100 all stats, 31 IVs, 252 EVs, Adamant (+Atk, -SpA)
        // Expected: { hp: 404, atk: 328, def: 299, spa: 269, spd: 299, spe: 299 }
        
        if let Some(config) = PokemonConfig::from_str("pikachu") {
            // Pikachu base stats: 35/55/40/50/50/90
            let config = config
                .level(50)
                .ivs([31, 31, 31, 31, 31, 31])
                .evs([0, 0, 0, 0, 0, 252])
                .nature(NatureId::from_str("timid").unwrap()); // +Spe, -Atk
            
            let stats = config.calculate_stats();
            
            // HP formula: floor((2*35 + 31 + 0) * 50 / 100) + 50 + 10 = 110
            assert_eq!(stats[0], 110, "Pikachu HP mismatch");
            
            // Speed with Timid (+10%) and 252 EVs:
            // Raw = floor((2*90 + 31 + 63) * 50 / 100) + 5 = 142
            // With Timid = floor(142 * 1.1) = 156
            assert_eq!(stats[5], 156, "Pikachu Speed mismatch");
            
            // Attack with Timid (-10%):
            // Raw = floor((2*55 + 31 + 0) * 50 / 100) + 5 = 75
            // With Timid = floor(75 * 0.9) = 67
            assert_eq!(stats[1], 67, "Pikachu Attack mismatch");
        }
        
        // Test with base 100 stats at level 100 (from damage-calc test)
        // Using Mew: 100/100/100/100/100/100 base stats
        if let Some(config) = PokemonConfig::from_str("mew") {
            let config = config
                .level(100)
                .ivs([31, 31, 31, 31, 31, 31])
                .evs([252, 252, 0, 0, 0, 0])
                .nature(NatureId::from_str("adamant").unwrap()); // +Atk, -SpA
            
            let stats = config.calculate_stats();
            
            // HP: floor((2*100 + 31 + 63) * 100 / 100) + 100 + 10 = 404
            assert_eq!(stats[0], 404, "Mew HP mismatch");
            
            // Atk with Adamant (+10%) and 252 EVs:
            // Raw = floor((2*100 + 31 + 63) * 100 / 100) + 5 = 299
            // With Adamant = floor(299 * 1.1) = 328
            assert_eq!(stats[1], 328, "Mew Attack mismatch");
            
            // SpA with Adamant (-10%) and 0 EVs:
            // Raw = floor((2*100 + 31 + 0) * 100 / 100) + 5 = 236
            // With Adamant = floor(236 * 0.9) = 212
            assert_eq!(stats[3], 212, "Mew SpA mismatch");
        }
    }
    
    #[test]
    fn test_spawn() {
        let mut state = BattleState::new();
        
        if let Some(config) = PokemonConfig::from_str("pikachu") {
            config.level(50).spawn(&mut state, 0, 0);
            
            assert!(!state.is_fainted(0));
            assert!(state.hp[0] > 0);
            assert_eq!(state.level[0], 50);
            assert_eq!(state.team_sizes[0], 1);
        }
    }

    #[test]
    fn test_shedinja_hp() {
        if let Some(config) = PokemonConfig::from_str("shedinja") {
            let stats = config.calculate_stats();
            assert_eq!(stats[0], 1, "Shedinja must have 1 HP");

            // Even at level 100 with max IVs/EVs
            let config_max = config.level(100).ivs([31; 6]).evs([252; 6]);
            let stats_max = config_max.calculate_stats();
            assert_eq!(stats_max[0], 1, "Shedinja must have 1 HP even at max level/investment");
        } else {
             panic!("Shedinja not found in pokedex data");
        }
    }
}
