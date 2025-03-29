use log::trace;
use rand::rngs::StdRng;

use crate::{
    attack_ids::AttackId,
    card_ids::CardId,
    hooks::get_damage_from_attack,
    types::{EnergyType, StatusCondition},
    State,
};

use super::{
    apply_action_helpers::{
        apply_common_mutation, handle_attack_damage, FnMutation, Mutation, Mutations, Probabilities,
    },
    Action, SimpleAction,
};

// This is a reducer of all actions relating to attacks.
pub(crate) fn forecast_attack(
    acting_player: usize,
    state: &State,
    index: usize,
) -> (Probabilities, Mutations) {
    let active = state.get_active(acting_player);
    let attack = active.card.get_attacks()[index].clone();
    if attack.effect.is_none() {
        index_damage_doutcome(index, |_, _, _| {})
    } else {
        forecast_effect_attack(acting_player, state, index)
    }
}

/// Handles attacks that have effects.
fn forecast_effect_attack(
    acting_player: usize,
    state: &State,
    index: usize,
) -> (Probabilities, Mutations) {
    let attack_id = {
        let active = state.get_active(acting_player);
        AttackId::from_pokemon_index(&active.get_id()[..], index).unwrap_or_else(|| {
            panic!(
                "Attack not found for Pokemon: {:?} {:?} {:?}",
                active.card,
                active.card.get_attacks(),
                index
            )
        })
    };
    match attack_id {
        AttackId::A1003VenusaurMegaDrain => self_heal_attack(30, index),
        AttackId::A1004VenusaurExGiantBloom => self_heal_attack(30, index),
        AttackId::A1013VileplumeSoothingScent => damage_status_attack(80, StatusCondition::Asleep),
        AttackId::A1017VenomothPoisonPowder => damage_status_attack(30, StatusCondition::Poisoned),
        AttackId::A1022ExeggutorStomp => probabilistic_damage_attack(vec![0.5, 0.5], vec![30, 60]),
        AttackId::A1023ExeggutorExTropicalSwing => {
            probabilistic_damage_attack(vec![0.5, 0.5], vec![40, 80])
        }
        AttackId::A1024TangelaAbsorb => self_heal_attack(10, index),
        AttackId::A1026PinsirDoubleHorn => {
            probabilistic_damage_attack(vec![0.25, 0.5, 0.25], vec![0, 50, 100])
        }
        AttackId::A1029PetililBlot => self_heal_attack(10, index),
        AttackId::A1030LilligantLeafSupply => energy_bench_attack(0, 1, EnergyType::Grass),
        AttackId::A1031Skiddo => probabilistic_damage_attack(vec![0.5, 0.5], vec![0, 40]),
        AttackId::A1033CharmanderEmber => energy_discard_attack(0, vec![EnergyType::Fire]),
        AttackId::A1035CharizardFireSpin => {
            energy_discard_attack(0, vec![EnergyType::Fire, EnergyType::Fire])
        }
        AttackId::A1036CharizardExCrimsonStorm => {
            energy_discard_attack(1, vec![EnergyType::Fire, EnergyType::Fire])
        }
        AttackId::A1038NinetalesFlamethrower => energy_discard_attack(0, vec![EnergyType::Fire]),
        AttackId::A1040ArcanineHeatTackle => self_damage_attack(100, 20),
        AttackId::A1041ArcanineExInfernoOnrush => self_damage_attack(120, 20),
        AttackId::A1045FlareonFlamethrower => energy_discard_attack(0, vec![EnergyType::Fire]),
        AttackId::A1046MoltresSkyAttack => {
            probabilistic_damage_attack(vec![0.5, 0.5], vec![0, 130])
        }
        AttackId::A1047MoltresExInfernoDance => moltres_inferno_dance(),
        AttackId::A1052CentiskorchFireBlast => energy_discard_attack(0, vec![EnergyType::Fire]),
        AttackId::A1055BlastoiseHydroPump => hydro_pump_attack(acting_player, state, 80),
        AttackId::A1056BlastoiseExHydroBazooka => hydro_pump_attack(acting_player, state, 100),
        AttackId::A1057PsyduckHeadache => damage_and_turn_effect_attack(0, 1),
        AttackId::A1063TentacruelPoisonTentacles => {
            damage_status_attack(50, StatusCondition::Poisoned)
        }
        AttackId::A1096PikachuExCircleCircuit => {
            bench_count_attack(acting_player, state, 0, 30, Some(EnergyType::Lightning))
        }
        AttackId::A1101ElectabuzzThunderPunch => extra_or_self_damage_attack(40, 40, 20),
        AttackId::A1104ZapdosExThunderingHurricane => probabilistic_damage_attack(
            vec![0.0625, 0.25, 0.375, 0.25, 0.0625],
            vec![0, 50, 100, 150, 200],
        ),
        AttackId::A1106ZebstrikaThunderSpear => direct_damage(30, false),
        AttackId::A1128MewtwoPowerBlast => energy_discard_attack(index, vec![EnergyType::Psychic]),
        AttackId::A1129MewtwoExPsydrive => {
            energy_discard_attack(index, vec![EnergyType::Psychic, EnergyType::Psychic])
        }
        AttackId::A1153MarowakExBonemerang => {
            probabilistic_damage_attack(vec![0.25, 0.5, 0.25], vec![0, 80, 160])
        }
        AttackId::A1154HitmonleeStretchKick => direct_damage(30, true),
        AttackId::A1165ArbokCorner => damage_and_turn_effect_attack(index, 1),
        AttackId::A1171NidokingPoisonHorn => damage_status_attack(90, StatusCondition::Poisoned),
        AttackId::A1196MeowthPayDay => draw_and_damage_outcome(10),
        AttackId::A1203KangaskhanDizzyPunch => {
            probabilistic_damage_attack(vec![0.25, 0.5, 0.25], vec![0, 30, 60])
        }
        AttackId::A1a030DedenneThunderShock => {
            damage_chance_status_attack(10, 0.5, StatusCondition::Paralyzed)
        }
        AttackId::A2119DialgaExMetallicTurbo => {
            energy_bench_attack(acting_player, 2, EnergyType::Metal)
        }
        AttackId::A2a071ArceusExUltimateForce => {
            bench_count_attack(acting_player, state, 70, 20, None)
        }
    }
}

