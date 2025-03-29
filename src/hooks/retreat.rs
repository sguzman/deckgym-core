use crate::{
    card_ids::CardId,
    database::get_card_by_enum,
    types::{Card, EnergyType, PlayedCard},
    State,
};

pub(crate) fn can_retreat(state: &State) -> bool {
    let no_arbok_corner = !state
        .get_current_turn_effects()
        .iter()
        .any(|x| matches!(x, Card::Pokemon(pokemon_card) if pokemon_card.name == "Arbok"));
    !state.has_retreated && no_arbok_corner
}

pub(crate) fn get_retreat_cost(state: &State, card: &PlayedCard) -> Vec<EnergyType> {
    if let Card::Pokemon(pokemon_card) = &card.card {
        let mut normal_cost = pokemon_card.retreat_cost.clone();
        // Implement Retreat Cost Modifiers here
        let x_speed = state
            .get_current_turn_effects()
            .iter()
            .filter(|x| **x == get_card_by_enum(CardId::PA002XSpeed))
            .count();
        let leafs = state
            .get_current_turn_effects()
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

// Test Colorless is wildcard when counting energy
#[cfg(test)]
mod tests {
    use crate::hooks::core::to_playable_card;

    use super::*;

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
        state.add_turn_effect(get_card_by_enum(CardId::PA002XSpeed), 0);
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
        state.add_turn_effect(get_card_by_enum(CardId::PA002XSpeed), 0);
        state.add_turn_effect(get_card_by_enum(CardId::PA002XSpeed), 0);
        state.add_turn_effect(get_card_by_enum(CardId::A1a068Leaf), 0);
        let card = get_card_by_enum(CardId::A1211Snorlax);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(retreat_cost, vec![]);
    }
}
