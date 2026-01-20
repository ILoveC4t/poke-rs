use crate::damage::context::DamageContext;
use crate::damage::generations::GenMechanics;
use crate::moves::MoveId;
use crate::state::{BattleState, Status};
use crate::abilities::AbilityId;
use crate::items::ItemId;

/// Check if a move has variable base power (often 0 in data).
pub fn is_variable_power(move_id: MoveId) -> bool {
    matches!(move_id,
        MoveId::Grassknot | MoveId::Lowkick |
        MoveId::Heavyslam | MoveId::Heatcrash |
        MoveId::Eruption | MoveId::Waterspout |
        MoveId::Flail | MoveId::Reversal
    )
}

/// Calculate effective weight of an entity, applying modifiers.
pub fn get_modified_weight(state: &BattleState, entity_idx: usize, ability: AbilityId) -> u32 {
    let mut weight = state.weight[entity_idx] as u32;
    if weight == 0 {
        weight = state.species[entity_idx].data().weight as u32;
    }

    // Apply ability modifiers
    if ability == AbilityId::Heavymetal {
        weight *= 2;
    } else if ability == AbilityId::Lightmetal {
        weight /= 2;
    }

    // Apply item modifiers
    if state.items[entity_idx] == ItemId::Floatstone {
        weight /= 2;
    }

    weight.max(1)
}

/// Modify base power based on special move logic (weight, HP, etc.)
///
/// This is called BEFORE ability modifiers (Technician, etc.).
/// Returns the modified base power.
pub fn modify_base_power<G: GenMechanics>(ctx: &DamageContext<'_, G>) -> u32 {
    let mut bp = ctx.base_power as u32;
    
    // ========================================================================
    // Weight-based moves
    // ========================================================================
    
    // Grass Knot / Low Kick: BP based on target's weight
    // Weight is stored in 0.1kg units (fixed-point), so 200kg = 2000
    if ctx.move_id == MoveId::Grassknot || ctx.move_id == MoveId::Lowkick {
        let weight = get_modified_weight(ctx.state, ctx.defender, ctx.defender_ability);
        bp = match weight {
            w if w >= 2000 => 120, // >= 200kg
            w if w >= 1000 => 100, // >= 100kg
            w if w >= 500 => 80,   // >= 50kg
            w if w >= 250 => 60,   // >= 25kg
            w if w >= 100 => 40,   // >= 10kg
            _ => 20,               // < 10kg
        };
        return bp; // Prioritize weight calculation over base power (these moves usually have 0 BP in data)
    }
    
    // Heavy Slam / Heat Crash: BP based on weight ratio (attacker / defender)
    if ctx.move_id == MoveId::Heavyslam || ctx.move_id == MoveId::Heatcrash {
        let attacker_weight = get_modified_weight(ctx.state, ctx.attacker, ctx.attacker_ability);
        let defender_weight = get_modified_weight(ctx.state, ctx.defender, ctx.defender_ability);

        // Multiply by 10 for precision before dividing
        let ratio_x10 = (attacker_weight * 10) / defender_weight;
        bp = match ratio_x10 {
            r if r >= 50 => 120, // >= 5x
            r if r >= 40 => 100, // >= 4x
            r if r >= 30 => 80,  // >= 3x
            r if r >= 20 => 60,  // >= 2x
            _ => 40,             // < 2x
        };
        return bp;
    }
    
    // ========================================================================
    // HP-based moves
    // ========================================================================
    
    // Eruption / Water Spout: BP = 150 * currentHP / maxHP
    if ctx.move_id == MoveId::Eruption || ctx.move_id == MoveId::Waterspout {
        let current_hp = ctx.state.hp[ctx.attacker] as u32;
        let max_hp = ctx.state.max_hp[ctx.attacker] as u32;
        bp = (150 * current_hp / max_hp.max(1)).max(1);
        return bp;
    }
    
    // Flail / Reversal: BP increases as HP decreases
    if ctx.move_id == MoveId::Flail || ctx.move_id == MoveId::Reversal {
        let current_hp = ctx.state.hp[ctx.attacker] as u32;
        let max_hp = ctx.state.max_hp[ctx.attacker] as u32;
        // HP% thresholds: 48/255 = ~4.7%, 80/255 = ~10.2%, etc.
        let hp_percent = (current_hp * 48) / max_hp.max(1);
        bp = match hp_percent {
            0..=1 => 200,   // < 4.17%
            2..=4 => 150,   // < 10.42%
            5..=9 => 100,   // < 20.83%
            10..=16 => 80,  // < 35.42%
            17..=32 => 40,  // < 68.75%
            _ => 20,        // >= 68.75%
        };
        return bp;
    }
    
    // ========================================================================
    // Status-based modifiers
    // ========================================================================

    // Facade: 2x if burned, poisoned, or paralyzed
    if ctx.move_id == MoveId::Facade {
        let status = ctx.attacker_status();
        if status.intersects(Status::BURN | Status::POISON | Status::TOXIC | Status::PARALYSIS) {
            bp *= 2;
        }
    }
    
    // TODO: Venoshock (2x vs poisoned), Hex (2x vs statused), etc.
    // These should be added here.
    
    // ========================================================================
    // Other Conditional Power
    // ========================================================================
    
    // TODO: Brine (2x below 50% HP)
    // TODO: Assurance, Payback, Avalanche, Revenge, etc.
    
    bp
}