fn moltres_inferno_dance() -> (Probabilities, Mutations) {
    let probabilities = vec![0.125, 0.375, 0.375, 0.125]; // 0,1,2,3 heads
    let mutations = probabilities
        .iter()
        .enumerate()
        .map(|(heads, _)| {
            damage_effect_mutation(0, move |_, state, action| {
                if heads == 0 {
                    return;
                }

                // First collect all eligible fire pokemon in bench
                let mut fire_bench_idx = Vec::new();
                for (in_play_idx, pokemon) in state.enumerate_bench_pokemon(action.actor) {
                    if pokemon.get_energy_type() == Some(EnergyType::Fire) {
                        fire_bench_idx.push(in_play_idx);
                    }
                }

                if fire_bench_idx.is_empty() {
                    return;
                }

                let all_choices = generate_energy_distributions(&fire_bench_idx, heads);
                if !all_choices.is_empty() {
                    state
                        .move_generation_stack
                        .push((action.actor, all_choices));
                }
            })
        })
        .collect();
    (probabilities, mutations)
}

fn generate_energy_distributions(fire_bench_idx: &[usize], heads: usize) -> Vec<SimpleAction> {
    let mut all_choices = Vec::new();

    // Generate all possible ways to distribute the energy
    let mut distributions = Vec::new();
    generate_distributions(
        fire_bench_idx,
        heads,
        0,
        &mut vec![0; fire_bench_idx.len()],
        &mut distributions,
    );

    // Convert each distribution into an Attach action
    for dist in distributions {
        let mut attachments = Vec::new();
        for (i, &pokemon_idx) in fire_bench_idx.iter().enumerate() {
            if dist[i] > 0 {
                attachments.push((dist[i] as u32, EnergyType::Fire, pokemon_idx));
            }
        }
        all_choices.push(SimpleAction::Attach {
            attachments,
            is_turn_energy: false,
        });
    }

    all_choices
}

