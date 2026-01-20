//! Generation-specific mechanics abstraction.
//!
//! Each generation implements the `GenMechanics` trait, providing constants
//! and behaviors specific to that generation. Gen 9 is the "canonical" 
//! implementation; older generations are defined as deltas from their successor.
//!
//! # Design Philosophy
//!
//! - **Gen 9 is the base**: All default trait methods reflect Gen 9 behavior
//! - **Older gens override**: Only mechanics that differ need to be overridden
//! - **Custom rulesets**: Fan formats can implement `GenMechanics` with arbitrary rules

mod gen9;
mod gen8;
mod gen7;
mod gen6;
mod gen5;
mod gen4;
mod gen3;
mod gen2;
mod gen1;

pub use gen9::Gen9;
pub use gen8::Gen8;
pub use gen7::Gen7;
pub use gen6::Gen6;
pub use gen5::Gen5;
pub use gen4::Gen4;
pub use gen3::Gen3;
pub use gen2::Gen2;
pub use gen1::Gen1;

use crate::types::Type;
use crate::damage::{DamageContext, DamageResult, Modifier};

/// Fixed-point scale for modifiers (4096 = 1.0x)
pub const MOD_SCALE: u16 = 4096;

/// Weather conditions
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Weather {
    #[default]
    None = 0,
    Sun = 1,
    Rain = 2,
    Sand = 3,
    Hail = 4,
    Snow = 5,          // Gen 9 replaced Hail with Snow
    HarshSun = 6,      // Primal Groudon
    HeavyRain = 7,     // Primal Kyogre
    StrongWinds = 8,   // Mega Rayquaza
}

impl Weather {
    /// Convert from raw u8 stored in BattleState
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Weather::Sun,
            2 => Weather::Rain,
            3 => Weather::Sand,
            4 => Weather::Hail,
            5 => Weather::Snow,
            6 => Weather::HarshSun,
            7 => Weather::HeavyRain,
            8 => Weather::StrongWinds,
            _ => Weather::None,
        }
    }
}

/// Terrain types
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Terrain {
    #[default]
    None = 0,
    Electric = 1,
    Grassy = 2,
    Psychic = 3,
    Misty = 4,
}

impl Terrain {
    /// Convert from raw u8 stored in BattleState
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Terrain::Electric,
            2 => Terrain::Grassy,
            3 => Terrain::Psychic,
            4 => Terrain::Misty,
            _ => Terrain::None,
        }
    }
}

