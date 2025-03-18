/// These are the places/functions in the framework that custom logic is to be implemented per card.
/// That is those special "if Psyduck, do this", "if Darkrai, do that" kind of logic.
/// We call these "hooks" (like on_attach_tool, on_attach_energy, on_play, on_knockout, etc...).
use core::panic;
use std::vec;

use log::debug;

use crate::{
    card_ids::CardId,
    database::get_card_by_enum,
    tool_ids::ToolId,
    types::{Card, EnergyType, PlayedCard},
    State,
};

const PLAYABLE_TRAINER_CARD_NAMES: [&str; 3] = ["Helix Fossil", "Dome Fossil", "Old Amber"];

pub(crate) fn to_playable_card(card: &crate::types::Card, played_this_turn: bool) -> PlayedCard {
    let total_hp = match card {
        Card::Pokemon(pokemon_card) => pokemon_card.hp,
        Card::Trainer(trainer_card) => {
            if PLAYABLE_TRAINER_CARD_NAMES.contains(&trainer_card.name.as_str()) {
                40
            } else {
                panic!("Unplayable Trainer Card: {:?}", trainer_card);
            }
        }
    };
    PlayedCard {
        card: card.clone(),
        remaining_hp: total_hp,
        total_hp,
        attached_energy: vec![],
        attached_tool: None,
        played_this_turn,
        ability_used: false,
        poisoned: false,
        paralyzed: false,
        asleep: false,
        cards_behind: vec![],
    }
}

pub(crate) fn on_attach_tool(state: &mut State, actor: usize, in_play_idx: usize, tool_id: ToolId) {
    match tool_id {
        ToolId::A2147GiantCape => {
            // Add +20 to remaining_hp and total_hp
            let card = state.in_play_pokemon[actor][in_play_idx]
                .as_mut()
                .expect("Active Pokemon should be there");
            card.remaining_hp += 20;
            card.total_hp += 20;
        }
    }
}

// TODO: Implement Psyduck's attack and Gengars ability that disallow playing support cards.
pub(crate) fn can_play_support(state: &State) -> bool {
    !state.has_played_support
}

pub(crate) fn can_retreat(state: &State) -> bool {
    let no_arbok_corner = !state
        .turn_effects
        .iter()
        .any(|x| matches!(x, Card::Pokemon(pokemon_card) if pokemon_card.name == "Arbok"));
    !state.has_retreated && no_arbok_corner
}

pub(crate) fn get_retreat_cost(state: &State, card: &PlayedCard) -> Vec<EnergyType> {
    if let Card::Pokemon(pokemon_card) = &card.card {
        let mut normal_cost = pokemon_card.retreat_cost.clone();
        // Implement Retreat Cost Modifiers here
        let x_speed = state
            .turn_effects
            .iter()
            .filter(|x| **x == get_card_by_enum(CardId::PA002XSpeed))
            .count();
        let leafs = state
            .turn_effects
            .iter()
            .filter(|x| {
                **x == get_card_by_enum(CardId::A1a068Leaf)
                    || **x == get_card_by_enum(CardId::A1a082Leaf)
            })
            .count();
        // Retreat Effects accumulate so we add them.
        let to_subtract = leafs * 2 + x_speed;
        for _ in 0..to_subtract {
            normal_cost.pop(); // Remove one colorless energy from retreat cost
        }
        normal_cost
    } else {
        vec![]
    }
}

