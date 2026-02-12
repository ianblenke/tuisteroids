## MODIFIED Requirements

### Requirement: Game Update Sequence
During the Playing state, each fixed-timestep update SHALL process game logic in the following order:
1. Process input
2. Update ship (rotation, thrust, position)
3. Update bullets (position, lifetime)
4. Update asteroids (position)
5. Check collisions (bullet-asteroid, ship-asteroid)
6. Process collision results (scoring, splitting, lives)
7. Check wave completion and spawn new wave if needed

The update function SHALL return an UpdateResult containing both an optional state transition and a list of AudioEvent values corresponding to gameplay actions that occurred during the update.

Audio events SHALL be emitted for the following gameplay actions:
- Fire: when a bullet is fired
- Thrust: when thrust input is active
- AsteroidExplosionLarge/Medium/Small: when an asteroid is destroyed (variant matches size)
- ShipDestroyed: when the player ship is destroyed
- ExtraLife: when the player earns an extra life
- NewWave: when a new asteroid wave spawns

#### Scenario: Input processed before movement
- **GIVEN** the player presses Thrust on this frame
- **WHEN** the update runs
- **THEN** the ship's velocity SHALL be updated by thrust before the position is updated

#### Scenario: Collisions checked after movement
- **GIVEN** a bullet and asteroid are moving toward each other
- **WHEN** the update runs
- **THEN** collisions SHALL be checked against the new (post-movement) positions

#### Scenario: Scoring happens after collision detection
- **GIVEN** a bullet hits an asteroid
- **WHEN** the update processes collisions
- **THEN** the score SHALL be updated after the collision is detected and the asteroid is marked for splitting

#### Scenario: Update returns audio events for fire
- **GIVEN** the player presses fire during an update
- **WHEN** the update processes input and fires a bullet
- **THEN** the UpdateResult SHALL contain an AudioEvent::Fire

#### Scenario: Update returns audio events for thrust
- **GIVEN** the player holds thrust during an update
- **WHEN** the update processes input
- **THEN** the UpdateResult SHALL contain an AudioEvent::Thrust

#### Scenario: Update returns audio events for asteroid destruction
- **GIVEN** a bullet hits a large asteroid
- **WHEN** the update processes collisions
- **THEN** the UpdateResult SHALL contain an AudioEvent::AsteroidExplosionLarge

#### Scenario: Update returns audio events for ship destruction
- **GIVEN** the ship collides with an asteroid and has lives remaining
- **WHEN** the update processes collisions
- **THEN** the UpdateResult SHALL contain an AudioEvent::ShipDestroyed

#### Scenario: Update returns audio events for extra life
- **GIVEN** the player's score crosses an extra life threshold
- **WHEN** the update processes scoring
- **THEN** the UpdateResult SHALL contain an AudioEvent::ExtraLife

#### Scenario: Update returns audio events for new wave
- **GIVEN** all asteroids are destroyed and the wave delay has elapsed
- **WHEN** the update spawns a new wave
- **THEN** the UpdateResult SHALL contain an AudioEvent::NewWave

#### Scenario: Update returns empty audio events when nothing happens
- **GIVEN** a frame where no shots are fired, no collisions occur, and thrust is inactive
- **WHEN** the update completes
- **THEN** the UpdateResult audio_events list SHALL be empty
