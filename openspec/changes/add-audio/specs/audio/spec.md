## ADDED Requirements

### Requirement: Audio Engine Initialization
The system SHALL attempt to initialize an audio output stream at startup. If initialization fails (no audio device available), the AudioEngine SHALL operate in silent mode where all playback calls are no-ops. The game SHALL NOT crash or produce errors when audio is unavailable.

#### Scenario: Audio engine initializes with available device
- **GIVEN** the system has an available audio output device
- **WHEN** AudioEngine::try_new() is called
- **THEN** the engine SHALL be created with an active output stream handle

#### Scenario: Audio engine initializes without audio device
- **GIVEN** the system has no available audio output device (e.g. SSH session)
- **WHEN** AudioEngine::try_new() is called
- **THEN** the engine SHALL be created in silent mode with no output stream
- **AND** no error SHALL be raised

#### Scenario: Silent engine ignores play calls
- **GIVEN** an AudioEngine in silent mode (no output stream)
- **WHEN** play() is called with any AudioEvent
- **THEN** the call SHALL complete without error and no sound SHALL be produced

### Requirement: Audio Event Types
The system SHALL define an AudioEvent enum representing distinct gameplay sounds. The following events SHALL be supported:
- **Fire**: player shoots a bullet
- **Thrust**: player is thrusting (continuous while held)
- **AsteroidExplosionLarge**: large asteroid destroyed (splits into medium)
- **AsteroidExplosionMedium**: medium asteroid destroyed (splits into small)
- **AsteroidExplosionSmall**: small asteroid destroyed (fully destroyed)
- **ShipDestroyed**: player ship is destroyed
- **ExtraLife**: player earns an extra life
- **NewWave**: a new asteroid wave spawns

#### Scenario: All event variants are distinct
- **GIVEN** the AudioEvent enum
- **WHEN** each variant is constructed
- **THEN** each variant SHALL be a distinct value that can be matched independently

#### Scenario: AudioEvent is non-exhaustive for future extension
- **GIVEN** the AudioEvent enum
- **WHEN** new sound effects are needed in the future
- **THEN** the enum SHALL support addition of new variants without breaking existing match arms

### Requirement: Procedural Sound Synthesis
All sounds SHALL be generated procedurally (no external audio files). Each AudioEvent SHALL map to a distinct synthesized sound using combinations of waveforms (sine, square, noise) and amplitude envelopes.

#### Scenario: Fire sound is a short high-pitched burst
- **GIVEN** an active AudioEngine
- **WHEN** AudioEvent::Fire is played
- **THEN** a short (50-100ms) high-frequency tone SHALL be produced

#### Scenario: Thrust sound is a low rumble
- **GIVEN** an active AudioEngine
- **WHEN** AudioEvent::Thrust is played
- **THEN** a low-frequency noise burst SHALL be produced

#### Scenario: Large explosion is a long low boom
- **GIVEN** an active AudioEngine
- **WHEN** AudioEvent::AsteroidExplosionLarge is played
- **THEN** a longer duration, lower frequency explosion sound SHALL be produced than medium or small explosions

#### Scenario: Small explosion is a short high pop
- **GIVEN** an active AudioEngine
- **WHEN** AudioEvent::AsteroidExplosionSmall is played
- **THEN** a shorter duration, higher frequency sound SHALL be produced than large or medium explosions

#### Scenario: Ship destroyed is a distinctive explosion
- **GIVEN** an active AudioEngine
- **WHEN** AudioEvent::ShipDestroyed is played
- **THEN** a long, dramatic explosion sound SHALL be produced that is distinguishable from asteroid explosions

#### Scenario: Extra life is a positive ascending tone
- **GIVEN** an active AudioEngine
- **WHEN** AudioEvent::ExtraLife is played
- **THEN** an ascending frequency sweep SHALL be produced

#### Scenario: New wave is a brief alert tone
- **GIVEN** an active AudioEngine
- **WHEN** AudioEvent::NewWave is played
- **THEN** a brief alert/warning tone SHALL be produced

### Requirement: Audio Event Playback
The AudioEngine SHALL accept AudioEvent values and play the corresponding synthesized sound. Playback SHALL be non-blocking — sounds play on a background thread and do not delay the game loop. Multiple sounds MAY overlap (mix together).

#### Scenario: Playback does not block the game loop
- **GIVEN** an active AudioEngine
- **WHEN** play() is called with an AudioEvent
- **THEN** the call SHALL return immediately without waiting for the sound to finish

#### Scenario: Multiple simultaneous sounds mix together
- **GIVEN** an active AudioEngine
- **WHEN** multiple AudioEvents are played in rapid succession (e.g. fire + explosion)
- **THEN** both sounds SHALL be audible simultaneously (mixed by the audio backend)

### Requirement: Thrust Sound Management
Thrust audio SHALL play a short burst each time the Thrust event is received. The game loop sends Thrust events on each update tick while the player holds the thrust key, producing a continuous rumble effect from rapid successive bursts.

#### Scenario: Thrust produces continuous rumble while held
- **GIVEN** the player holds the thrust key for multiple update ticks
- **WHEN** AudioEvent::Thrust is emitted each tick
- **THEN** rapid successive thrust bursts SHALL produce a continuous rumble effect

#### Scenario: Thrust stops when key released
- **GIVEN** the player releases the thrust key
- **WHEN** no more Thrust events are emitted
- **THEN** the thrust sound SHALL fade out naturally as the last burst decays
