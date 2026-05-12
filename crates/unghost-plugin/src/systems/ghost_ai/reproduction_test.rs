#[cfg(test)]
mod tests {
    use unghost_core::components::logic::ghost_influence::{GhostInfluence, InfluenceType};
    use unghost_core::components::logic::ghost_sprite::GhostSprite;
    use unghost_core::resources::object_interaction::ObjectInteractionConfig;
    use unspatial_core::position::Position;
    use undifficulty_core::difficulty::Difficulty;
    use undifficulty_core::difficulty_settings::DifficultySettings;

    // We replicate the logic here because mocking the Bevy Query and Resources
    // for a simple math test is overly complex and fragile.

    fn calculate_score(
        dest: Position,
        ghost_pos: Position,
        ghost_sprite: &GhostSprite,
        objects: &[(Position, GhostInfluence)],
        config: &ObjectInteractionConfig,
        difficulty: Difficulty,
    ) -> f32 {
        let mut score = 1.0;

        // Attraction
        let mut attraction_score = 0.0;
        for (obj_pos, influence) in objects {
            // Formula: 1 / (dist + 5.0) with Z-factor 12
            let dx = dest.x - obj_pos.x;
            let dy = dest.y - obj_pos.y;
            let dz = (dest.z - obj_pos.z) * 12.0;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if influence.influence_type == InfluenceType::Attractive {
                attraction_score +=
                    config.attractive_influence_multiplier * influence.charge_value / (dist + 5.0);
            }
        }

        // Difficulty scaling (inverted breach attraction factor)
        score += attraction_score / difficulty.ghost_attraction_to_breach().max(0.1);

        // Movement Penalties (Production math: penalty = 1.0 + abs(penalty_score) / 10.0)
        let mut penalty = 1.0;
        if dest.z.round() != ghost_pos.z.round() {
            let mut penalty_score = 10.0; // FLOOR_CHANGE_PENALTY_BASE (absolute)
            if ghost_sprite.floor_stay_timer < 8.0 {
                penalty_score *= 20.0; // Hysteresis multiplier
            }
            penalty += penalty_score / 10.0;
        }

        score / penalty
    }

    #[test]
    fn test_ghost_attraction_tarin_library() {
        let config = ObjectInteractionConfig {
            num_destination_points_to_sample: 20,
            attractive_influence_multiplier: 1.0,
            ..Default::default()
        };

        let difficulty = Difficulty::MasterChallenge; // User said high difficulty

        let ghost_pos = Position::new_i64(32, 32, 2); // Ghost starts on different floor

        let mut ghost_sprite = GhostSprite::default();
        ghost_sprite.floor_stay_timer = 10.0; // Allowed to change floor

        // Pile of 9 objects near the breach (Floor 1)
        let mut objects = Vec::new();
        for _ in 0..9 {
            objects.push((
                Position::new_i64(35, 35, 1),
                GhostInfluence {
                    influence_type: InfluenceType::Attractive,
                    charge_value: 1.0,
                },
            ));
        }

        // Sample points
        let same_floor_dest = Position::new_i64(30, 30, 2);
        let other_floor_pile_dest = Position::new_i64(35, 35, 1);

        let score_same =
            calculate_score(same_floor_dest, ghost_pos, &ghost_sprite, &objects, &config, difficulty);
        let score_other = calculate_score(
            other_floor_pile_dest,
            ghost_pos,
            &ghost_sprite,
            &objects,
            &config,
            difficulty,
        );

        println!("Difficulty: {:?}", difficulty);
        println!("Score Same Floor: {:.4}", score_same);
        println!("Score Other Floor (with pile): {:.4}", score_other);

        // With Master Challenge (0.1 -> divisor 0.1), objects are x10
        // Attraction at distance 0 (exact hit): 9 * 1.0 / (0 + 5.0) = 1.8
        // Multiplied by difficulty: 1.8 / 0.1 = 18.0
        // Total base + attraction = 1.0 + 18.0 = 19.0
        // Penalty for floor change (normal): 1.0 + 10.0 / 10.0 = 2.0
        // Final score = 19.0 / 2.0 = 9.5

        // 9.5 > 1.0, so the ghost SHOULD move to the other floor!
        assert!(
            score_other > score_same,
            "Ghost should be pulled to the other floor pile"
        );
    }

    #[test]
    fn test_ghost_hysteresis_prevents_jump() {
        let config = ObjectInteractionConfig::default();
        let difficulty = Difficulty::MasterChallenge;

        let ghost_pos = Position::new_i64(32, 32, 2);
        let mut ghost_sprite = GhostSprite::default();
        ghost_sprite.floor_stay_timer = 2.0; // Too soon to change floor!

        let mut objects = Vec::new();
        for _ in 0..9 {
            objects.push((
                Position::new_i64(35, 35, 1),
                GhostInfluence {
                    influence_type: InfluenceType::Attractive,
                    charge_value: 1.0,
                },
            ));
        }

        let same_floor_dest = Position::new_i64(32, 32, 2);
        let other_floor_pile_dest = Position::new_i64(35, 35, 1);

        let score_same =
            calculate_score(same_floor_dest, ghost_pos, &ghost_sprite, &objects, &config, difficulty);
        let score_other = calculate_score(
            other_floor_pile_dest,
            ghost_pos,
            &ghost_sprite,
            &objects,
            &config,
            difficulty,
        );

        println!("Score Same Floor (Hysteresis): {:.4}", score_same);
        println!("Score Other Floor (Hysteresis): {:.4}", score_other);

        // Penalty (Hysteresis): 1.0 + (10 * 20) / 10 = 21.0
        // Final score = 19.0 / 21.0 = 0.9047
        // 0.9047 < 5.0 (Score Same Floor), so ghost STAYS on current floor.
        assert!(
            score_other < score_same,
            "Hysteresis should prevent floor jumping"
        );
    }
}
