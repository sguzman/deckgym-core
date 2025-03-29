use log::{debug, trace, LevelFilter};
use rand::rngs::StdRng;
use std::fmt::Debug;

use crate::actions::{forecast_action, Action};
use crate::{generate_possible_actions, Deck, State};

use super::Player;

pub struct ExpectiMiniMaxPlayer {
    pub deck: Deck,
    pub max_depth: usize, // max_depth = 1 it should be value function player
}

impl Player for ExpectiMiniMaxPlayer {
    fn decision_fn(
        &mut self,
        rng: &mut StdRng,
        state: &State,
        possible_actions: Vec<Action>,
    ) -> Action {
        let original_level = log::max_level();
        log::set_max_level(LevelFilter::Error); // Temporarily silence debug and trace logs
        let myself = possible_actions[0].actor;
        // Get value for each possible action
        let scores: Vec<f64> = possible_actions
            .iter()
            .map(|action| expected_value_function(rng, state, action, self.max_depth - 1, myself))
            .collect();

        // Select the one with best score
        let best_idx = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;
        log::set_max_level(original_level); // Restore the original logging level
        possible_actions[best_idx].clone()
    }

    fn get_deck(&self) -> Deck {
        self.deck.clone()
    }
}

fn expected_value_function(
    rng: &mut StdRng,
    state: &State,
    action: &Action,
    depth: usize,
    myself: usize,
) -> f64 {
    trace!("E({}) depth left: {} action: {:?}", myself, depth, action);
    let (probabilities, mutations) = forecast_action(state, action);
    let mut outcomes: Vec<State> = vec![];
    for mutation in mutations {
        let mut state = state.clone();
        mutation(rng, &mut state, action);
        outcomes.push(state);
    }
    outcomes
        .iter()
        .zip(probabilities.iter())
        .map(|(outcome, prob)| expectiminimax(rng, outcome, depth, myself) * prob)
        .sum()
}

fn expectiminimax(rng: &mut StdRng, state: &State, depth: usize, myself: usize) -> f64 {
    if state.is_game_over() || depth == 0 {
        return value_function(state, myself);
    }

    let (actor, actions) = generate_possible_actions(state);
    if actor == myself {
        // We are in maximing mode.
        let scores: Vec<f64> = actions
            .iter()
            .map(|action| expected_value_function(rng, state, action, depth - 1, myself))
            .collect();
        scores.iter().cloned().fold(0.0, f64::max)
    } else {
        // TODO: If minimizing, we can't just generate_possible_actions since
        //  not everything is public information. So we would have to have
        //  our own version of it that only returns the actions that are
        let scores: Vec<f64> = actions
            .iter()
            .map(|action| expected_value_function(rng, state, action, depth - 1, myself))
            .collect();
        scores.iter().cloned().fold(0.0, f64::min)
    }
}

fn value_function(state: &State, myself: usize) -> f64 {
    // TODO: Add more features. Other ideas:
    // Attached energy on enemies in play?
    // Can we give priorities to attached energies?
    // Health on the Active spot?
    // Closeness to getting a point(?) Num Knockouts?
    let opponent = (myself + 1) % 2;
    let attached_energy_in_play = state
        .enumerate_in_play_pokemon(myself)
        .map(|(_, card)| card.attached_energy.len() as f64)
        .sum::<f64>();
    let enemy_attached_energy_in_play = state
        .enumerate_in_play_pokemon(opponent)
        .map(|(_, card)| card.attached_energy.len() as f64)
        .sum::<f64>();

    let points = state.points[myself] as f64;
    let opponent_points = state.points[opponent] as f64;

    (points - opponent_points) * 100.0 + attached_energy_in_play - enemy_attached_energy_in_play
}

impl Debug for ExpectiMiniMaxPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExpectiMiniMaxPlayer")
    }
}