/// Generation-specific mechanics trait.
///
/// Implementors provide generation-specific constants and behavior overrides.
/// Default implementations reflect Gen 9 (Scarlet/Violet) mechanics.
pub trait GenMechanics: Copy + Clone + Send + Sync + 'static {
    /// Generation number (1-9, or 0 for custom)
    const GEN: u8;
    
    /// Get the generation number at runtime
    fn generation(&self) -> u8 {
        Self::GEN
    }
    
    // ========================================================================
    // Damage Calculation Entry Point
    // ========================================================================

    /// Calculate damage for a context.
    ///
    /// The default implementation uses the standard Gen 3+ formula.
    /// Gen 1-2 overrides this with their specific logic.
    fn calculate_damage(&self, ctx: &DamageContext<Self>) -> DamageResult {
        crate::damage::formula::calculate_standard(*ctx)
    }

    // ========================================================================
    // Damage Modifiers
    // ========================================================================
    
    /// Critical hit multiplier in 4096-scale.
    /// Gen 6+: 1.5x (6144), Gen 2-5: 2.0x (8192), Gen 1: special formula
    fn crit_multiplier(&self) -> Modifier {
        Modifier::ONE_POINT_FIVE // 1.5x for Gen 6+
    }
    
    /// STAB (Same Type Attack Bonus) multiplier in 4096-scale.
    ///
    /// # Arguments
    /// * `has_adaptability` - Whether the attacker has Adaptability
    /// * `is_tera_stab` - Whether this is a Tera-boosted STAB (Gen 9 only)
    fn stab_multiplier(&self, has_adaptability: bool, is_tera_stab: bool) -> Modifier {
        match (has_adaptability, is_tera_stab) {
            (true, _) => Modifier::DOUBLE,      // 2.0x with Adaptability
            (false, true) => Modifier::DOUBLE,  // 2.0x with Tera STAB
            (false, false) => Modifier::ONE_POINT_FIVE, // 1.5x normal STAB
        }
    }
    
    /// Weather damage modifier in 4096-scale.
    ///
    /// Returns `Some(modifier)` if weather affects this move type, `None` otherwise.
    fn weather_modifier(&self, weather: Weather, move_type: Type) -> Option<Modifier> {
        match (weather, move_type) {
            // Sun boosts Fire, weakens Water
            (Weather::Sun | Weather::HarshSun, Type::Fire) => Some(Modifier::ONE_POINT_FIVE),  // 1.5x
            (Weather::Sun | Weather::HarshSun, Type::Water) => Some(Modifier::HALF), // 0.5x
            
            // Rain boosts Water, weakens Fire
            (Weather::Rain | Weather::HeavyRain, Type::Water) => Some(Modifier::ONE_POINT_FIVE), // 1.5x
            (Weather::Rain | Weather::HeavyRain, Type::Fire) => Some(Modifier::HALF),  // 0.5x
            
            // Harsh Sun: Water moves fail entirely (handled elsewhere)
            // Heavy Rain: Fire moves fail entirely (handled elsewhere)
            
            _ => None,
        }
    }
    
    /// Terrain damage modifier in 4096-scale.
    ///
    /// Returns `Some(modifier)` if terrain affects this move, `None` otherwise.
    /// Note: Terrain only affects grounded Pokémon.
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<Modifier> {
        if !is_grounded {
            return None;
        }
        
        match (terrain, move_type) {
            (Terrain::Electric, Type::Electric) => Some(Modifier::ONE_POINT_THREE), // 1.3x (Gen 8+)
            (Terrain::Grassy, Type::Grass) => Some(Modifier::ONE_POINT_THREE),      // 1.3x
            (Terrain::Psychic, Type::Psychic) => Some(Modifier::ONE_POINT_THREE),   // 1.3x
            // Misty Terrain: 0.5x to Dragon moves hitting grounded targets
            (Terrain::Misty, Type::Dragon) => Some(Modifier::HALF),      // 0.5x
            _ => None,
        }
    }
    
    // ========================================================================
    // Mechanical Differences
    // ========================================================================
    
    /// Whether abilities exist in this generation.
    /// Gen 1-2: false, Gen 3+: true
    fn has_abilities(&self) -> bool {
        Self::GEN >= 3
    }
    
    /// Whether held items affect battle in this generation.
    /// Gen 1: false, Gen 2+: true
    fn has_held_items(&self) -> bool {
        Self::GEN >= 2
    }
    
    /// Whether the Physical/Special split exists.
    /// Gen 1-3: false (determined by type), Gen 4+: true (per-move)
    fn uses_physical_special_split(&self) -> bool {
        Self::GEN >= 4
    }
    
    /// Whether Terastallization exists.
    /// Only Gen 9.
    fn has_terastallization(&self) -> bool {
        Self::GEN >= 9
    }
    
    /// Whether Mega Evolution exists.
    /// Gen 6-7.
    fn has_mega_evolution(&self) -> bool {
        Self::GEN >= 6 && Self::GEN <= 7
    }

    /// Whether a species can Mega Evolve (checked via lookup usually)
    /// This is a helper to check if the mechanic is globally enabled
    /// AND if the specific logic allows it.
    fn can_mega_evolve(&self) -> bool {
        false // Override in Gen 6/7
    }

    /// Get stats for a Mega Evolution if applicable
    fn mega_stat_boosts(&self, _species: crate::species::SpeciesId) -> Option<[u8; 6]> {
        None
    }
    
    /// Whether Z-Moves exist.
    /// Gen 7.
    fn has_z_moves(&self) -> bool {
        Self::GEN == 7
    }
    
    /// Whether Dynamax exists.
    /// Gen 8.
    fn has_dynamax(&self) -> bool {
        Self::GEN == 8
    }

    /// Dynamax HP multiplier.
    /// Gen 8: 2.0x (usually).
    fn dynamax_hp_multiplier(&self) -> f32 {
        1.0
    }

    /// Max Move power based on base move power.
    fn max_move_power(&self, base_power: u16) -> u16 {
        base_power
    }
    
    // ========================================================================
    // Type Chart
    // ========================================================================
    
    /// Calculate type effectiveness multiplier.
    ///
    /// Returns a fixed-point value where 4 = neutral (1x):
    /// - 0 = immune (0x)
    /// - 1 = 0.25x
    /// - 2 = 0.5x
    /// - 4 = 1x
    /// - 8 = 2x
    /// - 16 = 4x
    ///
    /// Default uses the standard type chart. Gen 1 has different interactions
    /// (Ghost/Psychic, etc.) and should override.
    fn type_effectiveness(&self, atk_type: Type, def_type1: Type, def_type2: Option<Type>) -> u8 {
        crate::types::type_effectiveness(atk_type, def_type1, def_type2)
    }
    
    // ========================================================================
    // Burn Modifier
    // ========================================================================
    
    /// Burn damage reduction multiplier for Physical moves (4096-scale).
    /// Default: 0.5x (2048). Returns None if burn doesn't reduce damage.
    fn burn_modifier(&self) -> Modifier {
        Modifier::HALF // 0.5x
    }
}

