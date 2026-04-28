# CSS Switch: Theme Enhancement

This document tracks the changes made to `client/assets/main.css` to match the target premium dark design.

## Proposed Changes

### Global Theme
- Set background to a deep navy/black (`#0b0e14`).
- Update text colors for high contrast on dark background.

### Navigation Bar
- Glassmorphism effect with blur and dark semi-transparent background.
- Subtle bottom border.

### Cards (Game Modes, Preview)
- Dark card backgrounds (`#161b22`) with refined borders (`#30363d`).
- Hover effects with glow/elevation.
- Modern typography and spacing.

### Buttons
- Gradient backgrounds or solid vibrant colors.
- Rounded corners and smooth transitions.

## CSS Code Comparison

### Before (Basic Dark)
```css
.global_parent {
  background-color: var(--pico-background-color);
  color: var(--pico-color);
}
nav {
  background: rgba(0, 0, 0, 0.8) !important;
}
```

### After (Premium Dark)
```css
:root {
  --pico-background-color: #0b0e14;
  --pico-card-background-color: #161b22;
  --pico-color: #f0f6fc;
  --pico-muted-color: #8b949e;
  --pico-primary: #58a6ff;
}

.global_parent {
  background-color: #0b0e14;
  background-image: radial-gradient(circle at top right, rgba(88, 166, 255, 0.05), transparent 400px),
                    radial-gradient(circle at bottom left, rgba(123, 97, 255, 0.05), transparent 400px);
}
```

## Status: Applied
The CSS has been overhauled to match the target design. The homepage is being updated to use the new dashboard layout classes.
