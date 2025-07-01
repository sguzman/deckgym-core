
# Agent Guide

This document outlines the process for agents to contribute to the `deckgym-core` project. The primary goal is to implement all Pokémon cards from `res/pokemon.csv` into the simulation engine.

## Core Workflow

The main loop for an agent working on this project should be:

1.  **Identify Unimplemented Cards**: Programmatically identify a set of cards that have not yet been implemented. You can cross-reference `res/pokemon.csv` with the existing card implementations in the `src` directory, particularly `src/database.rs`.
2.  **Implement a Single Card**: Follow the instructions in the "Contributing" section of the `README.md` to implement a single card. This involves:
    *   Implementing attacks in `src/actions/apply_attack_action.rs`.
    *   Implementing abilities in `src/move_generation/move_generation_abilities.rs` and `src/actions/apply_abilities_action.rs`.
    *   Implementing trainer/support cards in `src/move_generation/move_generation_trainer.rs` and `src/actions/apply_trainer_action.rs`.
3.  **Test the Implementation**: After implementing a card, run the test suite to ensure that the changes haven't broken anything.
4.  **Log Actions**: Log the actions taken in `res/agent.log`.
5.  **Repeat**: Continue this process until all cards are implemented.

## Post-Edit Workflow

After making any edits to the codebase, it is crucial to perform the following steps to maintain code quality and ensure the project remains stable:

1.  **Format**: Run `cargo fmt` to automatically format the code according to the project's style guidelines.
2.  **Test**: Run `cargo test` to execute the automated test suite and ensure that your changes have not introduced any regressions.
3.  **Build**: Run `cargo build` to confirm that the project compiles successfully.

This three-step process helps catch errors early and keeps the codebase clean and consistent.

## Key Files

*   `res/pokemon.csv`: The source of truth for all Pokémon card data.
*   `src/database.rs`: Contains the mapping of card IDs to their implementations.
*   `src/attack_ids.rs`: Enum and map for attack IDs.
*   `src/ability_ids.rs`: Enum and map for ability IDs.
*   `src/actions/apply_attack_action.rs`: Logic for applying attacks.
*   `src/actions/apply_abilities_action.rs`: Logic for applying abilities.
*   `src/move_generation/move_generation_abilities.rs`: Logic for generating ability moves.
*   `src/move_generation/move_generation_trainer.rs`: Logic for generating trainer moves.
*   `tests/`: Directory containing the test suite.
*   `res/agent.log`: Log file for agent actions.

## Project Audit and Phased Implementation Plan

An audit of the project has revealed a significant number of unimplemented cards, attacks, and abilities. To manage this, a multi-phase implementation plan has been developed:

*   **Phase 1: Foundational Card Implementation**: Focus on implementing the attacks for all Basic Pokémon. These are generally the simplest and will provide a solid base of implemented cards.

*   **Phase 2: Evolution Card Implementation**: Implement the attacks for all Stage 1 and Stage 2 Pokémon. This will build upon the work in Phase 1 and complete the attack implementation for all Pokémon.

*   **Phase 3: Ability Implementation**: Implement all Pokémon abilities (Poké-Powers and Poké-Bodies). This will be a significant undertaking, as abilities often have more complex logic than attacks.

*   **Phase 4: Trainer and Special Energy Card Implementation**: Implement the logic for all Trainer and Special Energy cards. This will complete the implementation of all card types.

*   **Phase 5: Full Integration and Refinement**: Ensure all cards from `res/pokemon.csv` are present in `database.json` and fully implemented in the engine. Conduct thorough testing of the entire system to identify and fix any remaining bugs or inconsistencies.

Progress will be tracked with git commits, ensuring that the project remains in a stable, buildable state after each change.

## Getting Started

1.  Read the `README.md` and this document carefully.
2.  Set up your development environment as described in the `README.md`.
3.  Start by implementing a single, simple Pokémon card to familiarize yourself with the process.
4.  Log your progress in `res/agent.log`.
