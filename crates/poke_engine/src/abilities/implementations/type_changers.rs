//! Type-changing ability implementations.
//!
//! Abilities that change the type of moves: Aerilate, Pixilate, Refrigerate,
//! Galvanize, Normalize, and Liquid Voice.

use crate::state::BattleState;
use crate::moves::{Move, MoveFlags};
use crate::types::Type;

// ============================================================================
// Aerilate: Normal -> Flying
// ============================================================================

pub fn aerilate(
    _state: &BattleState,
    _attacker: usize,
    _move_data: &Move,
    current_type: Type,
) -> Type {
    if current_type == Type::Normal {
        Type::Flying
    } else {
        current_type
    }
}

// ============================================================================
// Pixilate: Normal -> Fairy
// ============================================================================

pub fn pixilate(
    _state: &BattleState,
    _attacker: usize,
    _move_data: &Move,
    current_type: Type,
) -> Type {
    if current_type == Type::Normal {
        Type::Fairy
    } else {
        current_type
    }
}

// ============================================================================
// Refrigerate: Normal -> Ice
// ============================================================================

pub fn refrigerate(
    _state: &BattleState,
    _attacker: usize,
    _move_data: &Move,
    current_type: Type,
) -> Type {
    if current_type == Type::Normal {
        Type::Ice
    } else {
        current_type
    }
}

// ============================================================================
// Galvanize: Normal -> Electric
// ============================================================================

pub fn galvanize(
    _state: &BattleState,
    _attacker: usize,
    _move_data: &Move,
    current_type: Type,
) -> Type {
    if current_type == Type::Normal {
        Type::Electric
    } else {
        current_type
    }
}

// ============================================================================
// Normalize: Any type -> Normal
// ============================================================================

pub fn normalize(
    _state: &BattleState,
    _attacker: usize,
    _move_data: &Move,
    _current_type: Type,
) -> Type {
    Type::Normal
}

// ============================================================================
// Liquid Voice: Sound moves -> Water
// ============================================================================

pub fn liquid_voice(
    _state: &BattleState,
    _attacker: usize,
    move_data: &Move,
    current_type: Type,
) -> Type {
    if move_data.flags.contains(MoveFlags::SOUND) {
        Type::Water
    } else {
        current_type
    }
}
