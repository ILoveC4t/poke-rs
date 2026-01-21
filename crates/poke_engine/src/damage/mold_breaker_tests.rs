#[cfg(test)]
mod tests {
    use crate::state::BattleState;
    use crate::damage::{DamageContext, Gen9, compute_final_damage};
    use crate::species::SpeciesId;
    use crate::types::Type;
    use crate::abilities::AbilityId;
    use crate::moves::MoveId;
    use crate::damage::special_moves::modify_base_power;

    #[test]
    fn test_mold_breaker_ignores_levitate() {
        let mut state = BattleState::new();
        let gen = Gen9;

        // Attacker: Pinsir (Mold Breaker)
        state.species[0] = SpeciesId::from_str("pinsir").unwrap_or(SpeciesId(127));
        state.types[0] = [Type::Bug, Type::Bug];
        state.abilities[0] = AbilityId::Moldbreaker;

        // Defender: Weezing (Levitate)
        // Weezing is Poison. Ground is 2x.
        state.species[1] = SpeciesId::from_str("weezing").unwrap_or(SpeciesId(110));
        state.types[1] = [Type::Poison, Type::Poison];
        state.abilities[1] = AbilityId::Levitate;

        // Move: Earthquake (Ground)
        let move_id = MoveId::Earthquake;

        let ctx = DamageContext::new(gen, &state, 0, 1, move_id, false);

        // Should be Super Effective (2x) because Levitate is ignored
        // Poison is weak to Ground.
        assert_eq!(ctx.effectiveness, 8, "Mold Breaker should allow Ground move to hit Levitate user (2x vs Poison)");
    }

    #[test]
    fn test_mold_breaker_ignores_multiscale() {
        let mut state = BattleState::new();
        let gen = Gen9;

        // Attacker: Pinsir (Mold Breaker)
        state.species[0] = SpeciesId::from_str("pinsir").unwrap_or(SpeciesId(127));
        state.types[0] = [Type::Bug, Type::Bug]; // Set types to avoid accidental STAB with Normal moves
        state.abilities[0] = AbilityId::Moldbreaker;
        state.stats[0][1] = 100; // Atk

        // Defender: Dragonite (Multiscale)
        state.species[1] = SpeciesId::from_str("dragonite").unwrap_or(SpeciesId(149));
        state.types[1] = [Type::Dragon, Type::Flying];
        state.abilities[1] = AbilityId::Multiscale;
        state.stats[1][2] = 100; // Def
        // Full HP
        state.hp[1] = 100;
        state.max_hp[1] = 100;

        // Move: Tackle (Physical, Neutral)
        let move_id = MoveId::Tackle;

        let ctx = DamageContext::new(gen, &state, 0, 1, move_id, false);

        // Calculate damage
        let rolls = compute_final_damage(&ctx, 100); // 100 base damage
        let damage = rolls[0]; // min roll (85)

        // Without Multiscale: 85.
        // With Multiscale: 85 * 0.5 = 42.

        assert_eq!(damage, 85, "Mold Breaker should ignore Multiscale reduction (got {})", damage);
    }

    #[test]
    fn test_mold_breaker_ignores_heavy_metal() {
        let mut state = BattleState::new();
        let gen = Gen9;

        // Attacker: Mold Breaker
        state.abilities[0] = AbilityId::Moldbreaker;

        // Defender: Heavy Metal (doubles weight)
        state.species[1] = SpeciesId::from_str("bronzong").unwrap_or(SpeciesId(437));
        state.abilities[1] = AbilityId::Heavymetal;
        // Base weight of Bronzong is 187kg -> 1870 in 0.1kg units.
        state.weight[1] = 0; // Use species default

        // Move: Grass Knot
        let move_id = MoveId::Grassknot;

        let ctx = DamageContext::new(gen, &state, 0, 1, move_id, false);
        let bp = modify_base_power(&ctx); // This calls get_modified_weight

        // 187kg -> BP 100 (>= 100kg, < 200kg).
        // With Heavy Metal: 374kg -> BP 120 (>= 200kg).

        // With Mold Breaker, should be BP 100.
        assert_eq!(bp, 100, "Mold Breaker should ignore Heavy Metal weight increase");
    }

    #[test]
    fn test_mold_breaker_respects_shadow_shield() {
        let mut state = BattleState::new();
        let gen = Gen9;

        // Attacker: Pinsir (Mold Breaker)
        state.species[0] = SpeciesId::from_str("pinsir").unwrap_or(SpeciesId(127));
        state.abilities[0] = AbilityId::Moldbreaker;

        // Defender: Lunala (Shadow Shield)
        state.species[1] = SpeciesId::from_str("lunala").unwrap_or(SpeciesId(792));
        state.types[1] = [Type::Psychic, Type::Ghost];
        state.abilities[1] = AbilityId::Shadowshield;
        state.hp[1] = 100;
        state.max_hp[1] = 100;

        // Move: Dark Pulse (Dark) - 2x vs Ghost/Psychic (4x effective!)
        let move_id = MoveId::Darkpulse;

        let ctx = DamageContext::new(gen, &state, 0, 1, move_id, false);

        // Calculate damage
        let rolls = compute_final_damage(&ctx, 100);
        let damage = rolls[0]; // min roll

        // Shadow Shield (0.5x) should apply.
        // Roll 85 (min).
        // Effectiveness 16 (4x). 85 * 4 = 340.
        // Shadow Shield: 340 * 0.5 = 170.
        // If ignored: 340.

        assert_eq!(damage, 170, "Mold Breaker should NOT ignore Shadow Shield");
    }
}
