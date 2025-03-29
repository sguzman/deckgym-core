use log::debug;
use rand::rngs::StdRng;

use crate::{actions::SimpleAction, types::Card, State};

use super::Action;

pub(crate) type Probabilities = Vec<f64>;

// Mutations should be deterministic. They take StdRng because we simplify some states spaces
//  like "shuffling a deck" (which would otherwise yield a huge state space) to a single
//  mutation/state ("shuffled deck"). Bots should not use deck order information when forecasting.
pub(crate) type FnMutation = Box<dyn Fn(&mut StdRng, &mut State, &Action)>;
pub(crate) type Mutation = Box<dyn FnOnce(&mut StdRng, &mut State, &Action)>;
pub(crate) type Mutations = Vec<Mutation>;

/// Advance state to the next turn (i.e. maintain current_player and turn_count)
pub(crate) fn forecast_end_turn(state: &State) -> (Probabilities, Mutations) {
    let in_initial_setup_phase = state.turn_count == 0;
    if in_initial_setup_phase {
        (
            vec![1.0],
            vec![Box::new({
                |_, state, action| {
                    apply_common_mutation(state, action);
                    // advance current_player, but only advance "turn" (i.e. stay in 0) when both players done.
                    state.current_player = (state.current_player + 1) % 2;
                    let both_players_initiated = state.in_play_pokemon[0][0].is_some()
                        && state.in_play_pokemon[1][0].is_some();
                    if both_players_initiated {
                        // Actually start game (no energy generation)
                        state.turn_count = 1;
                        state.reset_turn_states();
                        state.queue_draw_action(state.current_player);
                    }
                }
            })],
        )
    } else {
        forecast_pokemon_checkup(state)
    }
}

/// Handle Status Effects
fn forecast_pokemon_checkup(state: &State) -> (Probabilities, Mutations) {
    let mut sleeps_to_handle = vec![];
    let mut paralyzed_to_handle = vec![];
    let mut poisons_to_handle = vec![];
    for player in 0..2 {
        for (i, pokemon) in state.enumerate_in_play_pokemon(player) {
            if pokemon.asleep {
                sleeps_to_handle.push((player, i));
            }
            if pokemon.paralyzed {
                paralyzed_to_handle.push((player, i));
            }
            if pokemon.poisoned {
                poisons_to_handle.push((player, i));
                debug!("{}'s Pokemon {} is poisoned", player, i);
            }
        }
    }

    // Get all binary vectors representing the possible outcomes.
    // These are the "outcome_ids" (e.g. outcome [true, false] might represent
    // waking up one pokemon and not another).
    let outcome_ids = generate_boolean_vectors(sleeps_to_handle.len());
    let probabilities = vec![1.0 / outcome_ids.len() as f64; outcome_ids.len()];
    let mut outcomes: Mutations = vec![];
    for outcome in outcome_ids {
        let sleeps_to_handle = sleeps_to_handle.clone();
        let paralyzed_to_handle = paralyzed_to_handle.clone();
        let poisons_to_handle = poisons_to_handle.clone();
        outcomes.push(Box::new({
            |_, state, action| {
                apply_common_mutation(state, action);
                apply_pokemon_checkup(
                    state,
                    sleeps_to_handle,
                    paralyzed_to_handle,
                    poisons_to_handle,
                    outcome,
                );
            }
        }));
    }
    (probabilities, outcomes)
}

fn apply_pokemon_checkup(
    mutated_state: &mut State,
    sleeps_to_handle: Vec<(usize, usize)>,
    paralyzed_to_handle: Vec<(usize, usize)>,
    poisons_to_handle: Vec<(usize, usize)>,
    outcome: Vec<bool>,
) {
    for (i, is_awake) in sleeps_to_handle.iter().zip(outcome) {
        if is_awake {
            let (player, in_play_idx) = i;
            let pokemon = mutated_state.in_play_pokemon[*player][*in_play_idx]
                .as_mut()
                .expect("Pokemon should be there...");
            pokemon.asleep = false;
            debug!("{}'s Pokemon {} woke up", player, in_play_idx);
        }
    }
    // These always happen regardless of outcome_binary_vector
    for (player, in_play_idx) in paralyzed_to_handle {
        let pokemon = mutated_state.in_play_pokemon[player][in_play_idx]
            .as_mut()
            .expect("Pokemon should be there...");
        pokemon.paralyzed = false;
        debug!("{}'s Pokemon {} is un-paralyzed", player, in_play_idx);
    }
    for (player, in_play_idx) in poisons_to_handle {
        let opponent = (player + 1) % 2;
        handle_attack_damage(mutated_state, opponent, &vec![(10, in_play_idx)]);
    }
    // Advance turn
    mutated_state.advance_turn();
}