// Helper function to generate all possible distributions of 'heads' energy
// across the available Pokémon
fn generate_distributions(
    fire_bench_idx: &[usize],
    remaining: usize,
    start_idx: usize,
    current: &mut Vec<usize>,
    result: &mut Vec<Vec<usize>>,
) {
    if remaining == 0 {
        result.push(current.clone());
        return;
    }

    if start_idx >= fire_bench_idx.len() {
        return;
    }

    // Try different amounts for the current Pokémon
    for amount in 0..=remaining {
        current[start_idx] = amount;
        generate_distributions(
            fire_bench_idx,
            remaining - amount,
            start_idx + 1,
            current,
            result,
        );
    }
    current[start_idx] = 0;
}

/// Deal damage and attach energy to a pokemon of choice in the bench.
fn energy_bench_attack(
    attack_index: usize,
    amount: u32,
    energy: EnergyType,
) -> (Probabilities, Mutations) {
    index_damage_doutcome(attack_index, move |_, state, action| {
        let mut choices = Vec::new();
        for (in_play_idx, _) in state.enumerate_bench_pokemon(action.actor) {
            choices.push(SimpleAction::Attach {
                attachments: vec![(amount, energy, in_play_idx)],
                is_turn_energy: false,
            });
        }
        if choices.is_empty() {
            return; // do nothing, since we use common_attack_mutation, turn should end, and no damage applied.
        }
        state.move_generation_stack.push((action.actor, choices));
    })
}

/// Used for attacks that on heads deal extra damage, on tails deal self damage.
fn extra_or_self_damage_attack(
    base_damage: u32,
    extra_damage: u32,
    self_damage: u32,
) -> (Probabilities, Mutations) {
    let probabilities = vec![0.5, 0.5];
    let mutations: Mutations = vec![
        damage_mutation(base_damage + extra_damage),
        damage_effect_mutation(base_damage, move |_, state, action| {
            let active = state.get_active_mut(action.actor);
            active.apply_damage(self_damage);
        }),
    ];
    (probabilities, mutations)
}

fn damage_chance_status_attack(
    damage: u32,
    probability_of_status: f64,
    status: StatusCondition,
) -> (Probabilities, Mutations) {
    let probabilities = vec![probability_of_status, 1.0 - probability_of_status];
    let mutations: Mutations = vec![
        damage_effect_mutation(damage, build_status_effect(status)),
        damage_mutation(damage),
    ];
    (probabilities, mutations)
}

/// Used for attacks that do damage for each pokemon (optionally of a type) in your bench.
///  e.g. "Pikachu Ex Circle Circuit".
fn bench_count_attack(
    acting_player: usize,
    state: &State,
    base_damage: u32,
    damage_per: u32,
    energy: Option<EnergyType>,
) -> (Probabilities, Mutations) {
    let mut bench_count = 0;
    for (_, pokemon) in state.enumerate_bench_pokemon(acting_player) {
        if let Some(energy) = energy {
            if pokemon.get_energy_type() == Some(energy) {
                bench_count += 1;
            }
        } else {
            bench_count += 1;
        }
    }
    damage_doutcome(base_damage + damage_per * bench_count)
}

