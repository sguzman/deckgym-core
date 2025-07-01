use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use std::fmt::Debug;

use crate::actions::{Action, SimpleAction};
use crate::{Deck, State};

use super::Player;

pub struct WeightedRandomPlayer {
    pub deck: Deck,
}

impl Player for WeightedRandomPlayer {
    fn decision_fn(
        &mut self,
        rng: &mut StdRng,
        _: &State,
        possible_actions: Vec<Action>,
    ) -> Action {
        // Get weights for the possible actions
        let weights: Vec<u32> = possible_actions
            .iter()
            .map(|action| get_weight(&action.action))
            .collect();

        // Create a WeightedIndex based on the weights
        let dist = WeightedIndex::new(&weights).expect("Weights should be non-empty and non-zero");

        // Select a weighted random action
        possible_actions[dist.sample(rng)].clone()
    }

    fn get_deck(&self) -> Deck {
        self.deck.clone()
    }
}

impl Debug for WeightedRandomPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WeightedRandomPlayer")
    }
}

fn get_weight(action: &SimpleAction) -> u32 {
    match action {
        SimpleAction::DrawCard => 1,
        SimpleAction::Play { .. } => 5,
        SimpleAction::Place(_, _) => 5,
        SimpleAction::Attach { .. } => 10,
        SimpleAction::AttachTool { .. } => 10,
        SimpleAction::Evolve(_, _) => 10,
        SimpleAction::UseAbility(_) => 10,
        SimpleAction::Attack(_) => 10,
        SimpleAction::ApplyDamage { .. } => 10,
        SimpleAction::Retreat(_) => 2,
        SimpleAction::EndTurn => 1,
        SimpleAction::Heal { .. } => 5,
        SimpleAction::Activate { .. } => 1,
        SimpleAction::PreventDamage { .. } => 5,
    }
}
