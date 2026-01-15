//! Battle state representation using Struct-of-Arrays (SoA) layout.
//!
//! This module defines the core `BattleState` which holds all battle data
//! in a cache-friendly, stack-allocated format optimized for AI rollouts.

use crate::abilities::AbilityId;
use crate::items::ItemId;
use crate::moves::MoveId;
use crate::natures::NatureId;
use crate::species::SpeciesId;
use crate::types::Type;

/// Maximum team size per player
pub const MAX_TEAM_SIZE: usize = 6;

/// Total entity slots (2 players × 6 Pokémon each)
pub const MAX_ENTITIES: usize = MAX_TEAM_SIZE * 2;

/// Number of move slots per Pokémon
pub const MAX_MOVES: usize = 4;

/// Number of stats affected by boosts (Atk, Def, SpA, SpD, Spe, Acc, Eva)
pub const BOOST_STATS: usize = 7;

// ============================================================================
// Status & Volatile Flags
// ============================================================================

bitflags::bitflags! {
    /// Major status conditions (only one can be active at a time)
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Status: u8 {
        const NONE      = 0;
        const BURN      = 1 << 0;
        const FREEZE    = 1 << 1;
        const PARALYSIS = 1 << 2;
        const POISON    = 1 << 3;
        const TOXIC     = 1 << 4; // Badly poisoned
        const SLEEP     = 1 << 5;
        // FIXME: Add frostbite if supporting Gen 9 Legends Arceus mechanics
    }
}

bitflags::bitflags! {
    /// Volatile status conditions (multiple can be active)
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Volatiles: u32 {
        const CONFUSION     = 1 << 0;
        const FLINCH        = 1 << 1;
        const SUBSTITUTE    = 1 << 2;
        const LEECH_SEED    = 1 << 3;
        const TAUNT         = 1 << 4;
        const ENCORE        = 1 << 5;
        const DISABLE       = 1 << 6;
        const TORMENT       = 1 << 7;
        const PROTECT       = 1 << 8;
        const ENDURE        = 1 << 9;
        const DESTINY_BOND  = 1 << 10;
        const PERISH_SONG   = 1 << 11;
        const INGRAIN       = 1 << 12;
        const AQUA_RING     = 1 << 13;
        const MAGNET_RISE   = 1 << 14;
        const TELEKINESIS   = 1 << 15;
        const HEAL_BLOCK    = 1 << 16;
        const EMBARGO       = 1 << 17;
        const ATTRACT       = 1 << 18;
        const FOCUS_ENERGY  = 1 << 19;
        const TRAPPED       = 1 << 20;  // Mean Look, Block, etc.
        const NIGHTMARE     = 1 << 21;
        const CURSE         = 1 << 22;  // Ghost-type curse
        const YAWN          = 1 << 23;
        // FIXME: Add more volatile states as needed (Stockpile, Charge, etc.)
    }
}

bitflags::bitflags! {
    /// Side conditions (team-wide effects like hazards and screens)
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct SideConditions: u32 {
        // Entry hazards
        const STEALTH_ROCK  = 1 << 0;
        const SPIKES_1      = 1 << 1;
        const SPIKES_2      = 1 << 2;
        const SPIKES_3      = 1 << 3;
        const TOXIC_SPIKES_1 = 1 << 4;
        const TOXIC_SPIKES_2 = 1 << 5;
        const STICKY_WEB    = 1 << 6;
        
        // Screens
        const REFLECT       = 1 << 7;
        const LIGHT_SCREEN  = 1 << 8;
        const AURORA_VEIL   = 1 << 9;
        
        // Other
        const TAILWIND      = 1 << 10;
        const SAFEGUARD     = 1 << 11;
        const MIST          = 1 << 12;
        const LUCKY_CHANT   = 1 << 13;
        // FIXME: Add more side conditions as needed
    }
}

// ============================================================================
// Battle State
// ============================================================================

/// Core battle state in Struct-of-Arrays layout.
/// 
/// Entity indices:
/// - 0-5: Player 1's team (index 0 = slot 1, etc.)
/// - 6-11: Player 2's team (index 6 = slot 1, etc.)
/// 
/// This struct is `Copy` to allow cheap cloning for AI search trees.
#[derive(Clone, Copy, Debug)]
pub struct BattleState {
    // ------------------------------------------------------------------------
    // Active Pokémon indices
    // ------------------------------------------------------------------------
    /// Currently active Pokémon index for each player [player1, player2]
    /// For singles: one active per side. For doubles, this would be [2] per side.
    pub active: [u8; 2],
    
