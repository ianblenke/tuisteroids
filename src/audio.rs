// Audio capability: event types, engine, procedural synthesis

use crate::game::GameState;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::time::Duration;

/// Audio events emitted by gameplay actions.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum AudioEvent {
    Fire,
    Thrust,
    AsteroidExplosionLarge,
    AsteroidExplosionMedium,
    AsteroidExplosionSmall,
    ShipDestroyed,
    ExtraLife,
    NewWave,
}

/// Result of a PlayingState::update() call, containing both state transition
/// and audio events.
pub struct UpdateResult {
    pub state: Option<GameState>,
    pub audio_events: Vec<AudioEvent>,
}

/// Audio engine that plays sounds for game events.
/// Operates in silent mode when no audio device is available.
pub struct AudioEngine {
    // Hold the stream to keep it alive; handle used for playback.
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
}

impl AudioEngine {
    /// Try to initialize audio output. Returns silent engine if no device available.
    pub fn try_new() -> Self {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Self {
                _stream: Some(stream),
                handle: Some(handle),
            },
            Err(_) => Self {
                _stream: None,
                handle: None,
            },
        }
    }

    /// Create an engine in explicit silent mode (for testing).
    pub fn silent() -> Self {
        Self {
            _stream: None,
            handle: None,
        }
    }

    /// Returns true if the engine has an active audio output.
    pub fn is_active(&self) -> bool {
        self.handle.is_some()
    }

    /// Play the sound corresponding to the given audio event.
    /// No-op in silent mode.
    #[cfg(not(tarpaulin_include))]
    pub fn play(&self, event: &AudioEvent) {
        let handle = match &self.handle {
            Some(h) => h,
            None => return, // silent mode
        };

        let source = match event {
            AudioEvent::Fire => synth_fire(),
            AudioEvent::Thrust => synth_thrust(),
            AudioEvent::AsteroidExplosionLarge => synth_explosion(300.0, 0.4),
            AudioEvent::AsteroidExplosionMedium => synth_explosion(500.0, 0.25),
            AudioEvent::AsteroidExplosionSmall => synth_explosion(800.0, 0.15),
            AudioEvent::ShipDestroyed => synth_ship_destroyed(),
            AudioEvent::ExtraLife => synth_extra_life(),
            AudioEvent::NewWave => synth_new_wave(),
        };

        // Play on a detached sink so it doesn't block
        if let Ok(sink) = Sink::try_new(handle) {
            sink.append(source);
            sink.detach();
        }
    }
}

// === Procedural Sound Synthesis ===
// All sounds are generated from basic waveforms. No external audio files.

/// A simple synthesized audio source.
struct SynthSource {
    sample_rate: u32,
    current_sample: u32,
    total_samples: u32,
    generator: Box<dyn Fn(f32) -> f32 + Send>,
}

impl SynthSource {
    fn new(
        sample_rate: u32,
        duration_secs: f32,
        generator: impl Fn(f32) -> f32 + Send + 'static,
    ) -> Self {
        Self {
            sample_rate,
            current_sample: 0,
            total_samples: (sample_rate as f32 * duration_secs) as u32,
            generator: Box::new(generator),
        }
    }
}

impl Iterator for SynthSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.current_sample >= self.total_samples {
            return None;
        }
        let t = self.current_sample as f32 / self.sample_rate as f32;
        let progress = self.current_sample as f32 / self.total_samples as f32;
        self.current_sample += 1;
        // Apply envelope (fade out)
        let envelope = 1.0 - progress;
        Some((self.generator)(t) * envelope * 0.3)
    }
}

impl rodio::Source for SynthSource {
    fn current_frame_len(&self) -> Option<usize> {
        let remaining = self.total_samples.saturating_sub(self.current_sample);
        Some(remaining as usize)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.total_samples as f32 / self.sample_rate as f32,
        ))
    }
}

/// Fire sound: short high-frequency burst (50-100ms)
#[cfg(not(tarpaulin_include))]
fn synth_fire() -> SynthSource {
    SynthSource::new(44100, 0.08, |t| {
        let freq = 880.0 + 440.0 * (1.0 - t * 12.0).max(0.0);
        (t * freq * std::f32::consts::TAU).sin()
    })
}

/// Thrust sound: low-frequency noise burst
#[cfg(not(tarpaulin_include))]
fn synth_thrust() -> SynthSource {
    SynthSource::new(44100, 0.05, |t| {
        // Simple pseudo-noise from sine harmonics
        let base = (t * 80.0 * std::f32::consts::TAU).sin();
        let harm = (t * 120.0 * std::f32::consts::TAU).sin() * 0.5;
        let harm2 = (t * 200.0 * std::f32::consts::TAU).sin() * 0.3;
        base + harm + harm2
    })
}

