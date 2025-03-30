use clap::Parser;
use colored::Colorize;
use deckgym::players::{create_players, fill_code_array, parse_player_code, PlayerCode};
use deckgym::state::GameOutcome;
use deckgym::Game;
use env_logger::{Builder, Env};
use log::{info, warn};
use num_format::{Locale, ToFormattedString};
use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the first deck file
    deck_a: String,

    /// Path to the second deck file
    deck_b: String,

    /// Players' strategies as a comma-separated list
    #[clap(long, value_delimiter = ',', value_parser(parse_player_code))]
    players: Option<Vec<PlayerCode>>,

    /// Number of simulations to run
    #[arg(short, long)]
    num: u32,

    /// Seed for random number generation
    #[arg(short, long)]
    seed: Option<u64>,
}

/// The CLI tool to simulate games between two decks.
fn main() {
    // Initialize env_logger with a custom format
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{}",
                record.args() // Log message
            )
        })
        .init();
    let args = Args::parse();

    warn!("Welcome to {}!", "deckgym".blue().bold());

    // Read the decks files and initialize Players
    let deck_a = deckgym::Deck::from_file(&args.deck_a).expect("Failed to parse deck from file");
    let deck_b = deckgym::Deck::from_file(&args.deck_b).expect("Failed to parse deck from file");
    let cli_players = fill_code_array(args.players);

    // Simulate Games and accumulate statistics
    warn!(
        "Running {} games with players {:?}",
        args.num.to_formatted_string(&Locale::en),
        cli_players
    );
    let start = Instant::now(); // Start the timer
    let num_simulations = args.num;
    let mut wins_per_deck = [0, 0, 0];
    let mut turns_per_game = Vec::new();
    let mut plys_per_game = Vec::new();
    let mut total_degrees = Vec::new();
    for i in 1..=num_simulations {
        let players = create_players(deck_a.clone(), deck_b.clone(), cli_players.clone());
        let seed = args.seed.unwrap_or(rand::random::<u64>());
        let mut game = Game::new(players, seed);
        let outcome = game.play();
        turns_per_game.push(game.get_state_clone().turn_count);
        plys_per_game.push(game.get_num_plys());
        total_degrees.extend(game.get_degrees_per_ply().iter());
        info!("Simulation {}: Winner is {:?}", i, outcome);
        match outcome {
            Some(GameOutcome::Win(winner_name)) => {
                wins_per_deck[winner_name] += 1;
            }
            Some(GameOutcome::Tie) | None => {
                wins_per_deck[2] += 1;
            }
        }
    }
    let duration = start.elapsed(); // Measure elapsed time
    let avg_time_per_game = duration.as_secs_f64() / num_simulations as f64;
    let avg_duration = Duration::from_secs_f64(avg_time_per_game);

    // Print statistics
    warn!(
        "Ran {} simulations in {} ({} per game)!",
        num_simulations.to_formatted_string(&Locale::en),
        humantime::format_duration(duration).to_string(),
        humantime::format_duration(avg_duration).to_string()
    );
    warn!(
        "Average number of turns per game: {:.2}",
        turns_per_game
            .iter()
            .map(|&turns| turns as u32)
            .sum::<u32>() as f32
            / num_simulations as f32
    );
    warn!(
        "Average number of plys per game: {:.2}",
        plys_per_game.iter().sum::<u32>() as f32 / num_simulations as f32
    );
    warn!(
        "Average number of degrees per ply: {:.2}",
        total_degrees.iter().sum::<u32>() as f32 / total_degrees.len() as f32
    );
    warn!(
        "Player {:?} with Deck {} wins: {} ({:.2}%)",
        cli_players[0],
        args.deck_a,
        wins_per_deck[0].to_formatted_string(&Locale::en),
        wins_per_deck[0] as f32 / num_simulations as f32 * 100.0
    );
    warn!(
        "Player {:?} with Deck {} wins: {} ({:.2}%)",
        cli_players[1],
        args.deck_b,
        wins_per_deck[1].to_formatted_string(&Locale::en),
        wins_per_deck[1] as f32 / num_simulations as f32 * 100.0
    );
    warn!(
        "Draws: {} ({:.2}%)",
        wins_per_deck[2].to_formatted_string(&Locale::en),
        wins_per_deck[2] as f32 / num_simulations as f32 * 100.0
    );
}
