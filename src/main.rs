use clap::{ArgAction, Parser};
use colored::Colorize;
use deckgym::{
    players::{parse_player_code, PlayerCode},
    simulate,
};
use env_logger::{Builder, Env};
use log::warn;
use std::io::Write;

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

    /// Increase verbosity (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = ArgAction::Count, default_value_t = 2)]
    verbose: u8,
}

/// The CLI tool to simulate games between two decks.
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

    warn!("Welcome to {}!", "deckgym".blue().bold());

    simulate(
        &args.deck_a,
        &args.deck_b,
        args.players,
        args.num,
        args.seed,
    );
}
