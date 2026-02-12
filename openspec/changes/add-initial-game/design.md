## Context

Tuisteroids is a new TUI Asteroids clone. No existing codebase — greenfield Rust project. The game must render vector-like graphics in a terminal at 60 FPS using Unicode characters.

Stakeholders: Solo developer. No external API consumers. The terminal is the only interface.

## Goals / Non-Goals

- **Goals:**
  - Playable Asteroids clone in the terminal
  - Smooth 60 FPS gameplay hiding Unicode rendering imperfections
  - Classic Asteroids mechanics (lives, scoring, waves, splitting)
  - 100% test coverage with spec-driven TDD
  - Clean module separation for independent testability

- **Non-Goals:**
  - Networked multiplayer
  - Sound/audio
  - Persistent high scores (can be added later)
  - Mouse input
  - Color/theming (monochrome vector aesthetic)

## Decisions

### Game Loop: Fixed Timestep with Accumulator
- **Decision**: Use a fixed timestep of 1/60s (~16.67ms) with an accumulator pattern
- **Why**: Deterministic physics regardless of render speed. Essential for testable game logic — tests can step the simulation by exact dt values.
- **Alternative**: Variable timestep — rejected because physics becomes non-deterministic and harder to test

### Rendering: Braille Character Rasterization
- **Decision**: Use Unicode Braille characters (U+2800–U+28FF) for sub-cell resolution line drawing
- **Why**: Each terminal cell maps to a 2×4 dot grid, giving 2x horizontal and 4x vertical resolution over block characters. Bresenham's line algorithm maps world-space line segments to braille dot positions. At 60 FPS, imperfections are imperceptible.
- **Alternative**: Box-drawing characters — rejected, too coarse for rotated shapes
- **Alternative**: Half-block characters (▀▄) — only 2x vertical resolution, insufficient for diagonal lines

### Entity Model: Simple Structs
- **Decision**: Use plain Rust structs with a shared trait for common entity behavior
- **Why**: Only ~4 entity types. ECS adds complexity without benefit at this scale.
- **Alternative**: ECS (specs/hecs/bevy_ecs) — rejected as over-engineered for <100 entities

### Collision: Circle-Circle with Toroidal Wrapping
- **Decision**: Each entity has a bounding radius. Collision = distance between centers < sum of radii, using shortest toroidal distance.
- **Why**: Simple, fast, sufficient for Asteroids. Polygon-precise collision is unnecessary when shapes are approximated anyway.
- **Alternative**: SAT polygon collision — rejected, overkill for arcade game

### State Machine: Enum-Based
- **Decision**: `GameState` enum with `Menu`, `Playing`, `GameOver` variants. Each variant holds its own state.
- **Why**: Simple, exhaustive match ensures all states handled. No dynamic dispatch overhead.

### Testability: Trait Boundaries for I/O
- **Decision**: Input and rendering go behind traits. Game logic operates on abstract input events and produces render commands. Tests inject mock input and verify render output without a real terminal.
- **Why**: Enables 100% code coverage. Game logic is pure computation.

## Risks / Trade-offs

- **Braille rendering performance** → Mitigated by only redrawing changed cells (dirty tracking) and ratatui's built-in diffing
- **Terminal compatibility** → Braille characters require Unicode-capable terminal. Most modern terminals support this. Document requirement.
- **60 FPS in terminal** → Some terminals may not keep up with rapid redraws. Mitigated by ratatui's efficient diffing and crossterm's buffered output.

## Migration Plan
N/A — greenfield project.

## Open Questions
- None — all architectural decisions resolved.
