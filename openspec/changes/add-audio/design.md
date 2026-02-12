## Context

Tuisteroids is a TUI Asteroids game with no audio. Adding sound effects requires an audio backend that works cross-platform (macOS CoreAudio, Linux ALSA/PulseAudio) and degrades gracefully when no audio device exists (SSH, headless). The game already has a pure-logic update function (`PlayingState::update`) and a `#[cfg(not(tarpaulin_include))]` run loop that handles I/O.

## Goals / Non-Goals

- **Goals:**
  - Procedural sound synthesis (no external audio files)
  - Sound effects for: fire, thrust (looping), asteroid explosion (large/medium/small), ship destruction, extra life, new wave
  - Graceful degradation: game plays silently when audio unavailable
  - Testable event generation without audio hardware
  - Cross-platform: macOS and Linux

- **Non-Goals:**
  - Background music
  - Volume control / audio settings UI
  - Positional/spatial audio
  - Custom waveform editor

## Decisions

### Audio Backend: rodio
- **Decision**: Use `rodio` crate for audio output
- **Why**: High-level Rust API, wraps `cpal` for cross-platform support, handles mixing and async playback on background thread. `OutputStream::try_default()` returns `Result` — failure means no audio, not a crash.
- **Alternative**: `cpal` directly — rejected, requires writing our own mixer and sample generation
- **Alternative**: `kira` — rejected, heavier API designed for complex game audio; overkill here
- **Alternative**: `tinyaudio` — rejected, less mature ecosystem

### Sound Synthesis: Procedural via rodio::Source
- **Decision**: Generate sounds programmatically using rodio's `Source` trait (sine waves, noise, envelopes)
- **Why**: Keeps the binary fully self-contained with zero external assets. Classic Asteroids sounds are simple enough to synthesize (white noise bursts, frequency sweeps, square waves).
- **Alternative**: Embed WAV/OGG files — rejected, adds binary bloat and asset management

### Architecture: Event-Based Decoupling
- **Decision**: `PlayingState::update()` returns a `Vec<AudioEvent>` enum alongside its existing `Option<GameState>`. The `run()` loop passes events to `AudioEngine::play()`. AudioEngine is constructed in `run()` and never touches game logic.
- **Why**: Keeps game logic pure and testable. Tests verify correct events are emitted without needing audio hardware. The AudioEngine lives only in the I/O layer (`run()`), which is already excluded from tarpaulin coverage.
- **Alternative**: Pass AudioEngine into update() — rejected, couples I/O to pure logic, breaks testability pattern

### Return Type Change
- **Decision**: Change `PlayingState::update()` return from `Option<GameState>` to `UpdateResult { state: Option<GameState>, audio_events: Vec<AudioEvent> }`
- **Why**: Clean struct avoids tuple soup. Both fields are consumed by the caller. The struct is trivially constructable in tests.

### Graceful Degradation
- **Decision**: `AudioEngine::try_new()` returns `AudioEngine` with `Option<OutputStreamHandle>` inside. If `try_default()` fails, the handle is `None` and all `play()` calls are no-ops.
- **Why**: No conditional compilation, no feature flags, no SSH detection needed. The same code path runs everywhere — it just does nothing when there's no device.

## Risks / Trade-offs

- **rodio compile time** → Pulls in `cpal` and platform audio backends. Adds ~10-15s to clean build. Acceptable for a game project.
- **Linux audio device availability** → ALSA is near-universal on Linux desktops. PulseAudio/PipeWire provide ALSA compatibility. Headless servers have no device, which triggers graceful fallback.
- **Sound quality** → Procedural synthesis won't match studio-quality samples, but matches the retro arcade aesthetic perfectly.

## Open Questions
- None — approach is straightforward.