    /// Number of Pokémon on each team (for variable team sizes)
    pub team_sizes: [u8; 2],
    
    // ------------------------------------------------------------------------
    // Per-entity components (SoA layout)
    // ------------------------------------------------------------------------
    /// Species ID for each entity
    pub species: [SpeciesId; MAX_ENTITIES],
    
    /// Current HP
    pub hp: [u16; MAX_ENTITIES],
    
    /// Maximum HP
    pub max_hp: [u16; MAX_ENTITIES],
    
    /// Current stats [HP, Atk, Def, SpA, SpD, Spe] (calculated at spawn)
    pub stats: [[u16; 6]; MAX_ENTITIES],
    
    /// Stat boosts [Atk, Def, SpA, SpD, Spe, Acc, Eva] (-6 to +6)
    pub boosts: [[i8; BOOST_STATS]; MAX_ENTITIES],
    
    /// Primary type (can change via moves like Soak or abilities like Protean)
    pub types: [[Type; 2]; MAX_ENTITIES],
    
    /// Ability
    pub abilities: [AbilityId; MAX_ENTITIES],
    
    /// Held item (None represented as ItemId::default())
    pub items: [ItemId; MAX_ENTITIES],
    
    /// Move IDs (4 moves per entity)
    pub moves: [[MoveId; MAX_MOVES]; MAX_ENTITIES],
    
    /// Current PP for each move
    pub pp: [[u8; MAX_MOVES]; MAX_ENTITIES],
    
    /// Max PP for each move
    pub max_pp: [[u8; MAX_MOVES]; MAX_ENTITIES],
    
    /// Major status condition
    pub status: [Status; MAX_ENTITIES],
    
    /// Volatile status flags
    pub volatiles: [Volatiles; MAX_ENTITIES],
    
    /// Sleep/Toxic counters (repurposed per status)
    pub status_counter: [u8; MAX_ENTITIES],
    
    /// Level (needed for damage calc)
    pub level: [u8; MAX_ENTITIES],
    
    /// Nature (stored for potential recalculation)
    pub nature: [NatureId; MAX_ENTITIES],
    
    // ------------------------------------------------------------------------
    // Side-wide state
    // ------------------------------------------------------------------------
    /// Side conditions for each player
    pub side_conditions: [SideConditions; 2],
    
    /// Screen/condition turn counters [Reflect, LightScreen, AuroraVeil, Tailwind, Safeguard, Mist]
    /// FIXME: Consider a more flexible counter system
    pub side_counters: [[u8; 6]; 2],
    
    // ------------------------------------------------------------------------
    // Battle-wide state
    // ------------------------------------------------------------------------
    /// Current turn number
    pub turn: u16,
    
    /// Weather (0 = none, then encoded weather types)
    /// FIXME: Define Weather enum
    pub weather: u8,
    
    /// Weather turns remaining (0 = permanent)
    pub weather_turns: u8,
    
    /// Terrain (0 = none, then encoded terrain types)
    /// FIXME: Define Terrain enum  
    pub terrain: u8,
    
    /// Terrain turns remaining
    pub terrain_turns: u8,
    
    /// Trick Room active
    pub trick_room: bool,
    
    /// Trick Room turns remaining
    pub trick_room_turns: u8,
    
    /// Gravity active
    pub gravity: bool,
    
    /// Gravity turns remaining
    pub gravity_turns: u8,
    
    // FIXME: Add more global state (Magic Room, Wonder Room, etc.)
}

impl Default for BattleState {
    fn default() -> Self {
        Self::new()
    }
}

