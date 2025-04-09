use clap::{ArgAction, Parser};
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

    /// Increase verbosity (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = ArgAction::Count, default_value_t = 2)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();

    // Initialize the logger with the chosen log level.
    let level = match args.verbose {
        1 => "warn",
        2 => "info",
        3 => "debug",
        _ => "trace",
    };
    Builder::from_env(Env::default().default_filter_or(level))
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    warn!("Welcome to {} optimizer!", "deckgym".blue().bold());

    optimize(
        &args.incomplete_deck,
        &args.candidate_cards,
        &args.enemy_decks_folder,
        args.num,
        args.players,
        args.seed,
    );
}
