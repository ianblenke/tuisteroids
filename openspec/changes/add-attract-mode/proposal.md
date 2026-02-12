# Change: Add attract mode to menu screen

## Why
The menu screen is static text with no visual interest. Classic Asteroids (1979) featured an attract mode where a demo game played in the background. This adds the same feature to make the menu screen visually engaging.

## What Changes
- Add `demo-ai` capability: AI-controlled ship input for demo gameplay
- Modify `game-loop` capability: Game struct gains demo PlayingState, Menu state ticks demo
- Modify `renderer` capability: Menu screen renders demo game behind overlay text
- Modify `collision` capability: Add toroidal_direction helper (used by AI)

## Impact
- Affected specs: demo-ai (new), game-loop (modified), renderer (modified), collision (modified)
- Affected code: src/demo_ai.rs (new), src/game.rs, src/collision.rs, src/lib.rs
