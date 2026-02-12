# Tuisteroids

A classic Asteroids arcade game rendered entirely in the terminal using Unicode braille characters. Built in Rust with ratatui.

## Features

- Vector-style graphics using braille character rasterization
- Full arcade gameplay: ship control, shooting, asteroid splitting, wave progression
- Toroidal world (objects wrap at screen edges)
- Attract mode with AI-controlled demo on the menu screen
- Procedurally generated sound effects (no external audio files)
- 60 FPS fixed-timestep game loop
- Graceful audio degradation for headless/SSH sessions

## Controls

| Key | Action |
|-----|--------|
| Left / Right Arrow | Rotate ship |
| Up Arrow | Thrust |
| Space | Fire |
| Enter | Start game (from menu) |
| Q | Quit |

## Scoring

| Asteroid Size | Points |
|---------------|--------|
| Large | 20 |
| Medium | 50 |
| Small | 100 |

Extra life awarded at 10,000 points. You start with 3 lives.

## Building & Running

Requires [Rust](https://www.rust-lang.org/tools/install) (2021 edition).

```bash
# Build
cargo build --release

# Run
cargo run --release
```

## Testing

The project enforces spec-first TDD with 100% code coverage.

```bash
# Run tests
cargo test --lib

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --lib --timeout 120
```

## Dependencies

- [ratatui](https://github.com/ratatui/ratatui) — Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — Cross-platform terminal manipulation
- [rand](https://github.com/rust-random/rand) — Random number generation
- [rodio](https://github.com/RustAudio/rodio) — Audio playback

## Project Structure

```
src/
  main.rs        Entry point
  game.rs        Game loop, state machine, wave progression
  renderer.rs    Braille rasterization, HUD, menus
  ship.rs        Player ship physics and control
  asteroids.rs   Asteroid types, spawning, splitting
  bullets.rs     Projectile pool and lifetime
  collision.rs   Toroidal distance, circle-circle detection
  physics.rs     2D vector math, integration, wrapping
  input.rs       Keyboard polling and action mapping
  demo_ai.rs     AI controller for attract mode
  audio.rs       Procedural sound synthesis
```

## Development

This project uses [OpenSpec](https://github.com/openspec) for specification management and enforces a strict spec-first TDD workflow. See [CLAUDE.md](CLAUDE.md) for details.

## License

See [LICENSE](LICENSE) for details.