/// Used for attacks that can go directly to bench.
/// It will queue (via move_generation_stack) for the user to choose a pokemon to damage.
fn direct_damage(damage: u32, bench_only: bool) -> (Probabilities, Mutations) {
    damage_effect_doutcome(0, move |_, state, action| {
        let opponent = (action.actor + 1) % 2;
        let mut choices = Vec::new();
        if bench_only {
            for (in_play_idx, _) in state.enumerate_bench_pokemon(opponent) {
                choices.push(SimpleAction::ApplyDamage {
                    in_play_idx,
                    damage,
                });
            }
        } else {
            for (in_play_idx, _) in state.enumerate_in_play_pokemon(opponent) {
                choices.push(SimpleAction::ApplyDamage {
                    in_play_idx,
                    damage,
                });
            }
        }
        if choices.is_empty() {
            return; // do nothing, since we use common_attack_mutation, turn should end, and no damage applied.
        }
        state.move_generation_stack.push((action.actor, choices));
    })
}

fn energy_discard_attack(
    attack_index: usize,
    to_discard: Vec<EnergyType>,
) -> (Probabilities, Mutations) {
    index_damage_doutcome(attack_index, move |_, state, action| {
        let active = state.get_active_mut(action.actor);
        for energy in to_discard.iter() {
            active.discard_energy(energy);
        }
    })
}

/// For attacks that deal damage to opponent and also damage themselves
fn self_damage_attack(damage: u32, self_damage: u32) -> (Probabilities, Mutations) {
    damage_effect_doutcome(damage, move |_, state, action| {
        let active = state.get_active_mut(action.actor);
        active.apply_damage(self_damage);
    })
}

/// For attacks that deal damage and apply a status effect (e.g. Wigglituff Ex)
fn damage_status_attack(damage: u32, status: StatusCondition) -> (Probabilities, Mutations) {
    damage_effect_doutcome(damage, build_status_effect(status))
}

/// For cards like "Meowth Pay Day" that draw a card and deal damage.
fn draw_and_damage_outcome(damage: u32) -> (Probabilities, Mutations) {
    damage_effect_doutcome(damage, move |_, state, action| {
        state
            .move_generation_stack
            .push((action.actor, vec![SimpleAction::DrawCard]));
    })
}

// If this Pokemon has at least 2 extra Water Energy attached, this attack does 60 more damage.
fn hydro_pump_attack(
    acting_player: usize,
    state: &State,
    base_damage: u32,
) -> (Probabilities, Mutations) {
    let blastoise = state.in_play_pokemon[acting_player][0]
        .as_ref()
        .expect("Active Pokemon should be there if attacking");
    // has 2 extra, if at least 5 energies, 4 of which are water
    let has_2_extra = blastoise.attached_energy.len() >= 5
        && blastoise
            .attached_energy
            .iter()
            .filter(|&energy| *energy == EnergyType::Water)
            .count()
            >= 4;
    if has_2_extra {
        damage_doutcome(base_damage + 60)
    } else {
        damage_doutcome(base_damage)
    }
}

/// For attacks that given coin flips, deal different damage.
fn probabilistic_damage_attack(
    probabilities: Vec<f64>,
    damages: Vec<u32>,
) -> (Probabilities, Mutations) {
    let mutations = damages
        .into_iter()
        .map(|damage| damage_mutation(damage))
        .collect();
    (probabilities, mutations)
}

fn self_heal_attack(heal: u32, index: usize) -> (Probabilities, Mutations) {
    index_damage_doutcome(index, move |_, state, action| {
        let active = state.get_active_mut(action.actor);
        active.heal(heal);
    })
}

fn damage_and_turn_effect_attack(index: usize, effect_duration: u8) -> (Probabilities, Mutations) {
    index_damage_doutcome(index, move |_, state, action| {
        let active = state.get_active(action.actor);
        // TODO: Maybe create an EffectId enum and have a mapping between card,attack_idx to effect?
        state.add_turn_effect(active.card.clone(), effect_duration);
    })
}

// ===== These functions should share the common code of
// forcing the end of the turn, applying damage with calculations, forcing enemy
// to promote pokemon after knockout, etc... apply to all attacks.
// Doutcome means deterministic outcome.
fn damage_doutcome(damage: u32) -> (Probabilities, Mutations) {
    damage_effect_doutcome(damage, |_, _, _| {})
}

