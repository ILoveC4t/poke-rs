//! Battle state representation using Struct-of-Arrays (SoA) layout.
//!
//! This module defines the core `BattleState` which holds all battle data
//! in a cache-friendly, stack-allocated format optimized for AI rollouts.

use crate::abilities::AbilityId;
use crate::entities::Gender;
use crate::items::ItemId;
use crate::moves::{MoveId, MoveCategory};
use crate::natures::NatureId;
use crate::species::{SpeciesId, Species};
use crate::terrains::TerrainId;
use crate::types::{Type, type_effectiveness};

/// Maximum team size per player
pub const MAX_TEAM_SIZE: usize = 6;

/// Total entity slots (2 players × 6 Pokémon each)
pub const MAX_ENTITIES: usize = MAX_TEAM_SIZE * 2;

/// Number of move slots per Pokémon
pub const MAX_MOVES: usize = 4;

/// Number of stats affected by boosts (Atk, Def, SpA, SpD, Spe, Acc, Eva)
pub const BOOST_STATS: usize = 7;

// Weather constants (private, matching damage/generations/mod.rs)
const WEATHER_SUN: u8 = 1;
const WEATHER_RAIN: u8 = 2;
const WEATHER_SAND: u8 = 3;
const WEATHER_HAIL: u8 = 4;
const WEATHER_SNOW: u8 = 5;
const WEATHER_HARSH_SUN: u8 = 6;
const WEATHER_HEAVY_RAIN: u8 = 7;

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
        const SMACK_DOWN    = 1 << 24;
        // FIXME: Add more volatile states as needed (Stockpile, Charge, etc.)
    }
}