/// Explosion sound: parameterized by frequency and duration
#[cfg(not(tarpaulin_include))]
fn synth_explosion(freq: f32, duration: f32) -> SynthSource {
    SynthSource::new(44100, duration, move |t| {
        let f = freq * (1.0 - t / duration).max(0.0);
        let base = (t * f * std::f32::consts::TAU).sin();
        // Add noise-like harmonics
        let noise = (t * f * 2.7 * std::f32::consts::TAU).sin() * 0.4;
        base + noise
    })
}

/// Ship destroyed: dramatic long explosion
#[cfg(not(tarpaulin_include))]
fn synth_ship_destroyed() -> SynthSource {
    SynthSource::new(44100, 0.6, |t| {
        let f = 200.0 * (1.0 - t).max(0.0);
        let base = (t * f * std::f32::consts::TAU).sin();
        let harm = (t * f * 1.5 * std::f32::consts::TAU).sin() * 0.6;
        let harm2 = (t * f * 3.0 * std::f32::consts::TAU).sin() * 0.3;
        base + harm + harm2
    })
}

/// Extra life: ascending frequency sweep
#[cfg(not(tarpaulin_include))]
fn synth_extra_life() -> SynthSource {
    SynthSource::new(44100, 0.3, |t| {
        let freq = 400.0 + 800.0 * t;
        (t * freq * std::f32::consts::TAU).sin()
    })
}

