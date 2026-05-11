# AGENTS.md

This document provides guidance for AI agents (e.g., GitHub Copilot, MCP servers, or LLM-based assistants) interacting with the Azure SDK for Rust repository.

## Repository Overview



- **Primary Language**: Rust
- **Minimum Supported Rust Version (MSRV)**: found in the root `Cargo.toml` file
- **Key Technologies**: Cargo, Dioxus 0.6

## Repository Structure

```text
.
├── database/migrations/*.sql
├── game/ - crate with basic game logic
├── game_net/ - crate game networking logic
├── protocol/ - crate with game p2p api routes
├── server/ - crate with p2p server with api bots

```

See the following descriptions:
# Sparganothis-v2 Code Map

```text
.
├── .cargo/                 # Cargo configuration
├── .github/                # GitHub workflows and actions
├── .kilo/                  # Kilo (planning/tracking) files
│   └── plans/              # Implementation plans
├── client/                 # Dioxus web client (frontend)
│   ├── assets/             # CSS, images, and other static assets
│   │   ├── main.css        # Primary application styles
│   │   └── pico.jade.min.css # Pico CSS base theme - DO NOT EDIT
│   └── src/                # Rust source code for the client
│       ├── comp/           # Reusable Dioxus components
│       ├── pages/          # Page-level components (homepage, etc.)
│       └── main.rs         # Client entry point
├── client_terminal/        # Terminal-based client
├── database/               # Database migrations and utilities
│   └── migrations/         # SQL migration files
├── dist2/                  # Compiled web assets
├── docs/                   # Project documentation and screenshots
├── game/                   # Core game logic crate
├── game_net/               # Networking logic for game sync/P2P
├── protocol/               # Shared API routes and data structures
├── server/                 # P2P server and API bots
└── target/                 # Build artifacts
```

## Core Components

- **game**: Basic game logic (board, pieces, scoring).
- **game_net**: P2P networking and synchronization.
- **protocol**: Definitions for communication between client and server.
- **server**: Host for P2P coordination and bot players.
- **client**: Modern web interface built with Dioxus 0.6.


## Agent Capabilities

Always check if there is an MCP tool or skill available before performing operations manually, including listing Azure subscriptions, deploying resources, setting up a new crate, generating code, and other common workflows.

### Recommended Actions

AI agents can assist with:

1. **Code Generation**
   - Writing new Rust code following the coding conventions below
   - Generating unit tests using `#[cfg(test)]` modules
   - Creating integration tests with `#[recorded::test]` attributes (see `CONTRIBUTING.md` for details)
   - Generating documentation tests in `.rs` files (avoid `no_run` when tests can be run)
   - Running `cargo fmt` and `cargo clippy` on all modified crates (see [Linting and Formatting](#linting-and-formatting))


5. **Refactoring**
   - Applying clippy suggestions
   - Improving code organization and modularity
   - Updating dependencies in `Cargo.toml`
   - Consolidating imports (e.g., `use std::{borrow::Cow, marker::PhantomData};` instead of separate lines)

### Restricted Actions

AI agents **should not**:

## Persona

You are an expert Rust programmer. You write safe, efficient, maintainable, and well-tested code.

- Use an informal tone.
- Do not be overly apologetic and focus on clear guidance.
- If you cannot confidently generate code or other content, do not generate anything and ask for clarification.

## Prerequisites

## Building

```bash
cargo build
```

## Testing

When running `cargo test`, use `--all-features` to ensure no tests are missed.

```bash
# Run tests for a specific crate
cargo test -p <crate-name> --all-features

# Run integration tests with recordings
cargo test -p <crate-name> --test <test-name>
```

```bash
# Format code
cargo fmt -p <crate-name>

# Lint code
cargo clippy -p <crate-name>

# Auto-fix some issues
cargo clippy --fix -p <crate-name>
```

## Code requirements

You will run the following after every code change:

- `cargo build` - Compilation check
- `cargo test` - Unit and integration tests
- `cargo clippy` - Lint checks
- `cargo fmt --check` - Format validation

You will then open a browser at 127.0.0.1:8080 and view the output to make sure it is correct.

## UI Development / CSS 

Read the "*CSS_*.md" files for a past planning of changing some UI - the files will probably be the same or around the listed files.

Run the following bash script to run the UI: `bash start_web.sh` in a simple bash shell 

Run the browser at 127.0.0.1:8080 after every build and click all the main pages to test them. Take screenshots that you place in docs/ folder and crate a new "SCREENSHOT_DIFF.md" file that takes note of the differences between the desired UI from chat and the actual UI. Maintain it along with any changes noted, the user will be deleting this file.

After the changes are recorded, change the patch such that it will match the target UI. Then, run the build again. If it passes, do the test scripts, then do clippy, and then format. Finally, run the browser again until satisfied the output matches, if it does not match, change the patch again and repeat the process.

