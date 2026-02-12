# Project Context

## Purpose
Tuisteroids - a TUI-based asteroids game.

## Tech Stack
- **Language**: Rust (2021 edition)
- **TUI Rendering**: `ratatui` — terminal UI framework with widget system
- **Terminal Backend**: `crossterm` — cross-platform terminal manipulation and input
- **RNG**: `rand` crate — random number generation for asteroid shapes and spawns
- **Testing**: `cargo test` with `cargo-tarpaulin` for code coverage
- **Target Framerate**: 60 FPS (fixed timestep: ~16.67ms per tick)

## Project Conventions

### Development Methodology: Spec-First TDD

This project follows a strict spec-first, test-driven development workflow. **No code is written without a spec, and no code is written without a failing test.**

#### The Workflow (in order)
1. **Write specs** - All requirements MUST be written as OpenSpec specifications first
2. **Generate tests from specs** - Every `#### Scenario:` in a spec MUST produce at least one test case. 100% spec coverage is required — no scenario left untested
3. **Run tests (they MUST fail)** - Tests are run to confirm they fail before any implementation exists
4. **Write code to pass tests** - Implementation is driven solely by making failing tests pass
5. **Verify 100% code coverage** - All generated tests MUST fully cover the implementation. No untested code paths

#### Coverage Rules
- **100% spec coverage**: Every requirement and scenario in specs MUST have corresponding test(s)
- **100% code coverage**: Every line/branch of implementation code MUST be exercised by tests
- Coverage reports MUST be generated and checked as part of the workflow
- PRs MUST NOT be merged if coverage drops below 100%

#### What This Means in Practice
- Never write implementation code without a spec backing it
- Never write a test without a spec scenario driving it
- Never ship code that isn't fully covered by tests
- If a bug is found, write a spec scenario for it first, then a failing test, then fix

### Code Style
- Follow standard `rustfmt` formatting
- Use `clippy` with default lints
- Prefer explicit types on public APIs, infer on internals
- Module-per-file organization

### Architecture Patterns
- **Game loop**: Fixed-timestep accumulator pattern (update at 60Hz, render as fast as possible)
- **Entity model**: Simple structs (no ECS) — Ship, Asteroid, Bullet, Debris
- **Rendering**: Braille character rasterization for vector-like line drawing
- **Collision**: Circle-circle with toroidal distance
- **State machine**: Menu → Playing → GameOver → Menu
- **Testability**: All game logic is pure (no I/O); rendering and input behind trait boundaries

### Testing Strategy
- **Framework**: `cargo test` (built-in Rust test framework)
- **Coverage**: `cargo-tarpaulin` with 100% line coverage threshold
- **Test organization**: Unit tests in each module (`#[cfg(test)] mod tests`), integration tests in `tests/`
- Tests MUST be generated from spec scenarios
- Tests MUST achieve 100% code coverage
- Tests MUST achieve 100% spec coverage (every scenario tested)
- Coverage tooling MUST be configured to enforce thresholds

### Git Workflow
- All changes go through OpenSpec proposal process
- PRs require passing tests with full coverage
- Spec changes require proposal approval before implementation

## Domain Context
Tuisteroids is an asteroids-style arcade game rendered in a terminal user interface (TUI).

## Important Constraints
- Specs drive everything — no cowboy coding
- 100% spec coverage for tests (every scenario has a test)
- 100% code coverage for implementation (every code path is tested)
- TDD is mandatory: red-green-refactor cycle

## External Dependencies
- `ratatui` — TUI rendering framework
- `crossterm` — terminal backend for ratatui, keyboard input
- `rand` — random number generation
- `cargo-tarpaulin` — code coverage (dev dependency / CI tool)
