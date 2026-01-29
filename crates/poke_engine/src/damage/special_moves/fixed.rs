use crate::moves::MoveId;
use crate::state::BattleState;
use crate::types::Type;

/// Check if a move deals fixed damage (not affected by stats/type).
///
/// Returns `Some(damage)` for fixed damage moves, `None` otherwise.
pub fn get_fixed_damage(
    move_id: MoveId,
    state: &BattleState,
    attacker: usize,
    defender: usize,
) -> Option<u16> {
    let move_name = move_id.data().name;
    let level = state.level[attacker] as u16;

    // Check for immunity first for relevant moves
    let defender_types = state.types[defender];

    match move_name {
        // ====================================================================
        // Level-based fixed damage
        // ====================================================================
        "Night Shade" => {
            // Ghost-type move, Normal-types are immune
            if defender_types[0] == Type::Normal || defender_types[1] == Type::Normal {
                Some(0)
            } else {
                Some(level)
            }
        }
        "Seismic Toss" => {
            // Fighting-type move, Ghost-types are immune
            if defender_types[0] == Type::Ghost || defender_types[1] == Type::Ghost {
                Some(0)
            } else {
                Some(level)
            }
        }

        // ====================================================================
        // Constant fixed damage (removed in Gen 5+)
        // ====================================================================
        "Dragon Rage" => {
            // Dragon-type, Fairy-types are immune (Gen 6+)
            if defender_types[0] == Type::Fairy || defender_types[1] == Type::Fairy {
                Some(0)
            } else {
                Some(40)
            }
        }
        "Sonic Boom" => {
            // Normal-type, Ghost-types are immune
            if defender_types[0] == Type::Ghost || defender_types[1] == Type::Ghost {
                Some(0)
            } else {
                Some(20)
            }
        }

        // ====================================================================
        // HP percentage-based damage
        // ====================================================================

        // Super Fang / Nature's Madness: 50% of target's current HP
        "Super Fang" | "Nature's Madness" => {
            // Normal-type (Super Fang) - Ghost immune
            // Fairy-type (Nature's Madness) - no immunities by type
            if move_name == "Super Fang"
                && (defender_types[0] == Type::Ghost || defender_types[1] == Type::Ghost)
            {
                Some(0)
            } else {
                Some((state.hp[defender] / 2).max(1))
            }
        }

        // Guardian of Alola: 75% of target's current HP
        "Guardian of Alola" => Some((state.hp[defender] * 3 / 4).max(1)),

        // Ruination: 50% of target's current HP (Gen 9)
        "Ruination" => Some((state.hp[defender] / 2).max(1)),

        // ====================================================================
        // Attacker HP-based damage
        // ====================================================================

        // Final Gambit: damage = attacker's current HP (attacker faints)
        "Final Gambit" => {
            // Fighting-type, Ghost-types are immune
            if defender_types[0] == Type::Ghost || defender_types[1] == Type::Ghost {
                Some(0)
            } else {
                Some(state.hp[attacker])
            }
        }

        // ====================================================================
        // Endeavor: special handling (reduces target to attacker's HP)
        // ====================================================================
        "Endeavor" => {
            // Normal-type, Ghost-types are immune
            if defender_types[0] == Type::Ghost || defender_types[1] == Type::Ghost {
                Some(0)
            } else {
                let attacker_hp = state.hp[attacker];
                let defender_hp = state.hp[defender];
                if defender_hp > attacker_hp {
                    Some(defender_hp - attacker_hp)
                } else {
                    Some(0) // Fails if target HP <= attacker HP
                }
            }
        }

        // TODO: Implement these (require battle history tracking)
        // "Counter" => 2x physical damage received this turn
        // "Mirror Coat" => 2x special damage received this turn
        // "Metal Burst" => 1.5x damage received this turn
        // "Bide" => 2x stored damage over 2 turns
        // "Psywave" => Random damage between level * 0.5 and level * 1.5
        _ => None,
    }
}