// TODO: Ask for state so that we can get damage via index, before the mutation,
//  and reuse the common mutation code.
fn index_damage_doutcome<F>(attack_index: usize, additional_effect: F) -> (Probabilities, Mutations)
where
    F: Fn(&mut StdRng, &mut State, &Action) + 'static,
{
    (
        vec![1.0],
        vec![Box::new(move |rng, state, action| {
            apply_common_mutation(state, action);
            state
                .move_generation_stack
                .push((action.actor, vec![SimpleAction::EndTurn]));
            additional_effect(rng, state, action);

            let damage = get_damage_from_attack(state, action.actor, attack_index, 0);
            handle_attack_damage(state, action.actor, damage, 0);
        })],
    )
}

fn damage_effect_doutcome<F>(damage: u32, additional_effect: F) -> (Probabilities, Mutations)
where
    F: Fn(&mut StdRng, &mut State, &Action) + 'static,
{
    (
        vec![1.0],
        vec![damage_effect_mutation(damage, additional_effect)],
    )
}

// ===== Helper functions for building outcomes
fn damage_mutation(damage: u32) -> Mutation {
    damage_effect_mutation(damage, |_, _, _| {})
}

// TODO: Verify all use active
fn damage_effect_mutation<F>(damage: u32, additional_effect: F) -> Mutation
where
    F: Fn(&mut StdRng, &mut State, &Action) + 'static,
{
    Box::new({
        move |rng, state, action| {
            apply_common_mutation(state, action);
            state
                .move_generation_stack
                .push((action.actor, vec![SimpleAction::EndTurn]));
            additional_effect(rng, state, action);
            handle_attack_damage(state, action.actor, damage, 0);
        }
    })
}

