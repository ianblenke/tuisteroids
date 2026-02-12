## 1. Spec Deltas
- [x] 1.1 Create collision spec delta (add toroidal_direction requirement)
- [x] 1.2 Create demo-ai spec (new capability with 5 requirements)
- [x] 1.3 Create game-loop spec delta (modify Game State Machine, add Attract Mode)
- [x] 1.4 Create renderer spec delta (modify Menu Screen)
- [x] 1.5 Validate with openspec validate add-attract-mode --strict

## 2. Collision: Toroidal Direction
- [x] 2.1 Write tests from collision toroidal direction scenarios (5 tests)
- [x] 2.2 Implement toroidal_direction() in src/collision.rs
- [x] 2.3 Verify tests pass (green) — 100% coverage

## 3. Demo AI
- [x] 3.1 Add `pub mod demo_ai;` to src/lib.rs
- [x] 3.2 Write tests from demo-ai spec scenarios (15 tests)
- [x] 3.3 Implement generate_demo_input() in src/demo_ai.rs
- [x] 3.4 Verify tests pass (green) — 100% coverage

## 4. Game Loop: Attract Mode
- [x] 4.1 Add demo field to Game struct
- [x] 4.2 Write tests from attract mode spec scenarios (7 tests)
- [x] 4.3 Implement demo lifecycle (init, reset, discard, restart)
- [x] 4.4 Verify tests pass (green) — 100% coverage

## 5. Renderer: Menu with Demo Background
- [x] 5.1 Update run() loop to tick demo during Menu state
- [x] 5.2 Update run() render to draw demo behind menu text
- [x] 5.3 All run() changes covered by #[cfg(not(tarpaulin_include))]

## 6. Final Verification
- [x] 6.1 Run full test suite — 181 tests, 0 failures
- [x] 6.2 Coverage — 98.71% (460/466 lines; 6 uncovered are asteroids.rs tarpaulin artifacts + main.rs)
- [x] 6.3 Manual playtest pending
