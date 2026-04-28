# CSS Inline Switch: Modernization Plan

This document identifies all inline `style:` references within Dioxus components and provides a plan for migrating them to the unified CSS system in `main.css`.

## Goal
Remove all ad-hoc inline styles and replace them with semantic, reusable CSS classes that adhere to the premium dark theme established in the landing page.

## Style Audit Results

| File Path | Line | Inline Style Snippet | Replacement Strategy |
|-----------|------|----------------------|----------------------|
| `comp/game_display.rs` | 203+ | Multiple layout styles | Use `.game-preview-board` class |
| `comp/game_display.rs` | 370 | `color: #ffd700;` (Gold) | Use `.text-gold` or `--color-status-special` |
| `comp/game_display.rs` | 376 | `color: #ff69b4;` (Pink) | Use `.text-tspin` or `--color-status-tspin` |
| `comp/nav.rs` | 61 | `display: flex; align-items: center; gap: 0.5rem;` | Use `.nav-link-flex` (already in `main.css` for some parts) |
| `comp/chat/chat_display.rs` | 105 | `width: calc(90%-30px); color: {color};` | Use `.chat-bubble` class |
| `comp/chat/chat_display.rs` | 247 | `justify-content: {align};` | Use utility classes `.chat-align-left/right` |
| `comp/multiplayer/_1v1.rs` | 81 | `color: {node.html_color()}` | Keep dynamic colors but move other styles to `.player-node` |
| `pages/homepage.rs` | 29 | `font-size: 2rem; margin-bottom: 0.5rem;` | Use `.hero-title` or standard `h2` overrides |
| `pages/homepage.rs` | 88 | `display: flex; flex-direction: column; align-items: center;` | Use `.center-column` utility |
| `pages/homepage.rs` | 123 | `background: #4caf50;` | Use `.avatar-green` or CSS variables |
| `pages/singleplayer.rs` | 10 | `height: 80dvh; display: flex;` | Use `.full-page-container` |
| `pages/play_game/private_lobby.rs` | 89 | `flex-direction:row height: 80px;` | Use `.lobby-header` |

## Migration Plan

### Phase 1: Utility Class Definition
Add missing utility classes to `client/assets/main.css`:
- `.flex-center`, `.flex-row`, `.flex-col`
- `.mt-1`, `.mb-2`, etc.
- `.text-gold`, `.text-pink`, `.text-red`
- `.full-height`, `.h-80`

### Phase 2: Component-Specific Classes
Define semantic classes for complex components:
- `.chat-container`, `.chat-message`, `.chat-bubble`
- `.game-board-container`, `.game-stats-grid`
- `.settings-form-layout`

### Phase 3: Replacement
Iterate through the files listed above and replace `style: "..."` with `class: "..."`.

## Detailed Task List

- [ ] Define shared utility variables for colors (Gold, Pink, etc.) in `:root`.
- [ ] Implement `.chat-message` and `.chat-bubble` in `main.css`.
- [ ] Replace inline styles in `comp/chat/chat_display.rs`.
- [ ] Implement `.game-board-wrapper` and replace styles in `comp/game_display.rs`.
- [ ] Refactor `homepage.rs` to remove the last few ad-hoc styles added during the redesign.
- [ ] Standardize avatar colors using CSS variables.