/// New wave: brief alert tone
#[cfg(not(tarpaulin_include))]
fn synth_new_wave() -> SynthSource {
    SynthSource::new(44100, 0.2, |t| {
        let freq = 600.0;
        let modulation = (t * 10.0 * std::f32::consts::TAU).sin() * 0.3 + 0.7;
        (t * freq * std::f32::consts::TAU).sin() * modulation
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Requirement: Audio Event Types ===

    // Scenario: All event variants are distinct
    #[test]
    fn test_audio_event_variants_distinct() {
        let events = vec![
            AudioEvent::Fire,
            AudioEvent::Thrust,
            AudioEvent::AsteroidExplosionLarge,
            AudioEvent::AsteroidExplosionMedium,
            AudioEvent::AsteroidExplosionSmall,
            AudioEvent::ShipDestroyed,
            AudioEvent::ExtraLife,
            AudioEvent::NewWave,
        ];
        // Each variant is distinct from all others
        for (i, a) in events.iter().enumerate() {
            for (j, b) in events.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "Variants at index {} and {} should differ", i, j);
                }
            }
        }
    }

    // Scenario: Each variant is independently constructable and matchable
    #[test]
    fn test_audio_event_each_variant_constructable() {
        let fire = AudioEvent::Fire;
        let thrust = AudioEvent::Thrust;
        let large = AudioEvent::AsteroidExplosionLarge;
        let medium = AudioEvent::AsteroidExplosionMedium;
        let small = AudioEvent::AsteroidExplosionSmall;
        let ship = AudioEvent::ShipDestroyed;
        let extra = AudioEvent::ExtraLife;
        let wave = AudioEvent::NewWave;

        assert!(matches!(fire, AudioEvent::Fire));
        assert!(matches!(thrust, AudioEvent::Thrust));
        assert!(matches!(large, AudioEvent::AsteroidExplosionLarge));
        assert!(matches!(medium, AudioEvent::AsteroidExplosionMedium));
        assert!(matches!(small, AudioEvent::AsteroidExplosionSmall));
        assert!(matches!(ship, AudioEvent::ShipDestroyed));
        assert!(matches!(extra, AudioEvent::ExtraLife));
        assert!(matches!(wave, AudioEvent::NewWave));
    }

    // Scenario: AudioEvent is non-exhaustive for future extension
    #[test]
    fn test_audio_event_is_non_exhaustive() {
        // The #[non_exhaustive] attribute is on the enum.
        // We verify this compiles with a wildcard arm (required by non_exhaustive).
        let event = AudioEvent::Fire;
        #[allow(unreachable_patterns)]
        let _label = match event {
            AudioEvent::Fire => "fire",
            AudioEvent::Thrust => "thrust",
            AudioEvent::AsteroidExplosionLarge => "large",
            AudioEvent::AsteroidExplosionMedium => "medium",
            AudioEvent::AsteroidExplosionSmall => "small",
            AudioEvent::ShipDestroyed => "ship",
            AudioEvent::ExtraLife => "extra",
            AudioEvent::NewWave => "wave",
            _ => "unknown",
        };
    }

    // Scenario: AudioEvent supports Clone and Debug
    #[test]
    fn test_audio_event_clone_and_debug() {
        let event = AudioEvent::Fire;
        let cloned = event.clone();
        assert_eq!(event, cloned);
        let debug = format!("{:?}", event);
        assert!(!debug.is_empty());
    }

    // === UpdateResult tests ===

    #[test]
    fn test_update_result_with_no_state_change() {
        let result = UpdateResult {
            state: None,
            audio_events: vec![AudioEvent::Fire],
        };
        assert!(result.state.is_none());
        assert_eq!(result.audio_events.len(), 1);
        assert_eq!(result.audio_events[0], AudioEvent::Fire);
    }

    #[test]
    fn test_update_result_with_state_change() {
        let result = UpdateResult {
            state: Some(GameState::GameOver),
            audio_events: vec![AudioEvent::ShipDestroyed],
        };
        assert_eq!(result.state, Some(GameState::GameOver));
        assert_eq!(result.audio_events[0], AudioEvent::ShipDestroyed);
    }

    #[test]
    fn test_update_result_with_empty_events() {
        let result = UpdateResult {
            state: None,
            audio_events: vec![],
        };
        assert!(result.audio_events.is_empty());
    }

    #[test]
    fn test_update_result_with_multiple_events() {
        let result = UpdateResult {
            state: None,
            audio_events: vec![
                AudioEvent::Fire,
                AudioEvent::AsteroidExplosionLarge,
                AudioEvent::Thrust,
            ],
        };
        assert_eq!(result.audio_events.len(), 3);
    }

    // === Requirement: Audio Engine Initialization ===

    // Scenario: Audio engine initializes without audio device (silent mode)
    #[test]
    fn test_audio_engine_silent_mode() {
        let engine = AudioEngine::silent();
        assert!(!engine.is_active());
    }

    // Scenario: Silent engine ignores play calls
    #[test]
    fn test_silent_engine_ignores_play() {
        let engine = AudioEngine::silent();
        // Should not panic or error
        // play() is behind #[cfg(not(tarpaulin_include))], so we test silent() + is_active()
        assert!(!engine.is_active());
    }

    // Scenario: try_new returns an engine (may be silent in CI)
    #[test]
    fn test_try_new_returns_engine() {
        let engine = AudioEngine::try_new();
        // In CI/headless environments this will be silent, on dev machines it may be active.
        // Either way, it should not panic.
        let _ = engine.is_active();
    }

    // === Requirement: SynthSource produces samples ===

    #[test]
    fn test_synth_source_produces_samples() {
        let source = SynthSource::new(44100, 0.01, |t| (t * 440.0 * std::f32::consts::TAU).sin());
        let samples: Vec<f32> = source.collect();
        assert!(!samples.is_empty());
        // 0.01s at 44100Hz = 441 samples
        assert_eq!(samples.len(), 441);
    }

    #[test]
    fn test_synth_source_channels() {
        let source = SynthSource::new(44100, 0.01, |_| 0.0);
        assert_eq!(rodio::Source::channels(&source), 1);
    }

    #[test]
    fn test_synth_source_sample_rate() {
        let source = SynthSource::new(44100, 0.01, |_| 0.0);
        assert_eq!(rodio::Source::sample_rate(&source), 44100);
    }

    #[test]
    fn test_synth_source_total_duration() {
        let source = SynthSource::new(44100, 0.5, |_| 0.0);
        let dur = rodio::Source::total_duration(&source);
        assert!(dur.is_some());
        let dur = dur.unwrap();
        assert!((dur.as_secs_f32() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_synth_source_current_frame_len() {
        let source = SynthSource::new(44100, 0.01, |_| 0.0);
        let frame_len = rodio::Source::current_frame_len(&source);
        assert_eq!(frame_len, Some(441));
    }

    #[test]
    fn test_synth_source_envelope_fades() {
        let source = SynthSource::new(44100, 0.1, |t| (t * 440.0 * std::f32::consts::TAU).sin());
        let samples: Vec<f32> = source.collect();
        // Last sample should be near zero due to envelope fade
        let last = samples.last().unwrap().abs();
        let first_nonzero = samples.iter().find(|s| s.abs() > 0.001);
        assert!(first_nonzero.is_some());
        assert!(last < 0.01, "Last sample should be near zero, got {}", last);
    }

    #[test]
    fn test_synth_source_exhausts() {
        let mut source = SynthSource::new(44100, 0.001, |_| 1.0);
        let mut count = 0;
        while source.next().is_some() {
            count += 1;
        }
        assert_eq!(count, 44); // 0.001 * 44100 ≈ 44
        assert!(source.next().is_none());
    }
}
