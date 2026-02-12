## 1. Spec Deltas
- [ ] 1.1 Create audio spec (new capability with 5 requirements)
- [ ] 1.2 Create game-loop spec delta (modify Game Update Sequence for audio events)
- [ ] 1.3 Create design.md with architectural decisions
- [ ] 1.4 Validate with openspec validate add-audio --strict

## 2. Audio Event Types & UpdateResult
- [ ] 2.1 Add `rodio` dependency to Cargo.toml
- [ ] 2.2 Add `pub mod audio;` to src/lib.rs
- [ ] 2.3 Write tests for AudioEvent enum (variants are distinct, all constructable)
- [ ] 2.4 Write tests for UpdateResult struct (state + audio_events fields)
- [ ] 2.5 Implement AudioEvent enum and UpdateResult struct in src/audio.rs
- [ ] 2.6 Verify tests pass — 100% coverage of new types

## 3. Game Loop: Audio Event Emission
- [ ] 3.1 Write tests for update() returning AudioEvent::Fire when firing
- [ ] 3.2 Write tests for update() returning AudioEvent::Thrust when thrusting
- [ ] 3.3 Write tests for update() returning AudioEvent::AsteroidExplosionLarge/Medium/Small on hit
- [ ] 3.4 Write tests for update() returning AudioEvent::ShipDestroyed on ship collision
- [ ] 3.5 Write tests for update() returning AudioEvent::ExtraLife on extra life
- [ ] 3.6 Write tests for update() returning AudioEvent::NewWave on wave spawn
- [ ] 3.7 Write test for empty audio_events when nothing happens
- [ ] 3.8 Change PlayingState::update() return type from Option<GameState> to UpdateResult
- [ ] 3.9 Add audio event emission at each relevant point in update()
- [ ] 3.10 Update all existing tests that check update() return value to use UpdateResult
- [ ] 3.11 Verify all tests pass — 100% coverage

## 4. Audio Engine: Initialization & Playback
- [ ] 4.1 Write tests for AudioEngine::try_new() (silent mode when no device)
- [ ] 4.2 Write tests for AudioEngine::play() as no-op in silent mode
- [ ] 4.3 Implement AudioEngine struct with Option<(OutputStream, OutputStreamHandle)>
- [ ] 4.4 Implement try_new() wrapping rodio::OutputStream::try_default()
- [ ] 4.5 Implement play() method dispatching AudioEvent to synthesized sounds
- [ ] 4.6 Verify tests pass — 100% coverage of AudioEngine (silent path)

## 5. Procedural Sound Synthesis
- [ ] 5.1 Implement fire sound (short high-frequency burst)
- [ ] 5.2 Implement thrust sound (low-frequency noise burst)
- [ ] 5.3 Implement asteroid explosion sounds (3 sizes: long/low to short/high)
- [ ] 5.4 Implement ship destroyed sound (dramatic explosion)
- [ ] 5.5 Implement extra life sound (ascending sweep)
- [ ] 5.6 Implement new wave sound (alert tone)
- [ ] 5.7 Mark synthesis functions with #[cfg(not(tarpaulin_include))] if they require audio hardware

## 6. Game Loop Integration
- [ ] 6.1 Initialize AudioEngine in run() with try_new()
- [ ] 6.2 Collect UpdateResult from PlayingState::update() in run() loop
- [ ] 6.3 Dispatch audio_events to AudioEngine::play() each frame
- [ ] 6.4 All run() changes covered by #[cfg(not(tarpaulin_include))]

## 7. Final Verification
- [ ] 7.1 Run full test suite — all tests pass, 0 failures
- [ ] 7.2 Coverage — maintain existing coverage level (all new testable code covered)
- [ ] 7.3 Manual playtest — verify sounds play on local machine
- [ ] 7.4 SSH test — verify game runs silently without errors over SSH
