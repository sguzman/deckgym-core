use clap::Parser;
use serde_json::Value;
use std::fs;

#[derive(Parser)]
#[command(name = "search")]
#[command(about = "Search for Pokémon cards by name")]
struct Args {
    /// The search query to match against card names (case-insensitive)
    query: String,

    /// Show only name and id instead of full card details
    #[arg(short, long)]
    simple: bool,

    /// Path to the database file
    #[arg(short, long, default_value = "database.json")]
    database: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read the database file
    let database_content = fs::read_to_string(&args.database)?;
    let database: Vec<Value> = serde_json::from_str(&database_content)?;

    // Search for matching cards
    let mut matches = Vec::new();
    let query_lower = args.query.to_lowercase();

    for card in database {
        // Check if it's a Pokemon card
        if let Some(pokemon) = card.get("Pokemon") {
            if let Some(name) = pokemon.get("name").and_then(|n| n.as_str()) {
                if name.to_lowercase().contains(&query_lower) {
                    if args.simple {
                        // Create a simplified version with just id and name
                        let simplified = serde_json::json!({
                            "Pokemon": {
                                "id": pokemon.get("id"),
                                "name": name
                            }
                        });
                        matches.push(simplified);
                    } else {
                        matches.push(card);
                    }
                }
            }
        }
        // Check if it's a Trainer card
        else if let Some(trainer) = card.get("Trainer") {
            if let Some(name) = trainer.get("name").and_then(|n| n.as_str()) {
                if name.to_lowercase().contains(&query_lower) {
                    if args.simple {
                        // Create a simplified version with just id and name
                        let simplified = serde_json::json!({
                            "Trainer": {
                                "id": trainer.get("id"),
                                "name": name
                            }
                        });
                        matches.push(simplified);
                    } else {
                        matches.push(card);
                    }
                }
            }
        }
    }

    // Output the results as JSON
    let output = serde_json::to_string_pretty(&matches)?;
    println!("{}", output);

    Ok(())
}
