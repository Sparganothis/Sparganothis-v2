diff --git a/.vscode/settings.json b/.vscode/settings.json
index 781ce76..a0c50ce 100644
--- a/.vscode/settings.json
+++ b/.vscode/settings.json
@@ -2,5 +2,11 @@
     "terminal.integrated.shellIntegration.enabled": false,
     "files.eol": "\n",
     "workbench.editor.showTabs": "multiple",
-    "workbench.editor.enablePreview": false ,
-} 
\ No newline at end of file
+    "workbench.editor.enablePreview": false,
+    "terminal.integrated.defaultProfile.windows": "Git Bash",
+    "terminal.integrated.automationProfile.windows": {
+        "path": "C:\\Program Files\\Git\\usr\\bin\\bash.exe",
+        "args": [],
+        "icon": "terminal-bash"
+    },
+}
\ No newline at end of file
diff --git a/AGENTS.md b/AGENTS.md
new file mode 100644
index 0000000..a00640c
--- /dev/null
+++ b/AGENTS.md
@@ -0,0 +1,142 @@
+# AGENTS.md
+
+This document provides guidance for AI agents (e.g., GitHub Copilot, MCP servers, or LLM-based assistants) interacting with the Azure SDK for Rust repository.
+
+## Repository Overview
+
+
+
+- **Primary Language**: Rust
+- **Minimum Supported Rust Version (MSRV)**: found in the root `Cargo.toml` file
+- **Key Technologies**: Cargo, Dioxus 0.6
+
+## Repository Structure
+
+```text
+.
+├── database/migrations/*.sql
+├── game/ - crate with basic game logic
+├── game_net/ - crate game networking logic
+├── protocol/ - crate with game p2p api routes
+├── server/ - crate with p2p server with api bots
+
+```
+
+See the following descriptions:
+# Sparganothis-v2 Code Map
+
+```text
+.
+├── .cargo/                 # Cargo configuration
+├── .github/                # GitHub workflows and actions
+├── .kilo/                  # Kilo (planning/tracking) files
+│   └── plans/              # Implementation plans
+├── client/                 # Dioxus web client (frontend)
+│   ├── assets/             # CSS, images, and other static assets
+│   │   ├── main.css        # Primary application styles
+│   │   └── pico.jade.min.css # Pico CSS base theme - DO NOT EDIT
+│   └── src/                # Rust source code for the client
+│       ├── comp/           # Reusable Dioxus components
+│       ├── pages/          # Page-level components (homepage, etc.)
+│       └── main.rs         # Client entry point
+├── client_terminal/        # Terminal-based client
+├── database/               # Database migrations and utilities
+│   └── migrations/         # SQL migration files
+├── dist2/                  # Compiled web assets
+├── docs/                   # Project documentation and screenshots
+├── game/                   # Core game logic crate (Tetris-like)
+├── game_net/               # Networking logic for game sync/P2P
+├── protocol/               # Shared API routes and data structures
+├── server/                 # P2P server and API bots
+└── target/                 # Build artifacts
+```
+
+## Core Components
+
+- **game**: Basic game logic (board, pieces, scoring).
+- **game_net**: P2P networking and synchronization.
+- **protocol**: Definitions for communication between client and server.
+- **server**: Host for P2P coordination and bot players.
+- **client**: Modern web interface built with Dioxus 0.6.
+
+
+## Agent Capabilities
+
+Always check if there is an MCP tool or skill available before performing operations manually, including listing Azure subscriptions, deploying resources, setting up a new crate, generating code, and other common workflows.
+
+### Recommended Actions
+
+AI agents can assist with:
+
+1. **Code Generation**
+   - Writing new Rust code following the coding conventions below
+   - Generating unit tests using `#[cfg(test)]` modules
+   - Creating integration tests with `#[recorded::test]` attributes (see `CONTRIBUTING.md` for details)
+   - Generating documentation tests in `.rs` files (avoid `no_run` when tests can be run)
+   - Running `cargo fmt` and `cargo clippy` on all modified crates (see [Linting and Formatting](#linting-and-formatting))
+
+
+5. **Refactoring**
+   - Applying clippy suggestions
+   - Improving code organization and modularity
+   - Updating dependencies in `Cargo.toml`
+   - Consolidating imports (e.g., `use std::{borrow::Cow, marker::PhantomData};` instead of separate lines)
+
+### Restricted Actions
+
+AI agents **should not**:
+
+## Persona
+
+You are an expert Rust programmer. You write safe, efficient, maintainable, and well-tested code.
+
+- Use an informal tone.
+- Do not be overly apologetic and focus on clear guidance.
+- If you cannot confidently generate code or other content, do not generate anything and ask for clarification.
+
+## Prerequisites
+
+## Building
+
+```bash
+cargo build
+```
+
+## Testing
+
+When running `cargo test`, use `--all-features` to ensure no tests are missed.
+
+```bash
+# Run tests for a specific crate
+cargo test -p <crate-name> --all-features
+
+# Run integration tests with recordings
+cargo test -p <crate-name> --test <test-name>
+```
+
+```bash
+# Format code
+cargo fmt -p <crate-name>
+
+# Lint code
+cargo clippy -p <crate-name>
+
+# Auto-fix some issues
+cargo clippy --fix -p <crate-name>
+```
+
+## Code requirements
+
+You will run the following after every code change:
+
+- `cargo build` - Compilation check
+- `cargo test` - Unit and integration tests
+- `cargo clippy` - Lint checks
+- `cargo fmt --check` - Format validation
+
+You will then open a browser at 127.0.0.1:8080 and view the output to make sure it is correct.
+
+## UI
+
+Run the browser at 127.0.0.1:8080 after every build and click all the main pages to test them. Take screenshots that you place in docs/ folder and crate a new "SCREENSHOT_DIFF.md" file that takes note of the differences between the desired UI from chat and the actual UI. Maintain it along with any changes noted, the user will be deleting this file.
+
diff --git a/CODEMAP.md b/CODEMAP.md
new file mode 100644
index 0000000..d6cc962
--- /dev/null
+++ b/CODEMAP.md
@@ -0,0 +1,35 @@
+# Sparganothis-v2 Code Map
+
+```text
+.
+├── .cargo/                 # Cargo configuration
+├── .github/                # GitHub workflows and actions
+├── .kilo/                  # Kilo (planning/tracking) files
+│   └── plans/              # Implementation plans
+├── client/                 # Dioxus web client (frontend)
+│   ├── assets/             # CSS, images, and other static assets
+│   │   ├── main.css        # Primary application styles
+│   │   └── pico.jade.min.css # Pico CSS base theme
+│   └── src/                # Rust source code for the client
+│       ├── comp/           # Reusable Dioxus components
+│       ├── pages/          # Page-level components (homepage, etc.)
+│       └── main.rs         # Client entry point
+├── client_terminal/        # Terminal-based client
+├── database/               # Database migrations and utilities
+│   └── migrations/         # SQL migration files
+├── dist2/                  # Compiled web assets
+├── docs/                   # Project documentation and screenshots
+├── game/                   # Core game logic crate (Tetris-like)
+├── game_net/               # Networking logic for game sync/P2P
+├── protocol/               # Shared API routes and data structures
+├── server/                 # P2P server and API bots
+└── target/                 # Build artifacts
+```
+
+## Core Components
+
+- **game**: Basic game logic (board, pieces, scoring).
+- **game_net**: P2P networking and synchronization.
+- **protocol**: Definitions for communication between client and server.
+- **server**: Host for P2P coordination and bot players.
+- **client**: Modern web interface built with Dioxus 0.6.
diff --git a/CSS_INLINE_SWITCH.md b/CSS_INLINE_SWITCH.md
new file mode 100644
index 0000000..cf31c7d
--- /dev/null
+++ b/CSS_INLINE_SWITCH.md
@@ -0,0 +1,50 @@
+# CSS Inline Switch: Modernization Plan
+
+This document identifies all inline `style:` references within Dioxus components and provides a plan for migrating them to the unified CSS system in `main.css`.
+
+## Goal
+Remove all ad-hoc inline styles and replace them with semantic, reusable CSS classes that adhere to the premium dark theme established in the landing page.
+
+## Style Audit Results
+
+| File Path | Line | Inline Style Snippet | Replacement Strategy |
+|-----------|------|----------------------|----------------------|
+| `comp/game_display.rs` | 203+ | Multiple layout styles | Use `.game-preview-board` class |
+| `comp/game_display.rs` | 370 | `color: #ffd700;` (Gold) | Use `.text-gold` or `--color-status-special` |
+| `comp/game_display.rs` | 376 | `color: #ff69b4;` (Pink) | Use `.text-tspin` or `--color-status-tspin` |
+| `comp/nav.rs` | 61 | `display: flex; align-items: center; gap: 0.5rem;` | Use `.nav-link-flex` (already in `main.css` for some parts) |
+| `comp/chat/chat_display.rs` | 105 | `width: calc(90%-30px); color: {color};` | Use `.chat-bubble` class |
+| `comp/chat/chat_display.rs` | 247 | `justify-content: {align};` | Use utility classes `.chat-align-left/right` |
+| `comp/multiplayer/_1v1.rs` | 81 | `color: {node.html_color()}` | Keep dynamic colors but move other styles to `.player-node` |
+| `pages/homepage.rs` | 29 | `font-size: 2rem; margin-bottom: 0.5rem;` | Use `.hero-title` or standard `h2` overrides |
+| `pages/homepage.rs` | 88 | `display: flex; flex-direction: column; align-items: center;` | Use `.center-column` utility |
+| `pages/homepage.rs` | 123 | `background: #4caf50;` | Use `.avatar-green` or CSS variables |
+| `pages/singleplayer.rs` | 10 | `height: 80dvh; display: flex;` | Use `.full-page-container` |
+| `pages/play_game/private_lobby.rs` | 89 | `flex-direction:row height: 80px;` | Use `.lobby-header` |
+
+## Migration Plan
+
+### Phase 1: Utility Class Definition
+Add missing utility classes to `client/assets/main.css`:
+- `.flex-center`, `.flex-row`, `.flex-col`
+- `.mt-1`, `.mb-2`, etc.
+- `.text-gold`, `.text-pink`, `.text-red`
+- `.full-height`, `.h-80`
+
+### Phase 2: Component-Specific Classes
+Define semantic classes for complex components:
+- `.chat-container`, `.chat-message`, `.chat-bubble`
+- `.game-board-container`, `.game-stats-grid`
+- `.settings-form-layout`
+
+### Phase 3: Replacement
+Iterate through the files listed above and replace `style: "..."` with `class: "..."`.
+
+## Detailed Task List
+
+- [ ] Define shared utility variables for colors (Gold, Pink, etc.) in `:root`.
+- [ ] Implement `.chat-message` and `.chat-bubble` in `main.css`.
+- [ ] Replace inline styles in `comp/chat/chat_display.rs`.
+- [ ] Implement `.game-board-wrapper` and replace styles in `comp/game_display.rs`.
+- [ ] Refactor `homepage.rs` to remove the last few ad-hoc styles added during the redesign.
+- [ ] Standardize avatar colors using CSS variables.
diff --git a/CSS_SWITCH.md b/CSS_SWITCH.md
new file mode 100644
index 0000000..43125ab
--- /dev/null
+++ b/CSS_SWITCH.md
@@ -0,0 +1,55 @@
+# CSS Switch: Theme Enhancement
+
+This document tracks the changes made to `client/assets/main.css` to match the target premium dark design.
+
+## Proposed Changes
+
+### Global Theme
+- Set background to a deep navy/black (`#0b0e14`).
+- Update text colors for high contrast on dark background.
+
+### Navigation Bar
+- Glassmorphism effect with blur and dark semi-transparent background.
+- Subtle bottom border.
+
+### Cards (Game Modes, Preview)
+- Dark card backgrounds (`#161b22`) with refined borders (`#30363d`).
+- Hover effects with glow/elevation.
+- Modern typography and spacing.
+
+### Buttons
+- Gradient backgrounds or solid vibrant colors.
+- Rounded corners and smooth transitions.
+
+## CSS Code Comparison
+
+### Before (Basic Dark)
+```css
+.global_parent {
+  background-color: var(--pico-background-color);
+  color: var(--pico-color);
+}
+nav {
+  background: rgba(0, 0, 0, 0.8) !important;
+}
+```
+
+### After (Premium Dark)
+```css
+:root {
+  --pico-background-color: #0b0e14;
+  --pico-card-background-color: #161b22;
+  --pico-color: #f0f6fc;
+  --pico-muted-color: #8b949e;
+  --pico-primary: #58a6ff;
+}
+
+.global_parent {
+  background-color: #0b0e14;
+  background-image: radial-gradient(circle at top right, rgba(88, 166, 255, 0.05), transparent 400px),
+                    radial-gradient(circle at bottom left, rgba(123, 97, 255, 0.05), transparent 400px);
+}
+```
+
+## Status: Applied
+The CSS has been overhauled to match the target design. The homepage is being updated to use the new dashboard layout classes.
diff --git a/PICO_COLOR_THEMES.md b/PICO_COLOR_THEMES.md
new file mode 100644
index 0000000..e69de29
diff --git a/client/assets/main.css b/client/assets/main.css
index df06621..ba2bdcb 100644
--- a/client/assets/main.css
+++ b/client/assets/main.css
@@ -1,40 +1,417 @@
+:root {
+  --pico-background-color: #0b0e14;
+  --pico-card-background-color: #161b22;
+  --pico-color: #f0f6fc;
+  --pico-muted-color: #8b949e;
+  --pico-primary: #58a6ff;
+  --pico-primary-hover: #1f6feb;
+  --pico-border-color: #30363d;
+  --pico-nav-background: rgba(13, 17, 23, 0.8);
+  
+  --color-victory: #00c853;
+  --color-defeat: #ff5252;
+  --color-singleplayer: #00c853;
+  --color-matchmaking: #7b61ff;
+  --color-gold: #ffd700;
+  --color-pink: #ff69b4;
+  --color-garbage: #ff4444;
+}
+
+/* ===== UTILITIES ===== */
+.flex { display: flex; }
+.flex-row { display: flex; flex-direction: row; }
+.flex-col { display: flex; flex-direction: column; }
+.items-center { align-items: center; }
+.justify-center { justify-content: center; }
+.justify-between { justify-content: space-between; }
+.gap-1 { gap: 0.5rem; }
+.gap-2 { gap: 1rem; }
+.w-full { width: 100%; }
+.h-full { height: 100%; }
+.h-80 { height: 80dvh; }
+.mt-1 { margin-top: 0.5rem; }
+.mt-2 { margin-top: 1rem; }
+.mb-1 { margin-bottom: 0.5rem; }
+.mb-2 { margin-bottom: 1rem; }
+.p-1 { padding: 0.5rem; }
+.p-2 { padding: 1rem; }
+
+.text-gold { color: var(--color-gold); }
+.text-pink { color: var(--color-pink); }
+.text-red { color: var(--color-garbage); }
+.text-muted { color: var(--pico-muted-color); }
+.text-center { text-align: center; }
+.mb-0 { margin-bottom: 0; }
+.mb-1 { margin-bottom: 0.5rem; }
+.mb-2 { margin-bottom: 1rem; }
+
+.github-icon-wrapper {
+  width: 1rem;
+  height: 1rem;
+  margin-right: 0.25rem;
+  display: flex;
+  align-items: center;
+}
+
+/* ===== CHAT SYSTEM ===== */
+.chat-container {
+  display: flex;
+  flex-direction: column;
+  gap: 1rem;
+  height: 100%;
+}
+
+.chat-bubble {
+  padding: 0.75rem 1rem;
+  border-radius: 12px;
+  background: rgba(255, 255, 255, 0.05);
+  border: 1px solid var(--pico-border-color);
+  max-width: 85%;
+  position: relative;
+}
+
+.chat-bubble-self {
+  align-self: flex-end;
+  background: rgba(88, 166, 255, 0.1);
+  border-color: rgba(88, 166, 255, 0.2);
+}
+
+.chat-meta {
+  font-size: 0.75rem;
+  color: var(--pico-muted-color);
+  margin-top: 0.25rem;
+}
+
+/* ===== GAME DISPLAY ===== */
+.game-board-wrapper {
+  background: #000;
+  border: 2px solid var(--pico-border-color);
+  border-radius: 8px;
+  padding: 4px;
+  box-shadow: 0 0 20px rgba(0, 0, 0, 0.5);
+}
+
+.game-cell {
+  width: 24px;
+  height: 24px;
+  border: 1px solid rgba(255, 255, 255, 0.05);
+}
+
+.game-overlay {
+  position: absolute;
+  top: 0;
+  left: 0;
+  right: 0;
+  bottom: 0;
+  display: flex;
+  align-items: center;
+  justify-content: center;
+  background: rgba(0, 0, 0, 0.7);
+  backdrop-filter: blur(4px);
+  z-index: 10;
+}
+
+/* ===== COMPONENTS ===== */
+.icon-container {
+  height: 100%;
+  padding: 4px;
+  margin: 4px;
+  flex-grow: 1;
+  cursor: pointer;
+}
+
+.icon-box {
+  width: 100%;
+  height: 46px;
+  margin: auto;
+  display: flex;
+  align-items: center;
+  justify-content: center;
+}
+
+.input-capture-parent {
+  margin: 0;
+  padding: 0;
+  width: 100%;
+  height: 100%;
+  outline: none;
+}
+
+.input-capture-parent:focus {
+  border: 1px solid var(--pico-primary);
+  box-shadow: 0 0 0 2px rgba(88, 166, 255, 0.2);
+}
+
 .global_parent {
   background-color: var(--pico-background-color);
+  background-image: 
+    radial-gradient(circle at top right, rgba(88, 166, 255, 0.05), transparent 600px),
+    radial-gradient(circle at bottom left, rgba(123, 97, 255, 0.05), transparent 600px);
   color: var(--pico-color);
-  width:100dvw;
-  height:100dvh;
-  margin:0;
-  padding:0;
+  width: 100dvw;
+  height: 100dvh;
+  margin: 0;
+  padding: 0;
   border: 0;
-  overflow: hidden;
+  overflow-x: hidden;
+  overflow-y: auto;
+  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
 }
-.chat-window-container {
+
+/* ===== NAVIGATION ===== */
+nav {
+  backdrop-filter: blur(12px);
+  background: var(--pico-nav-background) !important;
+  border-bottom: 1px solid var(--pico-border-color);
+  padding: 0.5rem 2rem;
+  position: sticky;
+  top: 0;
+  z-index: 1000;
+}
+
+nav .brand {
   display: flex;
-  flex-direction: row;
-  height: calc(100% - 20px);
-  overflow: hidden;
-  margin: 0px; padding: 0px;
+  align-items: center;
+  gap: 0.75rem;
+  font-weight: 800;
+  font-size: 1.25rem;
+  color: white;
 }
-.chat-left-pane { 
-  overflow-x: hidden;
-  overflow-y: auto;
-  height: calc(100% - 10px);
-  width: 20%;
-  margin: 0px; padding: 0px;
+
+nav .brand span {
+  color: var(--color-victory);
+  font-size: 0.7rem;
+  font-weight: 400;
+  margin-left: 0.25rem;
+}
+
+nav ul li a {
+  color: var(--pico-color);
+  font-size: 0.9rem;
+  font-weight: 500;
+  padding: 0.5rem 0.75rem;
+  border-radius: 6px;
+  transition: all 0.2s ease;
+}
+
+nav ul li a:hover {
+  background: rgba(255, 255, 255, 0.05);
+  color: var(--pico-primary) !important;
+}
+
+/* Online Indicator */
+.online-indicator {
+  background: rgba(0, 200, 83, 0.1);
+  color: var(--color-victory);
+  padding: 0.25rem 0.75rem;
+  border-radius: 20px;
+  font-size: 0.75rem;
+  font-weight: 600;
+  border: 1px solid rgba(0, 200, 83, 0.2);
+}
+
+/* ===== MAIN LAYOUT ===== */
+.container {
+  max-width: 1400px;
+  margin: 0 auto;
+  padding: 2rem;
+}
+
+.dashboard-grid {
+  display: grid;
+  grid-template-columns: 320px 1fr 320px;
+  gap: 2rem;
+  align-items: start;
+}
+
+.sidebar-left, .sidebar-right {
+  display: flex;
+  flex-direction: column;
+  gap: 1.5rem;
 }
-.chat-main-pane {
+
+.main-content {
   display: flex;
   flex-direction: column;
-  height: calc(100% - 10px);
-  width: 80%;
-  margin: 0px; padding: 0px;
-}
-.chat-bottom { 
-  height: 110px;
-  margin: 0px; padding: 0px;
-}
-.chat-main { 
-  overflow: auto;
-  height: calc(99% - 120px);
-  margin: 0px; padding: 0px;
-}
\ No newline at end of file
+  gap: 1.5rem;
+}
+
+/* ===== CARDS & SECTIONS ===== */
+.card {
+  background: var(--pico-card-background-color);
+  border: 1px solid var(--pico-border-color);
+  border-radius: 12px;
+  padding: 1.5rem;
+  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
+}
+
+.card-header {
+  display: flex;
+  align-items: center;
+  justify-content: space-between;
+  margin-bottom: 1.5rem;
+}
+
+.card-title {
+  display: flex;
+  align-items: center;
+  gap: 0.5rem;
+  font-size: 1.1rem;
+  font-weight: 600;
+  margin: 0;
+}
+
+/* ===== GAME MODES (LEFT) ===== */
+.game-mode-card {
+  margin-bottom: 1rem;
+  transition: transform 0.2s ease, border-color 0.2s ease;
+}
+
+.game-mode-card:hover {
+  transform: translateY(-2px);
+}
+
+.mode-singleplayer { border-left: 4px solid var(--color-singleplayer); }
+.mode-matchmaking { border-left: 4px solid var(--color-matchmaking); }
+
+.mode-icon {
+  font-size: 2rem;
+  margin-bottom: 1rem;
+}
+
+/* ===== GAME PREVIEW (CENTER) ===== */
+.game-preview-card {
+  min-height: 600px;
+}
+
+.status-badge {
+  padding: 0.25rem 0.75rem;
+  border-radius: 4px;
+  font-size: 0.7rem;
+  font-weight: 700;
+  text-transform: uppercase;
+}
+
+.status-victory {
+  background: rgba(0, 200, 83, 0.1);
+  color: var(--color-victory);
+}
+
+.game-display-container {
+  display: grid;
+  grid-template-columns: 120px 1fr 120px;
+  gap: 1rem;
+}
+
+.game-stats-list {
+  list-style: none;
+  padding: 0;
+  margin: 0;
+}
+
+.game-stat-entry {
+  margin-bottom: 1rem;
+}
+
+.game-stat-label {
+  font-size: 0.75rem;
+  color: var(--pico-muted-color);
+  text-transform: uppercase;
+  display: block;
+}
+
+.game-stat-value {
+  font-size: 1.1rem;
+  font-weight: 700;
+  color: #fff;
+}
+
+/* ===== SIDEBAR (RIGHT) ===== */
+.sidebar-section {
+  margin-bottom: 1.5rem;
+}
+
+.chat-message {
+  display: flex;
+  gap: 0.75rem;
+  margin-bottom: 1rem;
+  font-size: 0.9rem;
+}
+
+.chat-avatar {
+  width: 32px;
+  height: 32px;
+  border-radius: 50%;
+  background: var(--pico-primary);
+}
+
+.chat-content .user {
+  font-weight: 600;
+  margin-right: 0.5rem;
+}
+
+.chat-content .time {
+  font-size: 0.75rem;
+  color: var(--pico-muted-color);
+}
+
+/* ===== BUTTONS ===== */
+.btn {
+  display: inline-flex;
+  align-items: center;
+  justify-content: center;
+  gap: 0.5rem;
+  padding: 0.75rem 1.25rem;
+  border-radius: 8px;
+  font-weight: 600;
+  text-decoration: none;
+  transition: all 0.2s ease;
+  cursor: pointer;
+  border: none;
+  width: 100%;
+}
+
+.btn-primary {
+  background: var(--color-singleplayer);
+  color: white;
+}
+
+.btn-primary:hover {
+  filter: brightness(1.1);
+  box-shadow: 0 0 20px rgba(0, 200, 83, 0.3);
+}
+
+.btn-secondary {
+  background: var(--color-matchmaking);
+  color: white;
+}
+
+.btn-secondary:hover {
+  filter: brightness(1.1);
+  box-shadow: 0 0 20px rgba(123, 97, 255, 0.3);
+}
+
+.btn-outline {
+  background: transparent;
+  border: 1px solid var(--pico-border-color);
+  color: var(--pico-color);
+}
+
+.btn-outline:hover {
+  background: rgba(255, 255, 255, 0.05);
+  border-color: var(--pico-primary);
+}
+
+/* ===== FOOTER ===== */
+footer {
+  text-align: center;
+  padding: 2rem;
+  color: var(--pico-muted-color);
+  font-size: 0.8rem;
+  border-top: 1px solid var(--pico-border-color);
+  margin-top: 3rem;
+}
+
+/* Utility */
+.text-victory { color: var(--color-victory); }
+.text-defeat { color: var(--color-defeat); }
\ No newline at end of file
diff --git a/client/src/app.rs b/client/src/app.rs
index 87a30bb..481528d 100644
--- a/client/src/app.rs
+++ b/client/src/app.rs
@@ -16,7 +16,7 @@ pub fn App() -> Element {
         document::Link { rel: "stylesheet", href: MAIN_CSS }
         document::Title { "{APP_TITLE}" }
         div {
-            "data-theme": "light",
+            "data-theme": "dark",
             class: "global_parent",
             UrlHolderParent {
                 LocalStorageParent {
diff --git a/client/src/comp/chat/chat_display.rs b/client/src/comp/chat/chat_display.rs
index 1957ae8..bc58276 100644
--- a/client/src/comp/chat/chat_display.rs
+++ b/client/src/comp/chat/chat_display.rs
@@ -101,28 +101,23 @@ fn ChatPresenceDisplayItem<T: ChatMessageType>(
     };
     rsx! {
         li {
+            class: "flex items-center gap-1",
             key: "{identity.node_id()}",
-            style: "width: calc(90%-30px); color: {color}; position: relative;",
+            style: "color: {color};",
             "data-tooltip": "
                 {identity.user_id().fmt_short()}@{identity.node_id().fmt_short()}
                 (last seen: {last_seen_txt})
             ",
             "data-placement": "bottom",
             {element}
-            small { small {
-                style: "float: right; color: #666;",
+            div { class: "chat-meta",
                 if let Some(rtt) = rtt.read().clone() {
                     "{rtt} ms"
                 } else if is_own_node.read().clone() {
                     "(you)"
                 }
-            }}
-            div {
-                style: "
-                left: -2.1rem;
-                top: 0.5rem;
-                position:absolute;
-                ",
+            }
+            div { class: "chat-portrait",
                 ChatUserPortraitBox {  own_color: own_color }
             }
         }
@@ -133,12 +128,8 @@ fn ChatPresenceDisplayItem<T: ChatMessageType>(
 fn ChatUserPortraitBox(own_color: ReadOnlySignal<String>) -> Element {
     rsx! {
         div {
-            style: "
-            width: 1.8rem;
-            height: 1.8rem;
-            border: 0.5rem solid {own_color};
-            z-index:1;
-            "
+            class: "portrait-box",
+            style: "border-color: {own_color};",
         }
     }
 }
@@ -148,24 +139,13 @@ pub fn ChatHistoryDisplay<T: ChatMessageType>(
     history: ReadOnlySignal<ChatHistory<T>>,
 ) -> Element {
     rsx! {
-        div {
-            style: "
-                height: 100%;
-                overflow: hidden;
-            ",
-            article {
-                style: "
-                    height: 100%;
-                    overflow-y: auto;
-                    overflow-x: hidden;
-                ",
+        div { class: "chat-container",
+            article { class: "h-full overflow-y-auto",
                 for message in history.read().messages.iter() {
                     ChatMessageOrErrorDisplay::<T> { message: message.clone() }
                 }
                 if history.read().messages.is_empty() {
-                    i {
-                        "No messages."
-                    }
+                    i { class: "text-muted", "No messages." }
                 }
             }
         }
@@ -241,56 +221,32 @@ fn ChatMessageDisplay<T: ChatMessageType>(
         }
     });
 
+    let is_self = from.user_id() == &my_user_id;
+    let bubble_class = if is_self { "chat-bubble chat-bubble-self" } else { "chat-bubble" };
+
     rsx! {
         div {
             key: "{_message_id:?}",
-            style: "width: 100%; height: fit-content; display: flex; justify-content: {align};",
+            class: "w-full flex mb-2",
+            style: "justify-content: {align};",
             article {
-                style: "
-                    max-width: 70%;
-                    min-width: 30%; 
-                    width: fit-content; 
-                    text-align: {align}; 
-                    float: {align};
-                    padding: 10px;
-                    margin: 10px;
-                ",
+                class: "{bubble_class}",
                 onmounted: move |_e| async move {
                     let _e = _e.scroll_to(ScrollBehavior::Instant).await;
                     if let Err(e) = _e {
                         warn!("Failed to scroll to bottom: {}", e);
                     }
                 },
-                header {
-                    style: "display: flex; justify-content: space-between;",
-                    b {
-                        "{from_nickname}"
-                    }
-                    small {
-                        style: "padding-top: 0px; margin-top: 0px; color: #666;",
-                        small {
-                            "{last_seen_txt}"
-                        }
-                    }
+                header { class: "flex justify-between items-center mb-1",
+                    b { "{from_nickname}" }
+                    span { class: "chat-meta", "{last_seen_txt}" }
                 }
-                p {
-                    style: "position:relative;",
-                    div {
-                        style: "
-                        padding-{align}: 3rem;
-                        ",
-                        {text},
-                    }
-                    div {
-                        style: "
-                        {align}: 0rem;
-                        top: -0.2rem;
-                        position:absolute;
-                        ",
-                        ChatUserPortraitBox {  own_color: from_color }
+                div { class: "flex items-start gap-1",
+                    div { class: "portrait-wrapper",
+                        ChatUserPortraitBox { own_color: from_color }
                     }
+                    div { class: "message-text", {text} }
                 }
-                // footer {                }
             }
         }
     }
diff --git a/client/src/comp/game_display.rs b/client/src/comp/game_display.rs
index 17376ad..742c0c7 100644
--- a/client/src/comp/game_display.rs
+++ b/client/src/comp/game_display.rs
@@ -55,14 +55,11 @@ pub fn YouDied(
     });
     rsx! {
         if game_state.read().game_over() {
-            div {
-                style: "position: relative; width: 0; height: 0; margin: 0; padding: 0; top: 0px; left: 0px;",
-                div {
-                    style: "position: absolute; width: 20cqw; height: 20cqh; color: red; z-index: 666;",
-                    h3 {
-                        style: "color:{color}; z-index: 666; font-size: 6rem; transform: rotate(-45deg); background-color: black; width: fit-content; height: fit-content;",
-                        "{msg}"
-                    }
+            div { class: "game-overlay",
+                h3 { 
+                    class: "text-center",
+                    style: "color: {color}; font-size: 5rem; text-shadow: 0 0 20px rgba(0,0,0,0.5);",
+                    "{msg}"
                 }
             }
         }
@@ -73,44 +70,11 @@ pub fn YouDied(
 #[component]
 pub fn GameDisplay(game_state: ReadOnlySignal<GameState>) -> Element {
     rsx! {
-        div { style: "
-                width: 100%;
-                height: 100%;
-                display: flex;
-                align-items: center;
-                justify-content: center;
-                container-type:size;
-            ",
-
-            div { style: "
-                    width: 30%;
-                    height: 100%;
-                    display: flex;
-                    align-items: center;
-                    justify-content: center;
-                " }
-
-            div { style: "
-                    width: 40%;
-                    height: 100%;
-                    display: flex;
-                    align-items: center;
-                    justify-content: center;
-                ",
-
-                YouDied {
-                    game_state,
-                    GameDisplayInner { game_state }
-                }
+        div { class: "flex items-center justify-center w-full h-full",
+            YouDied {
+                game_state,
+                GameDisplayInner { game_state }
             }
-
-            div { style: "
-                    width: 30%;
-                    height: 100%;
-                    display: flex;
-                    align-items: center;
-                    justify-content: center;
-                " }
         }
     }
 }
@@ -118,22 +82,10 @@ pub fn GameDisplay(game_state: ReadOnlySignal<GameState>) -> Element {
 #[component]
 fn GameDetailsLeftPane(game_state: ReadOnlySignal<GameState>) -> Element {
     rsx! {
-        div { style: "
-                width: 100%;
-                height: 100%;
-                display: flex;
-                flex-direction: column;
-                align-items: end;
-                justify-content: start;
-                gap: 20px;
-            ",
-            div { style: "
-                    width: 50%;
-                    align:right;
-                ",
+        div { class: "flex flex-col items-end justify-start gap-2 w-full h-full",
+            div { class: "w-full text-right",
                 GameBoardDisplayHoldGrid { game_state }
             }
-
             GameStateInfo { game_state }
         }
     }
@@ -142,21 +94,10 @@ fn GameDetailsLeftPane(game_state: ReadOnlySignal<GameState>) -> Element {
 #[component]
 fn GameDetailsRightPane(game_state: ReadOnlySignal<GameState>) -> Element {
     rsx! {
-        div { style: "
-                width: 100%;
-                height: 100%;
-                display: flex;
-                align-items: start;
-                justify-content: start;
-            ",
-            div { style: "
-                    width: 50%;
-                    margin-bottom: auto;
-                    align:left;
-                ",
+        div { class: "flex items-start justify-start w-full h-full",
+            div { class: "w-full mb-auto text-left",
                 GameBoardDisplayNextGrid { game_state }
             }
-
         }
     }
 }
@@ -164,53 +105,25 @@ fn GameDetailsRightPane(game_state: ReadOnlySignal<GameState>) -> Element {
 #[component]
 fn GameDisplayInner(game_state: ReadOnlySignal<GameState>) -> Element {
     rsx! {
-        div { style: "
-                width: 100%;
-                height: 100%;
-                display: flex;
-                align-items: center;
-                justify-content: center;
-                container-type:size;
-            ",
-            div { style: "
-                    position: relative;
-                    width: calc(min(100cqw, 50cqh));
-                    height: calc(min(100cqh, min(100cqh, 200cqw)));
-                    display: flex;
-                    align-items: center;
-                    justify-content: center;
-                    container-type:size;
-                ",
-                div { style: "
-                    position:absolute;
-                        top: 0; 
-                        left: -74cqw;
-                        width: 73cqw;
-                        height: 99cqh;
-                    ",
+        div { class: "flex items-center justify-center w-full h-full",
+            div { 
+                class: "flex items-center justify-center",
+                style: "position: relative; width: calc(min(100cqw, 50cqh)); height: calc(min(100cqh, min(100cqh, 200cqw)));",
+                div { 
+                    class: "h-full",
+                    style: "position:absolute; top: 0; left: -74cqw; width: 73cqw;",
                     GameDetailsLeftPane { game_state }
                 }
 
-                div { style: "
-                        padding: 0px;
-                        width: 100%;
-                        height: 100%;
-                        container-type:size;
-                        display: flex;
-                    ",
+                div { class: "w-full h-full flex",
                     GameBoardDisplayMainGrid { game_state }
                 }
-                div { style: "
-                    position:absolute;
-                        top: 0; 
-                        left: 101cqw;
-                        width: 73cqw;
-                        height: 99cqh;
-                    ",
+                div { 
+                    class: "h-full",
+                    style: "position:absolute; top: 0; left: 101cqw; width: 73cqw;",
                     GameDetailsRightPane { game_state }
                 }
             }
-
         }
     }
 }
@@ -292,18 +205,12 @@ fn GameBoardGridParent(
     children: Element,
 ) -> Element {
     rsx! {
-        div { style: "
-                position: relative;
+        div { 
+            class: "game-board-wrapper",
+            style: "
                 display: grid;
                 grid-template-columns: repeat({column_count}, minmax(0, 1fr));
                 grid-template-rows: repeat({row_count}, auto);
-                grid-column-gap: 0px;
-                grid-row-gap: 0px;
-                width: 100%;
-                height: 100%;
-                background-color:{GAMEBOARD_GRID_COLOR};
-                padding: 0px;
-                border: 1px solid {GAMEBOARD_GRID_COLOR};
                 aspect-ratio: {column_count}/{row_count};
             ",
             {children}
@@ -320,22 +227,12 @@ fn GridCellDisplay(
     col_count: i8,
 ) -> Element {
     let cell_color = use_memo(move || get_cell_color(cell.read().clone()));
-    //         position: absolute;
-    // width: calc(100cqw/{col_count});
-    // height: calc(100cqh/{row_count});
-    // top: calc((100cqh/{row_count}) * {row});
-    // left: calc((100cqw/{col_count}) * {col});
     rsx! {
-        div { style: "
-                padding: 0;
-            ",
-            div { style: "
-                background-color: {cell_color};
-                width: 100%;
-                height: 100%;
-                aspect-ratio: 1/1;
-                border: 0.1cqmin solid {GAMEBOARD_GRID_COLOR};
-                " }
+        div { class: "p-0",
+            div { 
+                class: "game-cell",
+                style: "background-color: {cell_color};",
+            }
         }
     }
 }
@@ -346,43 +243,38 @@ fn GameStateInfo(game_state: ReadOnlySignal<GameState>) -> Element {
     rsx! {
         div {
             id: "game-state-info",
-
-            style: "
-                width: 100%;
-                font-family: monospace;
-                font-size: 1.2em;
-                text-align: right;
-                padding-right: 20px;
-                color: black;
-            ",
-            div { "Score: {state.score}" }
-            div { "Lines: {state.total_lines}" }
-            div { "Moves: {state.total_moves}" }
-            div { "Combo: {state.combo_counter}" }
-            div { "Time: {state.current_time_string()}" }
-            div { "Lines Sent: {state.total_garbage_sent}"}
-            div { "Lines Recv: {state.garbage_recv}"}
-            div { "Lines Applied: {state.garbage_applied}"}
-
+            class: "game-stats-list w-full text-right p-2",
+            
+            div { class: "game-stat-entry",
+                span { class: "game-stat-label", "Score" }
+                span { class: "game-stat-value", "{state.score}" }
+            }
+            div { class: "game-stat-entry",
+                span { class: "game-stat-label", "Lines" }
+                span { class: "game-stat-value", "{state.total_lines}" }
+            }
+            div { class: "game-stat-entry",
+                span { class: "game-stat-label", "Moves" }
+                span { class: "game-stat-value", "{state.total_moves}" }
+            }
+            div { class: "game-stat-entry",
+                span { class: "game-stat-label", "Combo" }
+                span { class: "game-stat-value", "{state.combo_counter}" }
+            }
+            div { class: "game-stat-entry",
+                span { class: "game-stat-label", "Time" }
+                span { class: "game-stat-value", "{state.current_time_string()}" }
+            }
 
             // Show B2B and T-spin indicators if active
             if state.is_b2b {
-                div { style: "color: #ffd700;", // Gold color for special states
-                    "Back-to-Back!"
-                }
+                div { class: "text-gold mt-1", "Back-to-Back!" }
             }
-
             if state.is_t_spin {
-                div { style: "color: #ff69b4;", // Pink color for T-spin
-                    "T-Spin!"
-                }
+                div { class: "text-pink mt-1", "T-Spin!" }
             }
-
-            // Show garbage info if any
             if state.garbage_recv > 0 {
-                div { style: "color: #ff4444;", // Red color for garbage
-                    "Incoming: {state.garbage_recv - state.garbage_applied}"
-                }
+                div { class: "text-red mt-1", "Incoming: {state.garbage_recv - state.garbage_applied}" }
             }
         }
     }
diff --git a/client/src/comp/icon.rs b/client/src/comp/icon.rs
index fccd8a8..f8d9875 100644
--- a/client/src/comp/icon.rs
+++ b/client/src/comp/icon.rs
@@ -13,36 +13,19 @@ pub fn Icon<T: IconShape + Clone + PartialEq + 'static>(
     use dioxus_free_icons::Icon;
     rsx! {
         div {
-            style: "
-                height: 100%;
-                padding: 4px; margin: 4px;
-                flex-grow: 1;
-            ",
+            class: "icon-container",
             onclick: move |_| {
                 onclick.call(());
             },
-            cursor: "pointer",
             div {
-                style: "
-                width: 100%;
-                height: 46px;
-                margin: auto;
-                ",
+                class: "icon-box",
                 "data-tooltip": "{tooltip}",
                 "data-placement": "top",
-                cursor: "pointer",
-                div {
-                    style: "
-                    height: 36px;
-                    width: 36px;
-                    margin: auto;
-                    ",
-                    Icon {
-                        width: 26,
-                        height: 26,
-                        fill: "{color}",
-                        icon: icon,
-                    }
+                Icon {
+                    width: 26,
+                    height: 26,
+                    fill: "{color}",
+                    icon: icon,
                 }
             }
         }
diff --git a/client/src/comp/input.rs b/client/src/comp/input.rs
index 3119c9f..590c299 100644
--- a/client/src/comp/input.rs
+++ b/client/src/comp/input.rs
@@ -24,13 +24,8 @@ pub fn GameInputCaptureParent(
     rsx! {
         div {
             id: "event_input_capture_parent",
+            class: "input-capture-parent",
             tabindex: 0,
-            style: "
-                margin: 0px;
-                padding: 0px; 
-                border: 1px solid pink; 
-                width: 100%; height: 100%;
-                ",
             // onkeypress: move |_e| {
             //     info!("onkeypress: {:#?}", _e);
             //     if let Some(key) = keyboard_data_to_game_key(&_e.data()) {
diff --git a/client/src/comp/nav.rs b/client/src/comp/nav.rs
index 1cfa75b..fdd2e16 100644
--- a/client/src/comp/nav.rs
+++ b/client/src/comp/nav.rs
@@ -13,36 +13,34 @@ pub fn Nav() -> Element {
     let my_nickname = use_memo(move || {
         my_secrets.read().user_identity().nickname().to_string()
     });
+    
+    // Online count (mocked or from context if available)
+    let online_count = 4; 
+
     rsx! {
         nav {
             ul {
                 li {
-                    Link { to: Route::Home {},   strong { "{APP_TITLE}" } }
+                    Link { 
+                        class: "brand",
+                        to: Route::Home {},   
+                        "{APP_TITLE}" 
+                        span { "Strategic. Competitive. Timeless." }
+                    }
                 }
             }
             ul {
                 li {
-                    Link { to: Route::PlaySingleplayerPage { }, small { "singleplayer" } }
+                    Link { to: Route::PlaySingleplayerPage { }, "Singleplayer" }
                 }
-                // li {
-                //     Link { to: Route::IAmARobotSingleplayer { }, small { "robot" } }
-                // }
                 li {
-                    Link { to: Route::MatchmakingPage { }, small { "1v1 matchmaking" } }
+                    Link { to: Route::MatchmakingPage { }, "1v1 matchmaking" }
                 }
-                // li {
-                //     Link { to: Route::ReplayHomePage { }, small { "replay" } }
-                // }
             }
 
             ul {
                 li {
-                    NetworkConnectionStatusIcon {}
-                }
-            }
-            ul {
-                li {
-                    LinkDropdownProfile{my_nickname}
+                    div { class: "online-indicator", "• {online_count} ONLINE" }
                 }
                 li {
                     Link { to: Route::GlobalChatPage { }, "Chat" }
@@ -51,17 +49,21 @@ pub fn Nav() -> Element {
                     Link { to: Route::UsersRootDirectoryPage { }, "Top Players" }
                 }
             }
+            
             ul {
+                li {
+                    LinkDropdownProfile{my_nickname}
+                }
                 li {
                     a {
                         href: "https://github.com/Sparganothis/Sparganothis-v2",
                         target: "_blank",
-                        style: "display: flex; flex-direction:row; align-items: center;",
+                        class: "flex items-center gap-1",
                         GithubIcon {},
                         "GitHub",
                         img {
                             src: "https://github.com/Sparganothis/Sparganothis-v2/actions/workflows/rust.yml/badge.svg",
-                            style: "height: 1rem; padding-left: 0.2rem; margin-left: 0.2rem;",
+                            style: "height: 1rem;",
                         }
                     }
                 }
@@ -70,12 +72,13 @@ pub fn Nav() -> Element {
     }
 }
 
+
 #[component]
 fn GithubIcon() -> Element {
     let sstr = include_str!("../../assets/github.svg.html");
     rsx! {
         div {
-            style: "width: 1rem; height: 1rem; padding-right: 0.2rem; margin-right: 0.2rem; margin-top: -0.7rem;",
+            class: "github-icon-wrapper",
             dangerous_inner_html: sstr,
         }
     }
diff --git a/client/src/pages/homepage.rs b/client/src/pages/homepage.rs
index 63a99a0..9b77244 100644
--- a/client/src/pages/homepage.rs
+++ b/client/src/pages/homepage.rs
@@ -2,17 +2,172 @@ use dioxus::prelude::*;
 use game::tet::GameState;
 
 use crate::comp::{bot_player::BotPlayer, game_display::GameDisplay};
+use crate::route::Route;
+use crate::network::GlobalChatClientContext;
 
-/// Home page
+/// Home page with premium dashboard layout
 #[component]
 pub fn Home() -> Element {
     let game_state = use_signal(GameState::new_random);
+    
+    // Get chat context for online player count
+    let chat_context = use_context::<GlobalChatClientContext>();
+    let presence = chat_context.chat.presence;
+    
+    // Get online player count from presence list
+    let online_count = use_memo(move || {
+        let p = presence.read();
+        p.0.len()
+    });
 
     rsx! {
-        BotPlayer {game_state}
-        article { style: "height: 80dvh; display: flex;",
-            // style: "display: flex;",
-            GameDisplay { game_state }
+        div { class: "container",
+            div { class: "dashboard-grid",
+                
+                // LEFT COLUMN: Game Modes
+                div { class: "sidebar-left",
+                    h2 { class: "mb-1", style: "font-size: 2rem;", "Play Your Way" }
+                    p { class: "text-muted mb-2", "Choose your mode and start playing" }
+                    
+                    div { class: "card game-mode-card mode-singleplayer",
+                        div { class: "mode-icon", "👤" }
+                        h3 { "Singleplayer" }
+                        p { "Challenge yourself and master the board." }
+                        Link {
+                            class: "btn btn-primary",
+                            to: Route::PlaySingleplayerPage {},
+                            "Play Singleplayer →"
+                        }
+                    }
+                    
+                    div { class: "card game-mode-card mode-matchmaking",
+                        div { class: "mode-icon", "⚔️" }
+                        h3 { "1v1 Matchmaking" }
+                        p { "Compete against real players in real time." }
+                        Link {
+                            class: "btn btn-secondary",
+                            to: Route::MatchmakingPage {},
+                            "Find Match →"
+                        }
+                    }
+                    
+                    div { class: "card mt-2 flex items-center gap-1",
+                        div { class: "online-indicator", "• {online_count} players online" }
+                        span { class: "text-muted", style: "font-size: 0.8rem;", "Active community" }
+                    }
+                }
+                
+                // CENTER COLUMN: Last Game / Live Preview
+                div { class: "main-content",
+                    div { class: "card game-preview-card",
+                        div { class: "card-header",
+                            div { class: "card-title", 
+                                "🕒 Last Game"
+                            }
+                            div { class: "status-badge status-victory", "Victory" }
+                        }
+                        
+                        div { class: "game-display-container",
+                            // Left Stats
+                            div { class: "game-stats-list",
+                                div { class: "game-stat-entry",
+                                    span { class: "game-stat-label", "Score" }
+                                    span { class: "game-stat-value", "920" }
+                                }
+                                div { class: "game-stat-entry",
+                                    span { class: "game-stat-label", "Lines" }
+                                    span { class: "game-stat-value", "12" }
+                                }
+                                div { class: "game-stat-entry",
+                                    span { class: "game-stat-label", "Moves" }
+                                    span { class: "game-stat-value", "153" }
+                                }
+                            }
+                            
+                            // Center Board
+                            div { class: "flex flex-col items-center",
+                                BotPlayer { game_state }
+                                GameDisplay { game_state }
+                            }
+                            
+                            // Right Stats (Next/Hold)
+                            div { class: "game-stats-list",
+                                div { class: "game-stat-entry",
+                                    span { class: "game-stat-label", "Time" }
+                                    span { class: "game-stat-value", "63.75s" }
+                                }
+                                div { class: "game-stat-entry",
+                                    span { class: "game-stat-label", "Combo" }
+                                    span { class: "game-stat-value text-victory", "-1" }
+                                }
+                            }
+                        }
+                        
+                        div { class: "mt-2 text-center",
+                            Link {
+                                class: "btn btn-outline",
+                                to: Route::UsersRootDirectoryPage {},
+                                "View Full Game Stats →"
+                            }
+                        }
+                    }
+                }
+                
+                // RIGHT COLUMN: Chat & Top Players
+                div { class: "sidebar-right",
+                    
+                    // Live Chat Section
+                    div { class: "card sidebar-section",
+                        div { class: "card-title mb-2", "💬 Live Chat" }
+                        div { class: "chat-message",
+                            div { class: "chat-avatar", style: "background: #4caf50;" }
+                            div { class: "chat-content",
+                                span { class: "user", "PlayerOne" }
+                                span { class: "time", "2m ago" }
+                                p { class: "mb-0", "GG!" }
+                            }
+                        }
+                        div { class: "chat-message",
+                            div { class: "chat-avatar", style: "background: #2196f3;" }
+                            div { class: "chat-content",
+                                span { class: "user", "TetrisMaster" }
+                                span { class: "time", "5m ago" }
+                                p { class: "mb-0", "Nice combo!" }
+                            }
+                        }
+                        Link {
+                            class: "btn btn-outline mt-1",
+                            to: Route::GlobalChatPage {},
+                            "Open Chat"
+                        }
+                    }
+                    
+                    // Top Players Section
+                    div { class: "card sidebar-section",
+                        div { class: "card-title mb-2", "🏆 Top Players" }
+                        // Placeholder for top players list
+                        div { class: "flex flex-col gap-1",
+                            for (i, name) in ["TetrisMaster", "BlockKing", "SpeedDemon"].iter().enumerate() {
+                                div { class: "flex justify-between items-center",
+                                    span { "{i+1}. {name}" }
+                                    span { class: "text-victory", "{2847 - (i as i32 * 500)}" }
+                                }
+                            }
+                        }
+                        Link {
+                            class: "btn btn-outline mt-1",
+                            to: Route::UsersRootDirectoryPage {},
+                            "View Leaderboard"
+                        }
+                    }
+                }
+
+            }
+            
+            footer {
+                p { "© 2024 Sparganothis. Built with Rust 🦀" }
+            }
         }
     }
 }
+
diff --git a/docs/Screenshot 2026-04-28 203939.png b/docs/Screenshot 2026-04-28 203939.png
new file mode 100644
index 0000000..1d33449
Binary files /dev/null and b/docs/Screenshot 2026-04-28 203939.png differ
diff --git a/docs/Screenshot 2026-04-28 205111.png b/docs/Screenshot 2026-04-28 205111.png
new file mode 100644
index 0000000..7f54c85
Binary files /dev/null and b/docs/Screenshot 2026-04-28 205111.png differ
