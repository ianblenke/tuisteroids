# Change: Add optional audio sound effects

## Why
The game currently has no audio feedback. Classic Asteroids (1979) had distinctive sound effects that provided critical gameplay feedback — thrust rumble, laser fire, explosion booms, and the iconic heartbeat. Adding procedurally-generated sound effects enhances the arcade feel while keeping the binary self-contained (no external audio files). Audio must degrade gracefully when no audio device is available (e.g. SSH sessions, headless servers).

## What Changes
- Add `audio` capability (new): AudioEngine with optional rodio output stream, procedural sound synthesis, and an event-driven playback API
- Modify `game-loop` capability: PlayingState::update() returns audio events alongside state transitions; run() initializes AudioEngine and dispatches events each frame

## Impact
- Affected specs: audio (new), game-loop (modified)
- Affected code: src/audio.rs (new), src/game.rs (modified), src/lib.rs (modified), Cargo.toml (add rodio dependency)
