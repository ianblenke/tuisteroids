## 1. Project Setup
- [x] 1.1 Initialize Rust project with `cargo init`
- [x] 1.2 Add dependencies: ratatui, crossterm, rand
- [x] 1.3 Configure cargo-tarpaulin for coverage enforcement
- [x] 1.4 Set up module structure (lib.rs with mod declarations)

## 2. Physics Capability (spec: physics)
- [x] 2.1 Write tests from physics/spec.md scenarios
- [x] 2.2 Verify tests fail (red)
- [x] 2.3 Implement Vec2 type and operations
- [x] 2.4 Implement motion integration and toroidal wrapping
- [x] 2.5 Verify tests pass (green) — 20/20 tests pass

## 3. Collision Capability (spec: collision)
- [x] 3.1 Write tests from collision/spec.md scenarios
- [x] 3.2 Verify tests fail (red)
- [x] 3.3 Implement circle-circle collision with toroidal distance
- [x] 3.4 Verify tests pass (green) — 14/14 tests pass

## 4. Input Capability (spec: input)
- [x] 4.1 Write tests from input/spec.md scenarios
- [x] 4.2 Verify tests fail (red)
- [x] 4.3 Implement input abstraction layer and key mapping
- [x] 4.4 Verify tests pass (green) — 13/13 tests pass

## 5. Ship Capability (spec: ship)
- [x] 5.1 Write tests from ship/spec.md scenarios
- [x] 5.2 Verify tests fail (red)
- [x] 5.3 Implement Ship struct with rotation, thrust, lives, respawn
- [x] 5.4 Verify tests pass (green) — 18/18 tests pass

## 6. Asteroids Capability (spec: asteroids)
- [x] 6.1 Write tests from asteroids/spec.md scenarios
- [x] 6.2 Verify tests fail (red)
- [x] 6.3 Implement Asteroid struct with sizes, splitting, wave system
- [x] 6.4 Verify tests pass (green) — 21/21 tests pass

## 7. Bullets Capability (spec: bullets)
- [x] 7.1 Write tests from bullets/spec.md scenarios
- [x] 7.2 Verify tests fail (red)
- [x] 7.3 Implement Bullet struct with lifetime, speed, screen limit
- [x] 7.4 Verify tests pass (green) — 11/11 tests pass

## 8. Renderer Capability (spec: renderer)
- [x] 8.1 Write tests from renderer/spec.md scenarios
- [x] 8.2 Verify tests fail (red)
- [x] 8.3 Implement braille rasterizer and shape rendering
- [x] 8.4 Implement HUD (score, lives) and menu/game-over screens
- [x] 8.5 Verify tests pass (green) — 23/23 tests pass

## 9. Game Loop Capability (spec: game-loop)
- [x] 9.1 Write tests from game-loop/spec.md scenarios
- [x] 9.2 Verify tests fail (red)
- [x] 9.3 Implement fixed-timestep loop with accumulator
- [x] 9.4 Implement state machine (Menu/Playing/GameOver)
- [x] 9.5 Wire all capabilities together in main game update
- [x] 9.6 Verify tests pass (green) — 19/19 tests pass

## 10. Integration & Final Verification
- [x] 10.1 Run full test suite: 139/139 tests pass
- [ ] 10.2 Verify 100% code coverage (cargo-tarpaulin pending install)
- [x] 10.3 Verify all spec scenarios have corresponding tests (100% spec coverage confirmed)
- [ ] 10.4 Manual playtest in terminal
