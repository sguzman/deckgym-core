use clap::Parser;
use colored::Colorize;
use deckgym::card_ids::CardId;
use deckgym::players::{create_players, fill_code_array, parse_player_code, PlayerCode};
use deckgym::state::GameOutcome;
use deckgym::{Deck, Game};
use env_logger::{Builder, Env};
use log::warn;
use num_format::{Locale, ToFormattedString};
use rand;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the incomplete deck file (missing up to 4 cards)
    incomplete_deck: String,

    /// Comma-separated list of candidate card IDs for completion
    candidate_cards: String,

    /// Folder containing enemy deck files
    enemy_decks_folder: String,

    /// Number of simulations to run per enemy deck for each combination
    #[arg(short, long)]
    num: u32,

    /// Players' strategies as a comma-separated list (e.g. "e,e")
    #[clap(long, value_delimiter = ',', value_parser(parse_player_code))]
    players: Option<Vec<PlayerCode>>,

    /// Seed for random number generation (optional)
    #[arg(short, long)]
    seed: Option<u64>,
}

fn main() {
    // Initialize the logger with a minimal format.
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    warn!("Welcome to {} optimizer!", "deckgym".blue().bold());
    let args = Args::parse();

    // Parse the candidate cards list.
    let candidate_cards: Vec<CardId> = args
        .candidate_cards
        .split(',')
        .map(|s| {
            // take last 3 to be id, then the rest of prefix will be set
            let s = s.trim();
            if s.len() < 3 {
                panic!("Card ID should be at least 3 characters long");
            }
            let number = &s[s.len() - 3..];
            let prefix = &s[..s.len() - 3];
            let id = format!("{} {}", prefix, number);
            CardId::from_card_id(id.as_str()).expect("Card ID should be valid")
        })
        .collect();

    // Read and validate the incomplete deck.
    let incomplete_deck = deckgym::Deck::from_file(&args.incomplete_deck)
        .expect("Failed to parse incomplete deck file");
    let current_count = incomplete_deck.cards.len();
    let missing_count = 20 - current_count;
    warn!(
        "Incomplete deck has {} cards, missing {} cards",
        current_count, missing_count
    );
    if missing_count == 0 {
        warn!("Deck is already complete (20 cards). No optimization needed.");
        return;
    }

    // For each candidate card, determine how many additional copies are allowed.
    // A card cannot appear more than twice in the deck.
    let mut allowed_map: HashMap<CardId, u32> = HashMap::new();
    for card in &candidate_cards {
        let count = incomplete_deck
            .cards
            .iter()
            .filter(|c| c.get_card_id() == *card)
            .count();
        let allowed = if count >= 2 { 0 } else { 2 - count };
        allowed_map.insert(card.clone(), allowed as u32);
    }

    // Read enemy decks from the specified folder.
    let enemy_deck_paths: Vec<String> = fs::read_dir(&args.enemy_decks_folder)
        .expect("Failed to read enemy decks folder")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().is_file() {
                Some(entry.path().to_str()?.to_string())
            } else {
                None
            }
        })
        .collect();
    let enemy_valid_decks: Vec<Deck> = enemy_deck_paths
        .iter()
        .filter_map(|path| {
            let deck = Deck::from_file(path).ok()?;
            if deck.cards.len() == 20 {
                Some(deck)
            } else {
                warn!("Skipping enemy deck {} since not valid", path);
                None
            }
        })
        .collect();
    warn!(
        "Found {} enemy deck files ({} valid). {:?}",
        enemy_deck_paths.len().to_formatted_string(&Locale::en),
        enemy_valid_decks.len(),
        enemy_deck_paths
            .iter()
            .map(|s| s.split('/').last().unwrap())
            .collect::<Vec<_>>()
    );

    // Generate all valid combinations (multiset selections) of candidate cards that sum to missing_count.
    // let combinations = generate_combinations(&candidate_cards, &allowed_map, missing_count as u32);
    // warn!(
    //     "Generated {} possible combinations to complete the deck.",
    //     combinations.len()
    // );

    // let mut best_win_percent = 0.0;
    // let mut best_combination = None;
    // let mut results = Vec::new();

    // For every valid combination, complete the deck and simulate games.
    // for comb in combinations {
    //     // Create a completed deck by cloning the incomplete one and adding the candidate cards.
    //     let mut completed_deck = incomplete_deck.clone();
    //     for card in &comb {
    //         completed_deck
    //             .add_card(card)
    //             .expect("Failed to add card to deck");
    //     }

    //     // Check that the deck is complete (20 cards).
    //     if completed_deck.cards().len() != 20 {
    //         warn!(
    //             "Completed deck is not 20 cards (has {}), skipping combination {:?}",
    //             completed_deck.cards().len(),
    //             comb
    //         );
    //         continue;
    //     }

    //     let mut total_wins = 0;
    //     let mut total_games = 0;

    //     // Simulate games for each enemy deck.
    //     for enemy_path in &enemy_deck_paths {
    //         let enemy_deck =
    //             deckgym::Deck::from_file(enemy_path.to_str().expect("Invalid enemy deck path"))
    //                 .expect("Failed to parse enemy deck file");

    //         for _ in 0..args.num {
    //             let players = create_players(
    //                 completed_deck.clone(),
    //                 enemy_deck.clone(),
    //                 fill_code_array(args.players.clone()),
    //             );
    //             let seed = args.seed.unwrap_or(rand::random::<u64>());
    //             let mut game = Game::new(players, seed);
    //             let outcome = game.play();

    //             // Assume that if outcome is a win and the first player (our deck) wins, it counts as a win.
    //             if let Some(GameOutcome::Win(winner)) = outcome {
    //                 if winner == 0 {
    //                     total_wins += 1;
    //                 }
    //             }
    //             total_games += 1;
    //         }
    //     }

    //     let win_percent = (total_wins as f32 / total_games as f32) * 100.0;
    //     results.push((comb.clone(), win_percent));
    //     warn!("Combination {:?} win percentage: {:.2}%", comb, win_percent);
    //     if win_percent > best_win_percent {
    //         best_win_percent = win_percent;
    //         best_combination = Some(comb.clone());
    //     }
    // }

    // // Report the best combination found.
    // match best_combination {
    //     Some(comb) => {
    //         warn!(
    //             "Best combination: {:?} with win percentage: {:.2}%",
    //             comb, best_win_percent
    //         );
    //     }
    //     None => {
    //         warn!("No valid combination found.");
    //     }
    // }
}

