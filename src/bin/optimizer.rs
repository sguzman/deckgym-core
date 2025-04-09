use clap::Parser;
use colored::Colorize;
use deckgym::optimize;
use deckgym::players::{parse_player_code, PlayerCode};
use env_logger::{Builder, Env};
use log::warn;
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

    optimize(
        &args.incomplete_deck,
        &args.candidate_cards,
        &args.enemy_decks_folder,
        args.num,
        args.players,
        args.seed,
    );
}