pub(crate) fn get_damage_from_attack(
    state: &State,
    player: usize,
    index: usize,
    receiving_index: usize,
) -> u32 {
    let active = state.get_active(player);
    let attack = active.card.get_attacks()[index].clone();

    // If attack is 0, not even Giovanni takes it to 10.
    if attack.fixed_damage == 0 {
        return attack.fixed_damage;
    }
    // If its bench attack, don't apply multipliers
    if receiving_index != 0 {
        return attack.fixed_damage;
    }

    // Giovanni's Modifier
    let mut giovanni_modifier = 0;
    if state.turn_effects.iter().any(|x| {
        matches!(x, Card::Trainer(trainer_card) if CardId::from_numeric_id(trainer_card.numeric_id) == Some(CardId::A1223Giovanni))
    }) {
        giovanni_modifier = 10;
    }

    // Weakness Modifier
    let opponent = (player + 1) % 2;
    let mut weakness_modifier = 0;
    let receiving = state.get_active(opponent);
    if let Card::Pokemon(pokemon_card) = &receiving.card {
        if pokemon_card.weakness == active.card.get_type() {
            debug!(
                "Weakness! {:?} is weak to {:?}",
                pokemon_card,
                active.card.get_type()
            );
            weakness_modifier = 20;
        }
    }

    attack.fixed_damage + weakness_modifier + giovanni_modifier
}

// Check if attached satisfies cost (considering Colorless)
pub(crate) fn contains_energy(attached: &[EnergyType], cost: &[EnergyType]) -> bool {
    // First try to match the non-colorless energy
    let non_colorless_cost = cost.iter().filter(|x| **x != EnergyType::Colorless);
    let colorless_cost = cost.iter().filter(|x| **x == EnergyType::Colorless);

    let mut attached_copy: Vec<EnergyType> = attached.to_vec();
    for energy in non_colorless_cost {
        let index = attached_copy.iter().position(|x| *x == *energy);
        if let Some(i) = index {
            attached_copy.remove(i);
        } else {
            return false;
        }
    }

    // If all non-colorless energy is satisfied, check if there are enough colorless energy
    attached_copy.len() >= colorless_cost.count()
}

// Test Colorless is wildcard when counting energy
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_energy() {
        let slice_a = vec![EnergyType::Fire, EnergyType::Fire, EnergyType::Fire];
        let slice_b = vec![EnergyType::Colorless, EnergyType::Fire];
        assert!(contains_energy(&slice_a, &slice_b));
    }

    #[test]
    fn test_contains_energy_colorless() {
        let slice_a = vec![EnergyType::Fire, EnergyType::Fire, EnergyType::Water];
        let slice_b = vec![EnergyType::Colorless, EnergyType::Fire, EnergyType::Fire];
        assert!(contains_energy(&slice_a, &slice_b));
    }

    #[test]
    fn test_contains_energy_false_missing() {
        let slice_a = vec![EnergyType::Grass, EnergyType::Grass, EnergyType::Fire];
        let slice_b = vec![EnergyType::Colorless, EnergyType::Fire, EnergyType::Water];
        assert!(!contains_energy(&slice_a, &slice_b));
    }

    #[test]
    fn test_contains_energy_double_colorless() {
        let slice_a = vec![EnergyType::Water, EnergyType::Water, EnergyType::Fire];
        let slice_b = vec![EnergyType::Colorless, EnergyType::Colorless];
        assert!(contains_energy(&slice_a, &slice_b));
    }

    #[test]
    fn test_retreat_costs() {
        let state = State::default();
        let card = get_card_by_enum(CardId::A1055Blastoise);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(
            retreat_cost,
            vec![
                EnergyType::Colorless,
                EnergyType::Colorless,
                EnergyType::Colorless
            ]
        );
    }

    #[test]
    fn test_retreat_costs_with_xspeed() {
        let mut state = State::default();
        state
            .turn_effects
            .push(get_card_by_enum(CardId::PA002XSpeed));
        let card = get_card_by_enum(CardId::A1055Blastoise);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(
            retreat_cost,
            vec![EnergyType::Colorless, EnergyType::Colorless]
        );
    }

    #[test]
    fn test_retreat_costs_with_two_xspeed_and_two_leafs() {
        let mut state = State::default();
        state
            .turn_effects
            .push(get_card_by_enum(CardId::PA002XSpeed));
        state
            .turn_effects
            .push(get_card_by_enum(CardId::PA002XSpeed));
        state
            .turn_effects
            .push(get_card_by_enum(CardId::A1a068Leaf));
        let card = get_card_by_enum(CardId::A1211Snorlax);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(retreat_cost, vec![]);
    }
}
