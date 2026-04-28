# UI Enhancement Plan for Sparganothis-v2

## Overview
Create a modern, engaging landing page and improve overall UI with clear CTAs for game modes, online player count display, and better visual design using CSS.

## Current State Analysis
- **Framework**: Dioxus 0.6.3 with Pico CSS + custom main.css
- **Homepage**: Basic bot player game display (`homepage.rs`)
- **Navigation**: Simple nav bar with links to Singleplayer, 1v1 Matchmaking, Chat, Top Players
- **API Available**: 
  - `GetReplayMatchList` - fetch replays for "previous game" display
  - `chat_presence` - get online player count via presence list
  - `GetUsersWithTopGameCounts` - top players data

## Implementation Plan

### 1. CSS Enhancements (`client/assets/main.css`)

Add the following styles:

```css
/* ===== LANDING PAGE STYLES ===== */

/* Hero section with game preview */
.hero-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 70dvh;
  padding: 2rem;
  gap: 2rem;
}

/* Game preview container */
.game-preview {
  width: 100%;
  max-width: 600px;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  background: var(--pico-card-background-color, #1a1a1a);
}

.game-preview-header {
  padding: 0.75rem 1rem;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  font-size: 0.9rem;
  color: var(--pico-muted-color);
}

/* CTA Buttons section */
.cta-section {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  justify-content: center;
  padding: 1rem 0;
}

.cta-button {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem 2rem;
  border-radius: 8px;
  font-size: 1.1rem;
  font-weight: 600;
  text-decoration: none;
  transition: all 0.2s ease;
  border: 2px solid transparent;
  cursor: pointer;
}

.cta-button-primary {
  background: var(--pico-primary, #4a90d9);
  color: white;
}

.cta-button-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(74, 144, 217, 0.4);
}

.cta-button-secondary {
  background: transparent;
  color: var(--pico-primary, #4a90d9);
  border-color: var(--pico-primary, #4a90d9);
}

.cta-button-secondary:hover {
  background: rgba(74, 144, 217, 0.1);
  transform: translateY(-2px);
}

/* Stats bar */
.stats-bar {
  display: flex;
  justify-content: center;
  gap: 2rem;
  padding: 1rem;
  flex-wrap: wrap;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--pico-primary, #4a90d9);
}

.stat-label {
  font-size: 0.85rem;
  color: var(--pico-muted-color);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Quick links section */
.quick-links {
  display: flex;
  justify-content: center;
  gap: 2rem;
  padding: 1.5rem;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  flex-wrap: wrap;
}

.quick-link {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--pico-color);
  text-decoration: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  transition: background 0.2s ease;
}

.quick-link:hover {
  background: rgba(255, 255, 255, 0.05);
}

/* Game mode cards */
.game-modes {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1.5rem;
  padding: 2rem 0;
  width: 100%;
  max-width: 800px;
}

.game-mode-card {
  background: var(--pico-card-background-color, #1a1a1a);
  border-radius: 12px;
  padding: 1.5rem;
  border: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.2s ease;
}

.game-mode-card:hover {
  border-color: var(--pico-primary, #4a90d9);
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.game-mode-card h3 {
  margin: 0 0 0.5rem 0;
  color: var(--pico-color);
}

.game-mode-card p {
  color: var(--pico-muted-color);
  margin: 0 0 1rem 0;
  font-size: 0.9rem;
}

/* Nav improvements */
nav {
  backdrop-filter: blur(10px);
  background: rgba(0, 0, 0, 0.8) !important;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

nav a {
  transition: color 0.2s ease;
}

nav a:hover {
  color: var(--pico-primary, #4a90d9) !important;
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .hero-section {
    padding: 1rem;
  }
  
  .cta-section {
    flex-direction: column;
    align-items: stretch;
  }
  
  .cta-button {
    justify-content: center;
  }
  
  .stats-bar {
    gap: 1rem;
  }
}
```

### 2. Homepage Component Updates (`client/src/pages/homepage.rs`)

Transform the homepage to include:

1. **Game Preview Section**: Display a replay or bot game with header "Watch Live" or "Previous Game"
2. **Game Mode Cards**: Visual cards for Singleplayer and 1v1 Matchmaking
3. **Stats Bar**: Show online players count (from chat presence), total matches, etc.
4. **Quick Links**: Prominent links to Chat and Top Players

Key changes:
- Fetch a recent replay using `GetReplayMatchList` API (limit to 1)
- Get online player count from `chat_presence.get_presence_list()`
- Create visually appealing CTA sections
- Use Dioxus components with the new CSS classes

### 3. Online Player Count Implementation

In the homepage or via a shared context:
- Access the global chat's presence list
- Count unique users in the presence list
- Display as "X players online"

Location: Use `NetworkState` -> `global_mm` -> `chat_presence()` to get the count

### 4. Navigation Improvements

Update `client/src/comp/nav.rs`:
- Add icons or better visual indicators for each link
- Highlight current route
- Add online status indicator prominently

## Files to Modify

1. **`client/assets/main.css`** - Add all new CSS styles
2. **`client/src/pages/homepage.rs`** - Complete redesign of landing page
3. **`client/src/comp/nav.rs`** - Optional nav improvements

## Verification

1. Run `dx serve` to test the UI changes
2. Verify all links work (Singleplayer, 1v1, Chat, Top Players)
3. Check responsive design on different viewport sizes
4. Confirm online player count displays correctly
5. Ensure game preview/replay displays properly
