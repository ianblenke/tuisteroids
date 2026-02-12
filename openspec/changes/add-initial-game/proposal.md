# Change: Add Initial Tuisteroids Game

## Why
Tuisteroids has no game yet — it's a blank project. We need to define the full initial game: a TUI-based Asteroids clone rendered with Unicode braille characters in a terminal, playable at 60 FPS with classic Asteroids mechanics.

## What Changes
- Add `game-loop` capability: fixed-timestep game loop with state machine (Menu/Playing/GameOver)
- Add `physics` capability: 2D vector math, motion integration, toroidal wrapping
- Add `input` capability: keyboard input handling (arrow keys, spacebar, q to quit)
- Add `ship` capability: player ship with rotation, thrust, inertia, lives, respawn
- Add `asteroids` capability: three asteroid sizes, splitting, wave progression, scoring
- Add `bullets` capability: projectiles fired from ship with lifetime and screen limit
- Add `collision` capability: circle-circle collision detection with toroidal wrapping
- Add `renderer` capability: braille-based Unicode vector rendering, HUD, menus

## Impact
- Affected specs: All new — game-loop, physics, input, ship, asteroids, bullets, collision, renderer
- Affected code: Entire `src/` directory (new Rust project)
- This is the foundational proposal that establishes the complete game
