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
├── game/                   # Core game logic crate (Tetris-like)
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