/// Runtime generation selection for when the generation isn't known at compile time.
#[derive(Clone, Copy, Debug)]
pub enum Generation {
    Gen1(Gen1),
    Gen2(Gen2),
    Gen3(Gen3),
    Gen4(Gen4),
    Gen5(Gen5),
    Gen6(Gen6),
    Gen7(Gen7),
    Gen8(Gen8),
    Gen9(Gen9),
}

impl Default for Generation {
    fn default() -> Self {
        Generation::Gen9(Gen9)
    }
}

impl Generation {
    /// Create a Generation from a numeric value.
    /// Defaults to Gen 9 for unsupported generations.
    pub fn from_num(gen: u8) -> Self {
        match gen {
            1 => Generation::Gen1(Gen1),
            2 => Generation::Gen2(Gen2),
            3 => Generation::Gen3(Gen3),
            4 => Generation::Gen4(Gen4),
            5 => Generation::Gen5(Gen5),
            6 => Generation::Gen6(Gen6),
            7 => Generation::Gen7(Gen7),
            8 => Generation::Gen8(Gen8),
            9 => Generation::Gen9(Gen9),
            _ => Generation::Gen9(Gen9),
        }
    }
    
    /// Get the generation number.
    pub fn num(&self) -> u8 {
        match self {
            Generation::Gen1(_) => 1,
            Generation::Gen2(_) => 2,
            Generation::Gen3(_) => 3,
            Generation::Gen4(_) => 4,
            Generation::Gen5(_) => 5,
            Generation::Gen6(_) => 6,
            Generation::Gen7(_) => 7,
            Generation::Gen8(_) => 8,
            Generation::Gen9(_) => 9,
        }
    }
}

// Implement GenMechanics for the enum by delegating
impl GenMechanics for Generation {
    const GEN: u8 = 0; // Runtime determined

    fn generation(&self) -> u8 {
        self.num()
    }