/// Generates all valid multisets of candidate cards (as vectors of strings) whose total count is `remaining`.
/// Each candidate card cannot be used more than allowed_map[card] times.
fn generate_combinations(
    candidates: &Vec<String>,
    allowed_map: &HashMap<String, u32>,
    remaining: u32,
) -> Vec<Vec<String>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    generate_combinations_recursive(
        candidates,
        allowed_map,
        remaining,
        0,
        &mut current,
        &mut result,
    );
    result
}

/// Helper recursive function to generate combinations.
fn generate_combinations_recursive(
    candidates: &Vec<String>,
    allowed_map: &HashMap<String, u32>,
    remaining: u32,
    index: usize,
    current: &mut Vec<String>,
    result: &mut Vec<Vec<String>>,
) {
    if remaining == 0 {
        result.push(current.clone());
        return;
    }
    if index >= candidates.len() {
        return;
    }
    let candidate = &candidates[index];
    let max_allowed = *allowed_map.get(candidate).unwrap_or(&2);
    // Try using this candidate 0 up to min(max_allowed, remaining) times.
    for count in 0..=std::cmp::min(max_allowed, remaining) {
        for _ in 0..count {
            current.push(candidate.clone());
        }
        generate_combinations_recursive(
            candidates,
            allowed_map,
            remaining - count,
            index + 1,
            current,
            result,
        );
        for _ in 0..count {
            current.pop();
        }
    }
}
