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

