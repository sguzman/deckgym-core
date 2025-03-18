# deckgym-core: Pokemon TCG Pocket Battle Simulator

deckgym-core has the core simulator logic. It can simulate 10,000 games in ~3 seconds. It also exposes a CLI tool you can use to face off any two decks.

## CLI Usage

Create two text files in https://my.limitlesstcg.com/builder text format with the two decks you want to simulate against each other.

For example, create a `venusaur-exeggutor.txt` file with the following contents:

```
Pokémon: 10
1 Bulbasaur A1 1
1 Bulbasaur A1 227
2 Exeggcute A1 21
1 Exeggutor ex A1 23
1 Exeggutor ex A1 252
2 Ivysaur A1 2
2 Venusaur ex A1 4

Trainer: 10
2 Professor's Research P-A 7
2 Poké Ball P-A 5
2 Erika A1 219
1 Sabrina A1 225
2 X Speed P-A 2
1 Red Card P-A 6
```

and `weezing-arbok.txt` with:

```
Pokémon: 8
2 Ekans A1 164
2 Arbok A1 165
2 Koffing A1 176
2 Weezing A1 177

Trainer: 12
2 Professor's Research P-A 7
2 Koga A1 222
2 Poké Ball P-A 5
2 Sabrina A1 225
2 Potion P-A 1
1 X Speed P-A 2
1 Giovanni A1 223
```

Then, face them off each other 1000 times:

```
deckgym venusaur-exeggutor.txt weezing-arbok.txt --num 1000
```

## Developing Deckgym

If you want to advance deckgym development and features, ensure you have
Rust installed (see https://www.rust-lang.org/tools/install). And use the
following commands:

**Running Automated Test Suite**

```bash
cargo test
```

**Running Main Script**

```bash
RUST_LOG=debug cargo run --bin deckgym -- example_decks/venusaur-exeggutor.txt example_decks/weezing-arbok.txt --num 1 --players r,r
RUST_LOG=warn cargo run --bin deckgym -- example_decks/venusaur-exeggutor.txt example_decks/weezing-arbok.txt --num 1000 --players r,r
```

**Generating database.rs**
Ensure database.json is up-to-date with latest data. Mock the `get_card_by_enum` in `database.rs` with a `_ => panic` so that
it compiles mid-way through the generation.

```bash
cargo run --bin card_enum_generator > tmp.rs && mv tmp.rs src/card_ids.rs && cargo fmt
cargo run --bin card_enum_generator -- --database > tmp.rs && mv tmp.rs src/database.rs && cargo fmt
```

**Running Benchmarks**

```
cargo bench
```

**Profiling Main Script**

```
sudo RUST_LOG=warn cargo flamegraph --dev --bin deckgym -- example_decks/venusaur-exeggutor.txt example_decks/weezing-arbok.txt --num 100 && open flamegraph.svg
```