/// Per-side battle conditions (screens, hazards, etc.)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SideConditions {
    // Screens (turns remaining, 0 = inactive)
    pub reflect_turns: u8,
    pub light_screen_turns: u8,
    pub aurora_veil_turns: u8,

    // Hazards (layer count, 0 = none)
    pub stealth_rock: bool,
    pub spikes_layers: u8,      // 0-3
    pub toxic_spikes_layers: u8, // 0-2
    pub sticky_web: bool,

    // Other side conditions
    pub tailwind_turns: u8,
    pub mist_turns: u8,
    pub safeguard_turns: u8,
    pub lucky_chant_turns: u8,
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

    /// Individual Values [HP, Atk, Def, SpA, SpD, Spe] (stored for recalculation)
    pub ivs: [[u8; 6]; MAX_ENTITIES],

    /// Effort Values [HP, Atk, Def, SpA, SpD, Spe] (stored for recalculation)
    pub evs: [[u8; 6]; MAX_ENTITIES],

    /// Weight in hectograms (0.1 kg)
    pub weight: [u16; MAX_ENTITIES],

    /// Gender
    pub gender: [Gender; MAX_ENTITIES],

    /// Transformed/Mega Evolved flag
    pub transformed: [bool; MAX_ENTITIES],
    
    // ------------------------------------------------------------------------
    // Side-wide state
    // ------------------------------------------------------------------------
    /// Side conditions for each player
    pub side_conditions: [SideConditions; 2],
    
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
            items: [ItemId::default(); MAX_ENTITIES],
            moves: [[MoveId::default(); MAX_MOVES]; MAX_ENTITIES],
            pp: [[0; MAX_MOVES]; MAX_ENTITIES],
            max_pp: [[0; MAX_MOVES]; MAX_ENTITIES],
            status: [Status::NONE; MAX_ENTITIES],
            volatiles: [Volatiles::empty(); MAX_ENTITIES],
            status_counter: [0; MAX_ENTITIES],
            level: [0; MAX_ENTITIES],
            nature: [NatureId::default(); MAX_ENTITIES],
            ivs: [[0; 6]; MAX_ENTITIES],
            evs: [[0; 6]; MAX_ENTITIES],
            weight: [0; MAX_ENTITIES],
            gender: [Gender::Genderless; MAX_ENTITIES],
            transformed: [false; MAX_ENTITIES],
            
            side_conditions: [SideConditions::default(); 2],
            
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
    
    /// Get the side (player index) of an entity
    #[inline]
    pub const fn get_side(&self, index: usize) -> usize {
        if index < MAX_TEAM_SIZE { 0 } else { 1 }
    }

    /// Check if doubles format
    #[inline]
    pub const fn is_doubles(&self) -> bool {
        // FIXME: Add format field to BattleState or infer?
        // For now assume singles if active is [0, 6], doubles if [0, 1, 6, 7] etc.
        // Or assume singles for simplicity until format is added.
        false
    }

    /// Get effective speed accounting for all modifiers.
    #[inline]
    pub fn effective_speed(&self, index: usize) -> u16 {
        let base = self.stats[index][5]; // Speed is stat index 5
        let boost = self.boosts[index][4]; // Speed is boost index 4
        let mut speed = apply_stat_boost(base, boost) as u32;
        
        // Paralysis: 0.5x (Gen 7+)
        if self.status[index].contains(Status::PARALYSIS) {
            speed /= 2;
        }
        
        // Weather and Terrain Abilities (2x)
        match self.abilities[index] {
            AbilityId::Chlorophyll => {
                if self.weather == WEATHER_SUN || self.weather == WEATHER_HARSH_SUN {
                    speed *= 2;
                }
            }
            AbilityId::Swiftswim => {
                if self.weather == WEATHER_RAIN || self.weather == WEATHER_HEAVY_RAIN {
                    speed *= 2;
                }
            }
            AbilityId::Sandrush => {
                if self.weather == WEATHER_SAND {
                    speed *= 2;
                }
            }
            AbilityId::Slushrush => {
                if self.weather == WEATHER_SNOW || self.weather == WEATHER_HAIL {
                    speed *= 2;
                }
            }
            AbilityId::Surgesurfer => {
                if self.terrain == TerrainId::Electric as u8 {
                    speed *= 2;
                }
            }
            _ => {}
        }
        
        // Tailwind: 2x speed for the side
        let side = self.get_side(index);
        if self.side_conditions[side].tailwind_turns > 0 {
            speed *= 2;
        }
        
        // Item modifiers
        let item = self.items[index];
        if item == ItemId::Choicescarf {
            // Choice Scarf: 1.5x
            speed = speed * 3 / 2;
        } else if item == ItemId::Ironball {
            // Iron Ball: 0.5x
            speed /= 2;
        }
        
        speed.min(u16::MAX as u32) as u16
    }
    
    /// Get effective stat with boost applied
    #[inline]
    pub fn effective_stat(&self, index: usize, stat_index: usize) -> u16 {
        if stat_index == 0 {
            return self.stats[index][0]; // HP has no boost
        }
        let base = self.stats[index][stat_index];
        let boost = self.boosts[index][stat_index - 1]; // Boost indices are shifted
        apply_stat_boost(base, boost)
    }

    /// Check if an entity is grounded.
    pub fn is_grounded(&self, index: usize) -> bool {
        // 1. Forced Grounding
        if self.gravity {
            return true;
        }

        let volatiles = self.volatiles[index];
        if volatiles.contains(Volatiles::INGRAIN) || volatiles.contains(Volatiles::SMACK_DOWN) {
            return true;
        }

        let item = self.items[index];
        if item == ItemId::Ironball {
            return true;
        }

        // 2. Ungrounded Checks
        if volatiles.contains(Volatiles::MAGNET_RISE) || volatiles.contains(Volatiles::TELEKINESIS) {
            return false;
        }

        if item == ItemId::Airballoon {
            return false;
        }

        let ability = self.abilities[index];
        if ability == AbilityId::Levitate {
            return false;
        }

        let types = self.types[index];
        if types[0] == Type::Flying || types[1] == Type::Flying {
            return false;
        }

        // 3. Default
        true
    }

    // ========================================================================
    // Task D Implementations
    // ========================================================================

    /// Change a Pokémon's forme mid-battle (Mega Evolution, Primal Reversion, etc.)
    /// Updates base stats and ability. Does NOT update moves.
    pub fn apply_forme_change(&mut self, entity_idx: usize, new_forme: SpeciesId) {
        let forme_data = new_forme.data();

        // Update species reference
        self.species[entity_idx] = new_forme;

        // Update weight
        self.weight[entity_idx] = forme_data.weight;

        // Update types
        self.types[entity_idx][0] = forme_data.primary_type();
        self.types[entity_idx][1] = forme_data.secondary_type().unwrap_or(forme_data.primary_type());

        // Recalculate stats with new base stats (HP stays, others recalculated)
        self.recalculate_stats(entity_idx, forme_data);

        // Update ability if forme has a specific ability (Mega/Primal usually forces ability)
        // For standard form changes, we might need more logic, but for Mega/Primal:
        // Pokedex data stores abilities.
        // For Mega, ability0 is the Mega Ability.
        // If it's a permanent form change, we take primary ability.
        self.abilities[entity_idx] = forme_data.primary_ability();

        // Mark as transformed
        self.transformed[entity_idx] = true;
    }

    /// Recalculate stats based on new species data (helper for forme change)
    fn recalculate_stats(&mut self, entity_idx: usize, species: &Species) {
        use crate::natures::BattleStat;

        let level = self.level[entity_idx] as u32;
        let base = species.base_stats;
        let ivs = self.ivs[entity_idx];
        let evs = self.evs[entity_idx];
        let nature = self.nature[entity_idx];

        // HP is NOT recalculated for form changes (unless specifically needed, but usually current HP stays same fraction? Or absolute value?)
        // In-game mechanics: Current HP stays same absolute value, Max HP changes.
        // If Max HP changes, we need to update max_hp.
        // If form change happens mid-battle, Max HP updates. Current HP is capped or maintained.
        // BUT, Shedinja (1HP) logic is weird.
        // Usually, HP = ((2 * Base + IV + EV/4) * Level / 100) + Level + 10

        let new_max_hp = if species.flags & crate::species::FLAG_FORCE_1_HP != 0 {
            1
        } else {
            let iv = ivs[0] as u32;
            let ev = evs[0] as u32;
            ((2 * (base[0] as u32) + iv + ev / 4) * level / 100) + level + 10
        } as u16;

        // Update HP scaling?
        // If max HP increases, current HP stays.
        // If max HP decreases below current HP, current HP is capped? Or stays?
        // In Dynamax, HP doubles.
        // In Mega Evolution, HP usually stays same (base stat doesn't change).
        // But Zygarde Complete changes HP.
        // Standard behavior: Current HP remains the same value, unless > new max (cap it).
        // BUT, some mechanics maintain percentage.
        // Mega Evolution specifically: Base HP never changes.
        // Primal Reversion: Base HP never changes.
        // Form changes like Zygarde: Adds current HP equal to increase in Max HP?

        // For simplicity and standard Mega/Primal rules (where HP doesn't change), we recalculate Max HP just in case.
        let _old_max_hp = self.max_hp[entity_idx];
        self.max_hp[entity_idx] = new_max_hp;

        // If HP changed (e.g. Zygarde), adjust current HP?
        // For now, cap it.
        self.hp[entity_idx] = self.hp[entity_idx].min(new_max_hp);
        // Note: If Zygarde logic is needed (Power Construct), it adds the difference.
        // But that's an ability hook logic (Task E).

        // Other stats
        for i in 1..6 {
            let iv = ivs[i] as u32;
            let ev = evs[i] as u32;
            let raw = ((2 * (base[i] as u32) + iv + ev / 4) * level / 100) + 5;

            let nature_stat = match i {
                1 => BattleStat::Atk,
                2 => BattleStat::Def,
                3 => BattleStat::SpA,
                4 => BattleStat::SpD,
                5 => BattleStat::Spe,
                _ => unreachable!(),
            };
            let modifier = nature.stat_modifier(nature_stat) as u32;

            self.stats[entity_idx][i] = ((raw * modifier) / 10) as u16;
        }
    }

    /// Apply damage to an entity
    pub fn apply_damage(&mut self, entity_idx: usize, damage: u16) {
        self.hp[entity_idx] = self.hp[entity_idx].saturating_sub(damage);
    }

    /// Apply stat change (boost)
    pub fn apply_stat_change(&mut self, entity_idx: usize, stat: usize, delta: i8) {
        // stat: 0=HP(invalid), 1=Atk, ..., 5=Spe, 6=Acc, 7=Eva
        // boosts array is 0-6 corresponding to Atk-Eva
        if stat == 0 || stat > BOOST_STATS { return; }

        let boost_idx = stat - 1;
        let current = self.boosts[entity_idx][boost_idx];
        self.boosts[entity_idx][boost_idx] = (current + delta).clamp(-6, 6);
    }

    /// Get the screen damage modifier for an incoming attack
    /// Returns multiplier in 4096ths (e.g., 2048 = 0.5×)
    pub fn get_screen_modifier(&self, defender_idx: usize, category: MoveCategory) -> u16 {
        let side = self.get_side(defender_idx);
        let conditions = &self.side_conditions[side];

        // Aurora Veil covers both physical and special
        if conditions.aurora_veil_turns > 0 {
            return if self.is_doubles() { 2732 } else { 2048 };  // 2/3 or 1/2
        }

        match category {
            MoveCategory::Physical if conditions.reflect_turns > 0 => {
                if self.is_doubles() { 2732 } else { 2048 }
            }
            MoveCategory::Special if conditions.light_screen_turns > 0 => {
                if self.is_doubles() { 2732 } else { 2048 }
            }
            _ => 4096,  // No reduction
        }
    }

    /// Apply entry hazard damage when a Pokémon switches in
    /// Returns damage dealt (0 if immune or no hazards)
    pub fn apply_entry_hazards(&mut self, entity_idx: usize) -> u16 {
        // If Heavy-Duty Boots, ignore hazards
        if self.items[entity_idx] == ItemId::Heavydutyboots {
            return 0;
        }
        // Magic Guard also ignores hazard damage (Task E/A logic?), but we should handle it if possible.
        // Checking ability here:
        if self.abilities[entity_idx] == AbilityId::Magicguard {
            return 0;
        }

        let side = self.get_side(entity_idx);
        let conditions = self.side_conditions[side]; // Copy since it's Copy
        let pokemon_types = self.types[entity_idx];
        let mut total_damage = 0u16;

        // Stealth Rock: Type effectiveness based damage (1/8 neutral)
        if conditions.stealth_rock {
            let eff = type_effectiveness(Type::Rock, pokemon_types[0], if pokemon_types[1] != pokemon_types[0] { Some(pokemon_types[1]) } else { None });
            // eff: 0=0x, 1=0.25x, 2=0.5x, 4=1x, 8=2x, 16=4x
            // Base is 1/8 of max HP.
            // 1x -> 1/8 = 0.125
            // 2x -> 1/4 = 0.25
            // 4x -> 1/2 = 0.5
            // 0.5x -> 1/16 = 0.0625
            // 0.25x -> 1/32 = 0.03125

            let factor = match eff {
                16 => 2, // 1/2
                8 => 4,  // 1/4
                4 => 8,  // 1/8
                2 => 16, // 1/16
                1 => 32, // 1/32
                _ => 0,  // Immune
            };

            if factor > 0 {
                total_damage += self.max_hp[entity_idx] / factor;
            }
        }

        // Spikes: Tier-based damage (grounded Pokémon only)
        if self.is_grounded(entity_idx) {
            let layers = conditions.spikes_layers;
            if layers > 0 {
                let factor = match layers {
                    1 => 8, // 1/8
                    2 => 6, // 1/6
                    _ => 4, // 1/4
                };
                total_damage += self.max_hp[entity_idx] / factor;
            }

            // Toxic Spikes
            let tspikes = conditions.toxic_spikes_layers;
            let is_poison = pokemon_types[0] == Type::Poison || pokemon_types[1] == Type::Poison;
            let is_steel = pokemon_types[0] == Type::Steel || pokemon_types[1] == Type::Steel;

            if tspikes > 0 {
                if is_poison && self.is_grounded(entity_idx) {
                    // Absorb Toxic Spikes
                    self.side_conditions[side].toxic_spikes_layers = 0;
                } else if !is_poison && !is_steel && self.is_grounded(entity_idx) {
                    // Apply poison
                    if self.status[entity_idx] == Status::NONE {
                        if tspikes >= 2 {
                            self.status[entity_idx] = Status::TOXIC;
                        } else {
                            self.status[entity_idx] = Status::POISON;
                        }
                    }
                }
            }

            // Sticky Web: -1 Speed to grounded Pokémon
            if conditions.sticky_web && self.is_grounded(entity_idx) {
                self.apply_stat_change(entity_idx, 5, -1);
            }
        }

        self.apply_damage(entity_idx, total_damage);
        total_damage
    }

    /// Decrement all turn-based side conditions. Call at end of turn.
    pub fn tick_side_conditions(&mut self) {
        for side in &mut self.side_conditions {
            side.reflect_turns = side.reflect_turns.saturating_sub(1);
            side.light_screen_turns = side.light_screen_turns.saturating_sub(1);
            side.aurora_veil_turns = side.aurora_veil_turns.saturating_sub(1);
            side.tailwind_turns = side.tailwind_turns.saturating_sub(1);
            side.mist_turns = side.mist_turns.saturating_sub(1);
            side.safeguard_turns = side.safeguard_turns.saturating_sub(1);
            side.lucky_chant_turns = side.lucky_chant_turns.saturating_sub(1);
        }
    }

    /// Apply weight modification (e.g., Autotomize reduces by 100kg)
    pub fn modify_weight(&mut self, entity_idx: usize, delta_hectograms: i16) {
        let current = self.weight[entity_idx] as i16;
        let new_weight = current + delta_hectograms;
        self.weight[entity_idx] = new_weight.max(1) as u16;  // Min 0.1kg
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

/// Priority bracket for turn order determination
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityBracket {
    /// Pursuit on switching target
    Pursuit = 0,
    /// Highest priority (+5: Helping Hand)
    Priority5 = 1,
    /// Priority +4 (Protect, Detect)
    Priority4 = 2,
    /// Priority +3 (Fake Out, Quick Guard)
    Priority3 = 3,
    /// Priority +2 (Extreme Speed, Follow Me)
    Priority2 = 4,
    /// Priority +1 (Aqua Jet, Mach Punch, Sucker Punch)
    Priority1 = 5,
    /// Normal priority (0)
    Normal = 6,
    /// Priority -1 (Vital Throw)
    PriorityMinus1 = 7,
    /// Priority -2 (Focus Punch)
    PriorityMinus2 = 8,
    /// Priority -3 (Avalanche, Revenge)
    PriorityMinus3 = 9,
    /// Priority -4 (Counter, Mirror Coat)
    PriorityMinus4 = 10,
    /// Priority -5 (Roar, Whirlwind)
    PriorityMinus5 = 11,
    /// Priority -6 (Trick Room, Circle Throw)
    PriorityMinus6 = 12,
    /// Priority -7 (Teleport)
    PriorityMinus7 = 13,
}

impl PriorityBracket {
    /// Convert a move's priority value to a bracket
    pub fn from_priority(priority: i8) -> Self {
        match priority {
            5.. => PriorityBracket::Priority5,
            4 => PriorityBracket::Priority4,
            3 => PriorityBracket::Priority3,
            2 => PriorityBracket::Priority2,
            1 => PriorityBracket::Priority1,
            0 => PriorityBracket::Normal,
            -1 => PriorityBracket::PriorityMinus1,
            -2 => PriorityBracket::PriorityMinus2,
            -3 => PriorityBracket::PriorityMinus3,
            -4 => PriorityBracket::PriorityMinus4,
            -5 => PriorityBracket::PriorityMinus5,
            -6 => PriorityBracket::PriorityMinus6,
            _ => PriorityBracket::PriorityMinus7,
        }
    }
}

/// Result of comparing two actions for turn order
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnOrder {
    /// First entity moves first
    First,
    /// Second entity moves first
    Second,
    /// Speed tie (random determination needed)
    Tie,
}

impl BattleState {
    /// Compare two entities to determine turn order.
    ///
    /// Takes into account:
    /// - Priority brackets
    /// - Trick Room (reverses speed comparison)
    /// - Effective speed
    ///
    /// Returns which entity should move first, or Tie if speeds are equal.
    pub fn compare_turn_order(
        &self,
        entity1: usize,
        priority1: i8,
        entity2: usize,
        priority2: i8,
    ) -> TurnOrder {
        let bracket1 = PriorityBracket::from_priority(priority1);
        let bracket2 = PriorityBracket::from_priority(priority2);
        
        // Higher priority (lower enum value) goes first
        if bracket1 < bracket2 {
            return TurnOrder::First;
        }
        if bracket2 < bracket1 {
            return TurnOrder::Second;
        }
        
        // Same priority bracket: compare speeds
        let speed1 = self.effective_speed(entity1);
        let speed2 = self.effective_speed(entity2);
        
        // Trick Room: slower goes first
        if self.trick_room {
            if speed1 < speed2 {
                return TurnOrder::First;
            }
            if speed2 < speed1 {
                return TurnOrder::Second;
            }
        } else {
            // Normal: faster goes first
            if speed1 > speed2 {
                return TurnOrder::First;
            }
            if speed2 > speed1 {
                return TurnOrder::Second;
            }
        }
        
        // Speed tie
        TurnOrder::Tie
    }
}

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
        assert_eq!(apply_stat_boost(100, 0), 100);
        assert_eq!(apply_stat_boost(100, 1), 150);
        assert_eq!(apply_stat_boost(100, 2), 200);
        assert_eq!(apply_stat_boost(100, 6), 400);
        assert_eq!(apply_stat_boost(100, -1), 66);
        assert_eq!(apply_stat_boost(100, -6), 25);
    }

    #[test]
    fn test_effective_speed_modifiers() {
        let mut state = BattleState::new();
        let idx = 0;
        state.stats[idx][5] = 100;

        assert_eq!(state.effective_speed(idx), 100);

        state.boosts[idx][4] = 1;
        assert_eq!(state.effective_speed(idx), 150);
        state.boosts[idx][4] = 0;

        state.status[idx] = Status::PARALYSIS;
        assert_eq!(state.effective_speed(idx), 50);
        state.status[idx] = Status::NONE;

        state.side_conditions[0].tailwind_turns = 3;
        assert_eq!(state.effective_speed(idx), 200);

        state.status[idx] = Status::PARALYSIS;
        assert_eq!(state.effective_speed(idx), 100);
        state.status[idx] = Status::NONE;
        state.side_conditions[0] = SideConditions::default();

        state.abilities[idx] = AbilityId::Swiftswim;
        state.weather = WEATHER_RAIN;
        assert_eq!(state.effective_speed(idx), 200);
        state.weather = WEATHER_SUN;
        assert_eq!(state.effective_speed(idx), 100);

        state.abilities[idx] = AbilityId::Chlorophyll;
        state.weather = WEATHER_SUN;
        assert_eq!(state.effective_speed(idx), 200);

        state.abilities[idx] = AbilityId::Sandrush;
        state.weather = WEATHER_SAND;
        assert_eq!(state.effective_speed(idx), 200);

        state.abilities[idx] = AbilityId::Slushrush;
        state.weather = WEATHER_SNOW;
        assert_eq!(state.effective_speed(idx), 200);
        state.weather = WEATHER_HAIL;
        assert_eq!(state.effective_speed(idx), 200);

        state.abilities[idx] = AbilityId::Surgesurfer;
        state.weather = 0;
        state.terrain = TerrainId::Electric as u8;
        assert_eq!(state.effective_speed(idx), 200);
        state.terrain = TerrainId::Grassy as u8;
        assert_eq!(state.effective_speed(idx), 100);

        state.terrain = TerrainId::Electric as u8;
        state.abilities[idx] = AbilityId::Surgesurfer;
        state.side_conditions[0].tailwind_turns = 3;
        state.status[idx] = Status::PARALYSIS;
        assert_eq!(state.effective_speed(idx), 200);
    }

    #[test]
    fn test_grounded_logic() {
        let mut state = BattleState::new();
        let idx = 0;

        state.types[idx][0] = Type::Normal;
        assert!(state.is_grounded(idx));

        state.types[idx][0] = Type::Flying;
        assert!(!state.is_grounded(idx));

        state.types[idx][0] = Type::Normal;
        state.abilities[idx] = AbilityId::Levitate;
        assert!(!state.is_grounded(idx));

        state.abilities[idx] = AbilityId::Noability;
        state.items[idx] = ItemId::Airballoon;
        assert!(!state.is_grounded(idx));

        state.items[idx] = ItemId::default();
        state.volatiles[idx] = Volatiles::MAGNET_RISE;
        assert!(!state.is_grounded(idx));

        state.volatiles[idx] = Volatiles::empty();
        state.types[idx][0] = Type::Flying;
        state.gravity = true;
        assert!(state.is_grounded(idx));

        state.gravity = false;
        state.items[idx] = ItemId::Ironball;
        assert!(state.is_grounded(idx));

        state.items[idx] = ItemId::default();
        state.types[idx][0] = Type::Normal;
        state.abilities[idx] = AbilityId::Levitate;
        state.volatiles[idx] = Volatiles::INGRAIN;
        assert!(state.is_grounded(idx));

        state.volatiles[idx] = Volatiles::SMACK_DOWN;
        state.abilities[idx] = AbilityId::Noability;
        state.items[idx] = ItemId::Airballoon;
        assert!(state.is_grounded(idx));
    }

    #[test]
    fn test_effective_speed_paralysis() {
        let mut state = BattleState::new();
        let idx = 0;
        state.stats[idx][5] = 100;
        
        assert_eq!(state.effective_speed(idx), 100);
        
        state.status[idx] = Status::PARALYSIS;
        assert_eq!(state.effective_speed(idx), 50);
    }
    
    #[test]
    fn test_effective_speed_tailwind() {
        let mut state = BattleState::new();
        let idx = 0;
        state.stats[idx][5] = 100;
        
        state.side_conditions[0].tailwind_turns = 3;
        assert_eq!(state.effective_speed(idx), 200);
        
        state.side_conditions[0] = SideConditions::default();
        state.side_conditions[1].tailwind_turns = 3;
        assert_eq!(state.effective_speed(idx), 100);
        
        let idx2 = 6;
        state.stats[idx2][5] = 100;
        assert_eq!(state.effective_speed(idx2), 200);
    }
    
    #[test]
    fn test_effective_speed_weather_abilities() {
        let mut state = BattleState::new();
        let idx = 0;
        state.stats[idx][5] = 100;
        
        state.abilities[idx] = AbilityId::Swiftswim;
        state.weather = 2; // Rain
        assert_eq!(state.effective_speed(idx), 200);
        
        state.weather = 0;
        assert_eq!(state.effective_speed(idx), 100);
        
        state.abilities[idx] = AbilityId::Chlorophyll;
        state.weather = 1; // Sun
        assert_eq!(state.effective_speed(idx), 200);
        
        state.abilities[idx] = AbilityId::Sandrush;
        state.weather = 3; // Sand
        assert_eq!(state.effective_speed(idx), 200);
    }
    
    #[test]
    fn test_effective_speed_items() {
        let mut state = BattleState::new();
        let idx = 0;
        state.stats[idx][5] = 100;
        
        state.items[idx] = ItemId::Choicescarf;
        assert_eq!(state.effective_speed(idx), 150);
        
        state.items[idx] = ItemId::Ironball;
        assert_eq!(state.effective_speed(idx), 50);
    }
    
    #[test]
    fn test_effective_speed_stacking() {
        let mut state = BattleState::new();
        let idx = 0;
        state.stats[idx][5] = 100;
        
        state.boosts[idx][4] = 1; // 150
        state.side_conditions[0].tailwind_turns = 3; // 300
        state.items[idx] = ItemId::Choicescarf; // 450
        assert_eq!(state.effective_speed(idx), 450);
    }
    
    #[test]
    fn test_turn_order_priority() {
        let state = BattleState::new();
        
        assert_eq!(
            state.compare_turn_order(0, 1, 6, 0),
            TurnOrder::First
        );
        assert_eq!(
            state.compare_turn_order(0, 0, 6, 1),
            TurnOrder::Second
        );
    }
    
    #[test]
    fn test_turn_order_speed() {
        let mut state = BattleState::new();
        state.stats[0][5] = 100;
        state.stats[6][5] = 80;
        
        assert_eq!(
            state.compare_turn_order(0, 0, 6, 0),
            TurnOrder::First
        );
        assert_eq!(
            state.compare_turn_order(6, 0, 0, 0),
            TurnOrder::Second
        );
    }
    
    #[test]
    fn test_turn_order_trick_room() {
        let mut state = BattleState::new();
        state.stats[0][5] = 100;
        state.stats[6][5] = 80;
        state.trick_room = true;
        
        assert_eq!(
            state.compare_turn_order(0, 0, 6, 0),
            TurnOrder::Second
        );
        assert_eq!(
            state.compare_turn_order(6, 0, 0, 0),
            TurnOrder::First
        );
    }
    
    #[test]
    fn test_turn_order_tie() {
        let mut state = BattleState::new();
        state.stats[0][5] = 100;
        state.stats[6][5] = 100;
        
        assert_eq!(
            state.compare_turn_order(0, 0, 6, 0),
            TurnOrder::Tie
        );
    }

    #[test]
    fn test_hazard_damage_stealth_rock() {
        let mut state = BattleState::new();
        let idx = 0;
        state.max_hp[idx] = 100;
        state.hp[idx] = 100;
        // Charizard: Fire/Flying -> 4x weakness to Rock
        state.types[idx] = [Type::Fire, Type::Flying];
        state.side_conditions[0].stealth_rock = true;

        let dmg = state.apply_entry_hazards(idx);
        // 4x weakness = 1/2 damage = 50
        assert_eq!(dmg, 50);
        assert_eq!(state.hp[idx], 50);
    }

    #[test]
    fn test_hazard_damage_spikes() {
        let mut state = BattleState::new();
        let idx = 0;
        state.max_hp[idx] = 100;
        state.hp[idx] = 100;
        state.types[idx] = [Type::Normal, Type::Normal]; // Grounded
        state.side_conditions[0].spikes_layers = 1; // 1 layer = 1/8 = 12

        let dmg = state.apply_entry_hazards(idx);
        assert_eq!(dmg, 12);
        assert_eq!(state.hp[idx], 88);

        // Test immune (Flying)
        state.hp[idx] = 100;
        state.types[idx] = [Type::Flying, Type::Normal];
        let dmg = state.apply_entry_hazards(idx);
        assert_eq!(dmg, 0);
    }

    #[test]
    fn test_toxic_spikes_absorption() {
        let mut state = BattleState::new();
        let idx = 0;
        state.types[idx] = [Type::Poison, Type::Normal]; // Grounded Poison
        state.side_conditions[0].toxic_spikes_layers = 1;

        state.apply_entry_hazards(idx);

        assert_eq!(state.side_conditions[0].toxic_spikes_layers, 0, "Poison type should absorb Toxic Spikes");
        assert_eq!(state.status[idx], Status::NONE, "Absorbing shouldn't poison");
    }

    #[test]
    fn test_screen_modifier() {
        let mut state = BattleState::new();
        let idx = 0;
        state.side_conditions[0].reflect_turns = 5;

        // Physical + Reflect = 0.5x (2048)
        assert_eq!(state.get_screen_modifier(idx, MoveCategory::Physical), 2048);
        // Special + Reflect = 1.0x (4096)
        assert_eq!(state.get_screen_modifier(idx, MoveCategory::Special), 4096);

        state.side_conditions[0].light_screen_turns = 5;
        // Special + Light Screen = 0.5x
        assert_eq!(state.get_screen_modifier(idx, MoveCategory::Special), 2048);
    }

    #[test]
    fn test_apply_forme_change() {
        let mut state = BattleState::new();
        let idx = 0;

        // Setup base: Charizard
        let charizard = SpeciesId::from_str("charizard").unwrap();
        state.species[idx] = charizard;
        state.types[idx] = [Type::Fire, Type::Flying];
        state.weight[idx] = charizard.data().weight;
        state.abilities[idx] = AbilityId::Blaze; // Default

        // Apply Mega X
        let mega_x = SpeciesId::from_str("charizardmegax").unwrap();
        state.apply_forme_change(idx, mega_x);

        // Verify changes
        assert_eq!(state.species[idx], mega_x);
        assert_eq!(state.types[idx][0], Type::Fire);
        assert_eq!(state.types[idx][1], Type::Dragon);
        assert_eq!(state.weight[idx], mega_x.data().weight);
        assert_eq!(state.abilities[idx], AbilityId::Toughclaws);
        assert!(state.transformed[idx]);
    }
}