fn generate_boolean_vectors(n: usize) -> Vec<Vec<bool>> {
    // The total number of combinations is 2^n
    let total_combinations = 1 << n; // 2^n

    // Generate all combinations
    (0..total_combinations)
        .map(|i| {
            // Convert the number `i` to its binary representation as a vector of booleans
            (0..n).map(|bit| (i & (1 << bit)) != 0).collect()
        })
        .collect()
}

pub(crate) fn handle_attack_damage(
    state: &mut State,
    attacking_player: usize,
    targets: &Vec<(u32, usize)>, // damage, in_play_idx
) {
    let opponent = (attacking_player + 1) % 2;
    let mut knockouts: Vec<usize> = vec![];
    for (damage, receiving_pokemon_idx) in targets {
        if *damage == 0 {
            continue;
        }

        // Apply damage to opponent's pokemon (without surpassing 0 HP)
        let receiving_pokemon = state.in_play_pokemon[opponent][*receiving_pokemon_idx]
            .as_mut()
            .expect("Pokemon should be there if receiving damage");
        receiving_pokemon.apply_damage(*damage);
        debug!(
            "Dealt {} damage to opponent's {} Pokemon. Remaining HP: {}",
            damage, receiving_pokemon_idx, receiving_pokemon.remaining_hp
        );

        // TODO: If pokemon is active and has counterattack mechanisms, apply.
        if receiving_pokemon.remaining_hp > 0 {
            continue; // nothing to do
        }
        knockouts.push(*receiving_pokemon_idx);
    }

    // Handle knockouts: Discard cards and award points (to potentially short-circuit promotions)
    // TODO: Could be a counter-attack knockout too.
    for receiving_pokemon_idx in knockouts.clone() {
        let receiving_pokemon = state.in_play_pokemon[opponent][receiving_pokemon_idx]
            .as_mut()
            .expect("Pokemon should be there if knocked out");

        // Sum points
        let points_won = if receiving_pokemon.card.is_ex() { 2 } else { 1 };
        state.points[attacking_player] += points_won;
        debug!(
            "Opponent's Pokemon {} fainted. Won {} points for a total of {}",
            receiving_pokemon_idx, points_won, state.points[attacking_player]
        );

        // Move card (and evolution chain) into discard pile
        let mut cards_to_discard = receiving_pokemon.cards_behind.clone();
        cards_to_discard.push(receiving_pokemon.card.clone());
        debug!("Discarding: {:?}", cards_to_discard);
        state.discard_piles[opponent].extend(cards_to_discard);
        state.in_play_pokemon[opponent][receiving_pokemon_idx] = None;
    }

    // TODO: Could be tie because of counterattack knockouts
    if state.points[attacking_player] >= 3 {
        state.winner = Some(attacking_player); // the next ticker should end the game
        return;
    }

    // Queue up promotion actions if the game is still on after a knockout
    // TODO: Attacker might also need to promote.
    if knockouts.contains(&0) {
        // If K.O. was Active and opponent hasn't win, check if can select from Bench
        let enumerated_bench_pokemon = state.enumerate_bench_pokemon(opponent).collect::<Vec<_>>();
        if enumerated_bench_pokemon.is_empty() {
            // If no bench pokemon, opponent loses
            state.winner = Some(attacking_player);
            debug!("Opponent lost due to no bench pokemon");
        } else if enumerated_bench_pokemon.len() == 1 {
            // If only one bench pokemon, automatically switch to it
            let bench_idx = enumerated_bench_pokemon[0].0;
            let bench_card = enumerated_bench_pokemon[0].1.clone();
            debug!("Automatically switching to Active: {:?}", bench_card);
            state.in_play_pokemon[opponent][0] = Some(bench_card);
            state.in_play_pokemon[opponent][bench_idx] = None;
        } else {
            // If multiple bench pokemon, let opponent choose
            let possible_moves = state
                .enumerate_bench_pokemon(opponent)
                .map(|(i, _)| SimpleAction::Activate { in_play_idx: i })
                .collect::<Vec<_>>();
            // Activate will keep control, end turn should happen after since stack
            debug!(
                "Triggering Activate moves: {:?} to player {}",
                possible_moves, opponent
            );
            state.move_generation_stack.push((opponent, possible_moves));
        }
    }
}

// Apply common mutations for all outcomes
// TODO: Is there a way outcome implementations don't have to remember to call this?
pub(crate) fn apply_common_mutation(state: &mut State, action: &Action) {
    if action.is_stack {
        state.move_generation_stack.pop();
    }
    if let SimpleAction::Play { trainer_card } = &action.action {
        let card = Card::Trainer(trainer_card.clone());
        state.discard_card_from_hand(action.actor, &card);
        if card.is_support() {
            state.has_played_support = true;
        }
    }
}