    fn calculate_damage(&self, ctx: &DamageContext<Self>) -> DamageResult {
        // Macro to avoid repetition? Rust macros here might be overkill or messy.
        // We just manually unwrap and reconstruct context.
        match self {
            Generation::Gen1(g) => {
                let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
            Generation::Gen2(g) => {
                let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
            Generation::Gen3(g) => {
                let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
            Generation::Gen4(g) => {
                let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
            Generation::Gen5(g) => {
                 let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
            Generation::Gen6(g) => {
                 let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
            Generation::Gen7(g) => {
                 let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
            Generation::Gen8(g) => {
                 let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
            Generation::Gen9(g) => {
                 let inner = DamageContext {
                    gen: *g,
                    state: ctx.state,
                    attacker: ctx.attacker,
                    defender: ctx.defender,
                    move_id: ctx.move_id,
                    move_data: ctx.move_data,
                    base_power: ctx.base_power,
                    category: ctx.category,
                    move_type: ctx.move_type,
                    is_crit: ctx.is_crit,
                    is_spread: ctx.is_spread,
                    attacker_grounded: ctx.attacker_grounded,
                    defender_grounded: ctx.defender_grounded,
                    chain_mod: ctx.chain_mod,
                    effectiveness: ctx.effectiveness,
                    has_stab: ctx.has_stab,
                    has_adaptability: ctx.has_adaptability,
                    is_tera_stab: ctx.is_tera_stab,
                    attacker_ability: ctx.attacker_ability,
                    defender_ability: ctx.defender_ability,
                };
                g.calculate_damage(&inner)
            },
        }
    }
    
    fn crit_multiplier(&self) -> Modifier {
        match self {
            Generation::Gen1(g) => g.crit_multiplier(),
            Generation::Gen2(g) => g.crit_multiplier(),
            Generation::Gen3(g) => g.crit_multiplier(),
            Generation::Gen4(g) => g.crit_multiplier(),
            Generation::Gen5(g) => g.crit_multiplier(),
            Generation::Gen6(g) => g.crit_multiplier(),
            Generation::Gen7(g) => g.crit_multiplier(),
            Generation::Gen8(g) => g.crit_multiplier(),
            Generation::Gen9(g) => g.crit_multiplier(),
        }
    }
    
    fn stab_multiplier(&self, has_adaptability: bool, is_tera_stab: bool) -> Modifier {
        match self {
            Generation::Gen1(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
            Generation::Gen2(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
            Generation::Gen3(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
            Generation::Gen4(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
            Generation::Gen5(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
            Generation::Gen6(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
            Generation::Gen7(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
            Generation::Gen8(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
            Generation::Gen9(g) => g.stab_multiplier(has_adaptability, is_tera_stab),
        }
    }
    
    fn weather_modifier(&self, weather: Weather, move_type: Type) -> Option<Modifier> {
        match self {
            Generation::Gen1(g) => g.weather_modifier(weather, move_type),
            Generation::Gen2(g) => g.weather_modifier(weather, move_type),
            Generation::Gen3(g) => g.weather_modifier(weather, move_type),
            Generation::Gen4(g) => g.weather_modifier(weather, move_type),
            Generation::Gen5(g) => g.weather_modifier(weather, move_type),
            Generation::Gen6(g) => g.weather_modifier(weather, move_type),
            Generation::Gen7(g) => g.weather_modifier(weather, move_type),
            Generation::Gen8(g) => g.weather_modifier(weather, move_type),
            Generation::Gen9(g) => g.weather_modifier(weather, move_type),
        }
    }
    
    fn terrain_modifier(&self, terrain: Terrain, move_type: Type, is_grounded: bool) -> Option<Modifier> {
        match self {
            Generation::Gen1(g) => g.terrain_modifier(terrain, move_type, is_grounded),
            Generation::Gen2(g) => g.terrain_modifier(terrain, move_type, is_grounded),
            Generation::Gen3(g) => g.terrain_modifier(terrain, move_type, is_grounded),
            Generation::Gen4(g) => g.terrain_modifier(terrain, move_type, is_grounded),
            Generation::Gen5(g) => g.terrain_modifier(terrain, move_type, is_grounded),
            Generation::Gen6(g) => g.terrain_modifier(terrain, move_type, is_grounded),
            Generation::Gen7(g) => g.terrain_modifier(terrain, move_type, is_grounded),
            Generation::Gen8(g) => g.terrain_modifier(terrain, move_type, is_grounded),
            Generation::Gen9(g) => g.terrain_modifier(terrain, move_type, is_grounded),
        }
    }
    
    fn has_abilities(&self) -> bool {
        match self {
            Generation::Gen1(g) => g.has_abilities(),
            Generation::Gen2(g) => g.has_abilities(),
            Generation::Gen3(g) => g.has_abilities(),
            Generation::Gen4(g) => g.has_abilities(),
            Generation::Gen5(g) => g.has_abilities(),
            Generation::Gen6(g) => g.has_abilities(),
            Generation::Gen7(g) => g.has_abilities(),
            Generation::Gen8(g) => g.has_abilities(),
            Generation::Gen9(g) => g.has_abilities(),
        }
    }
    
    fn has_held_items(&self) -> bool {
        match self {
            Generation::Gen1(g) => g.has_held_items(),
            Generation::Gen2(g) => g.has_held_items(),
            Generation::Gen3(g) => g.has_held_items(),
            Generation::Gen4(g) => g.has_held_items(),
            Generation::Gen5(g) => g.has_held_items(),
            Generation::Gen6(g) => g.has_held_items(),
            Generation::Gen7(g) => g.has_held_items(),
            Generation::Gen8(g) => g.has_held_items(),
            Generation::Gen9(g) => g.has_held_items(),
        }
    }
    
    fn uses_physical_special_split(&self) -> bool {
        match self {
            Generation::Gen1(g) => g.uses_physical_special_split(),
            Generation::Gen2(g) => g.uses_physical_special_split(),
            Generation::Gen3(g) => g.uses_physical_special_split(),
            Generation::Gen4(g) => g.uses_physical_special_split(),
            Generation::Gen5(g) => g.uses_physical_special_split(),
            Generation::Gen6(g) => g.uses_physical_special_split(),
            Generation::Gen7(g) => g.uses_physical_special_split(),
            Generation::Gen8(g) => g.uses_physical_special_split(),
            Generation::Gen9(g) => g.uses_physical_special_split(),
        }
    }
    
    fn has_terastallization(&self) -> bool {
        match self {
            Generation::Gen9(g) => g.has_terastallization(),
            _ => false,
        }
    }

    fn has_mega_evolution(&self) -> bool {
        match self {
            Generation::Gen6(g) => g.has_mega_evolution(),
            Generation::Gen7(g) => g.has_mega_evolution(),
            _ => false,
        }
    }

    fn can_mega_evolve(&self) -> bool {
        match self {
            Generation::Gen6(g) => g.can_mega_evolve(),
            Generation::Gen7(g) => g.can_mega_evolve(),
            _ => false,
        }
    }

    fn mega_stat_boosts(&self, species: crate::species::SpeciesId) -> Option<[u8; 6]> {
        match self {
            Generation::Gen6(g) => g.mega_stat_boosts(species),
            Generation::Gen7(g) => g.mega_stat_boosts(species),
            _ => None,
        }
    }

    fn has_z_moves(&self) -> bool {
        match self {
            Generation::Gen7(g) => g.has_z_moves(),
            _ => false,
        }
    }

    fn has_dynamax(&self) -> bool {
        match self {
            Generation::Gen8(g) => g.has_dynamax(),
            _ => false,
        }
    }

    fn dynamax_hp_multiplier(&self) -> f32 {
        match self {
            Generation::Gen8(g) => g.dynamax_hp_multiplier(),
            _ => 1.0,
        }
    }

    fn max_move_power(&self, base_power: u16) -> u16 {
        match self {
            Generation::Gen8(g) => g.max_move_power(base_power),
            _ => base_power,
        }
    }
    
    fn type_effectiveness(&self, atk_type: Type, def_type1: Type, def_type2: Option<Type>) -> u8 {
        match self {
            Generation::Gen1(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
            Generation::Gen2(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
            Generation::Gen3(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
            Generation::Gen4(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
            Generation::Gen5(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
            Generation::Gen6(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
            Generation::Gen7(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
            Generation::Gen8(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
            Generation::Gen9(g) => g.type_effectiveness(atk_type, def_type1, def_type2),
        }
    }
    
    fn burn_modifier(&self) -> Modifier {
        match self {
            Generation::Gen1(g) => g.burn_modifier(),
            Generation::Gen2(g) => g.burn_modifier(),
            Generation::Gen3(g) => g.burn_modifier(),
            Generation::Gen4(g) => g.burn_modifier(),
            Generation::Gen5(g) => g.burn_modifier(),
            Generation::Gen6(g) => g.burn_modifier(),
            Generation::Gen7(g) => g.burn_modifier(),
            Generation::Gen8(g) => g.burn_modifier(),
            Generation::Gen9(g) => g.burn_modifier(),
        }
    }
}