fn build_status_effect(status: StatusCondition) -> FnMutation {
    Box::new({
        move |_, state: &mut State, action: &Action| {
            let opponent = (action.actor + 1) % 2;
            let opponent_active = state.get_active_mut(opponent);

            // Arceus Ex avoids status effects
            let string_id = opponent_active.get_id();
            let arceus_ids = [
                CardId::A2a071ArceusEx,
                CardId::A2a086ArceusEx,
                CardId::A2a095ArceusEx,
                CardId::A2a096ArceusEx,
            ];
            let card_id = CardId::from_card_id(&string_id).unwrap();
            if arceus_ids.contains(&card_id) {
                trace!("Arceus Ex avoids status effect");
                return;
            }

            match status {
                StatusCondition::Asleep => opponent_active.asleep = true,
                StatusCondition::Paralyzed => opponent_active.paralyzed = true,
                StatusCondition::Poisoned => opponent_active.poisoned = true,
            }
        }
    })
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;

    use crate::{card_ids::CardId, database::get_card_by_enum, hooks::to_playable_card};

    use super::*;

    #[test]
    fn test_build_status_effect() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut state = State::default();
        let action = Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        };
        let bulbasuar = get_card_by_enum(CardId::A1001Bulbasaur);
        state.in_play_pokemon[1][0] = Some(to_playable_card(&bulbasuar, false));
        let effect = build_status_effect(StatusCondition::Asleep);
        effect(&mut rng, &mut state, &action);
        assert!(state.get_active(1).asleep);
    }

    #[test]
    fn test_arceus_avoids_status() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut state = State::default();
        let action = Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        };
        let arceus = get_card_by_enum(CardId::A2a071ArceusEx);
        state.in_play_pokemon[1][0] = Some(to_playable_card(&arceus, false));
        let effect = build_status_effect(StatusCondition::Asleep);
        effect(&mut rng, &mut state, &action);
        assert!(!state.get_active(1).asleep);
    }

    #[test]
    fn test_arceus_does_90_damage() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut state = State::default();
        let action = Action {
            actor: 0,
            action: SimpleAction::Attack(0),
            is_stack: false,
        };

        let receiver = get_card_by_enum(CardId::A1003Venusaur); // 160 hp
        state.in_play_pokemon[1][0] = Some(to_playable_card(&receiver, false));
        let attacker = get_card_by_enum(CardId::A2a071ArceusEx);
        state.in_play_pokemon[0][0] = Some(to_playable_card(&attacker, false));
        let some_base_pokemon = get_card_by_enum(CardId::A1001Bulbasaur);
        state.in_play_pokemon[0][1] = Some(to_playable_card(&some_base_pokemon, false));

        let (_, mut lazy_mutations) = bench_count_attack(0, &state, 70, 20, None);
        lazy_mutations.remove(0)(&mut rng, &mut state, &action);

        assert_eq!(state.get_active(1).remaining_hp, 70);
    }

    #[test]
    fn test_generate_energy_distributions() {
        // 1 pokemon, 1 head
        let fire_pokemon = vec![1];
        let choices = generate_energy_distributions(&fire_pokemon, 1);
        assert_eq!(choices.len(), 1);
        if let SimpleAction::Attach { attachments, .. } = &choices[0] {
            assert_eq!(attachments, &[(1, EnergyType::Fire, 1)]);
        } else {
            panic!("Expected SimpleAction::Attach");
        }

        // 1 pokemon, 2 heads
        let choices = generate_energy_distributions(&fire_pokemon, 2);
        assert_eq!(choices.len(), 1);
        if let SimpleAction::Attach { attachments, .. } = &choices[0] {
            assert_eq!(attachments, &[(2, EnergyType::Fire, 1)]);
        } else {
            panic!("Expected SimpleAction::Attach");
        }

        // 2 pokemon, 2 heads
        let fire_pokemon = vec![1, 2];
        let choices = generate_energy_distributions(&fire_pokemon, 2);
        assert_eq!(choices.len(), 3);
        let expected_distributions = [
            vec![(2, EnergyType::Fire, 2)],
            vec![(1, EnergyType::Fire, 1), (1, EnergyType::Fire, 2)],
            vec![(2, EnergyType::Fire, 1)],
        ];
        for (i, choice) in choices.iter().enumerate() {
            if let SimpleAction::Attach { attachments, .. } = choice {
                assert_eq!(attachments, &expected_distributions[i]);
            } else {
                panic!("Expected SimpleAction::Attach");
            }
        }

        // 2 pokemon, 3 heads
        let choices = generate_energy_distributions(&fire_pokemon, 3);
        assert_eq!(choices.len(), 4);
        let expected_distributions = [
            vec![(3, EnergyType::Fire, 2)],
            vec![(1, EnergyType::Fire, 1), (2, EnergyType::Fire, 2)],
            vec![(2, EnergyType::Fire, 1), (1, EnergyType::Fire, 2)],
            vec![(3, EnergyType::Fire, 1)],
        ];
        for (i, choice) in choices.iter().enumerate() {
            if let SimpleAction::Attach { attachments, .. } = choice {
                assert_eq!(attachments, &expected_distributions[i]);
            } else {
                panic!("Expected SimpleAction::Attach");
            }
        }

        // 3 pokemon, 2 heads
        let fire_pokemon = vec![1, 2, 3];
        let choices = generate_energy_distributions(&fire_pokemon, 2);
        assert_eq!(choices.len(), 6);
        let expected_distributions = [
            vec![(2, EnergyType::Fire, 3)],
            vec![(1, EnergyType::Fire, 2), (1, EnergyType::Fire, 3)],
            vec![(2, EnergyType::Fire, 2)],
            vec![(1, EnergyType::Fire, 1), (1, EnergyType::Fire, 3)],
            vec![(1, EnergyType::Fire, 1), (1, EnergyType::Fire, 2)],
            vec![(2, EnergyType::Fire, 1)],
        ];
        for (i, choice) in choices.iter().enumerate() {
            if let SimpleAction::Attach { attachments, .. } = choice {
                assert_eq!(attachments, &expected_distributions[i]);
            } else {
                panic!("Expected SimpleAction::Attach");
            }
        }
    }
}