impl BattleState {
    /// Create an empty battle state
    pub fn new() -> Self {
        Self {
            active: [0, 6], // First Pokémon of each team
            team_sizes: [0, 0],
            
            species: [SpeciesId(0); MAX_ENTITIES],
            hp: [0; MAX_ENTITIES],
            max_hp: [0; MAX_ENTITIES],
            stats: [[0; 6]; MAX_ENTITIES],
            boosts: [[0; BOOST_STATS]; MAX_ENTITIES],
            types: [[Type::Normal, Type::Normal]; MAX_ENTITIES],
            abilities: [AbilityId::Noability; MAX_ENTITIES],
            // SAFETY: ItemId and MoveId are repr(u16), so 0 is a valid representation
            items: [unsafe { core::mem::transmute::<u16, ItemId>(0) }; MAX_ENTITIES],
            moves: [[unsafe { core::mem::transmute::<u16, MoveId>(0) }; MAX_MOVES]; MAX_ENTITIES],
            pp: [[0; MAX_MOVES]; MAX_ENTITIES],
            max_pp: [[0; MAX_MOVES]; MAX_ENTITIES],
            status: [Status::NONE; MAX_ENTITIES],
            volatiles: [Volatiles::empty(); MAX_ENTITIES],
            status_counter: [0; MAX_ENTITIES],
            level: [0; MAX_ENTITIES],
            // SAFETY: NatureId is repr(u8), so 0 is a valid representation (Hardy)
            nature: [unsafe { core::mem::transmute::<u8, NatureId>(0) }; MAX_ENTITIES],
            
            side_conditions: [SideConditions::empty(); 2],
            side_counters: [[0; 6]; 2],
            
            turn: 0,
            weather: 0,
            weather_turns: 0,
            terrain: 0,
            terrain_turns: 0,
            trick_room: false,
            trick_room_turns: 0,
            gravity: false,
            gravity_turns: 0,
        }
    }
    
    /// Get the entity index for a player's team slot
    /// Player 0: indices 0-5, Player 1: indices 6-11
    #[inline]
    pub const fn entity_index(player: usize, slot: usize) -> usize {
        debug_assert!(player < 2);
        debug_assert!(slot < MAX_TEAM_SIZE);
        player * MAX_TEAM_SIZE + slot
    }
    
    /// Get the active entity index for a player
    #[inline]
    pub const fn active_index(&self, player: usize) -> usize {
        self.active[player] as usize
    }
    
    /// Check if an entity is fainted
    #[inline]
    pub const fn is_fainted(&self, index: usize) -> bool {
        self.hp[index] == 0
    }
    
    /// Get effective speed (base speed × boost multiplier)
    /// FIXME: This doesn't account for paralysis, Tailwind, weather abilities, etc.
    #[inline]
    pub fn effective_speed(&self, index: usize) -> u16 {
        let base = self.stats[index][5]; // Speed is stat index 5
        let boost = self.boosts[index][4]; // Speed is boost index 4
        apply_stat_boost(base, boost)
    }
    
    /// Get effective stat with boost applied
    #[inline]
    pub fn effective_stat(&self, index: usize, stat_index: usize) -> u16 {
        // stat_index: 0=HP (no boost), 1=Atk, 2=Def, 3=SpA, 4=SpD, 5=Spe
        if stat_index == 0 {
            return self.stats[index][0]; // HP has no boost
        }
        let base = self.stats[index][stat_index];
        let boost = self.boosts[index][stat_index - 1]; // Boost indices are shifted
        apply_stat_boost(base, boost)
    }
}

/// Apply stat stage boost to a base stat
/// Stages range from -6 to +6
/// Multipliers: -6 = 2/8, -5 = 2/7, ..., 0 = 2/2, ..., +6 = 8/2
#[inline]
pub fn apply_stat_boost(base: u16, stage: i8) -> u16 {
    let stage = stage.clamp(-6, 6) as i32;
    let (numerator, denominator) = if stage >= 0 {
        (2 + stage, 2)
    } else {
        (2, 2 - stage)
    };
    ((base as i32 * numerator) / denominator) as u16
}

// FIXME: Implement BattleQueue for action/event processing
// pub struct BattleQueue { ... }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_is_copy() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<BattleState>();
    }
    
    #[test]
    fn test_entity_index() {
        assert_eq!(BattleState::entity_index(0, 0), 0);
        assert_eq!(BattleState::entity_index(0, 5), 5);
        assert_eq!(BattleState::entity_index(1, 0), 6);
        assert_eq!(BattleState::entity_index(1, 5), 11);
    }
    
    #[test]
    fn test_stat_boost() {
        assert_eq!(apply_stat_boost(100, 0), 100);  // No boost
        assert_eq!(apply_stat_boost(100, 1), 150);  // +1 = 3/2
        assert_eq!(apply_stat_boost(100, 2), 200);  // +2 = 4/2
        assert_eq!(apply_stat_boost(100, 6), 400);  // +6 = 8/2
        assert_eq!(apply_stat_boost(100, -1), 66);  // -1 = 2/3
        assert_eq!(apply_stat_boost(100, -6), 25);  // -6 = 2/8
    }
}
