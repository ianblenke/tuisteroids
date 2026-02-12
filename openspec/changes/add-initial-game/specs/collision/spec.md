## ADDED Requirements

### Requirement: Circle-Circle Collision Detection
The system SHALL detect collisions between two entities by comparing the distance between their centers to the sum of their radii. A collision occurs when `distance(center_a, center_b) < radius_a + radius_b`.

#### Scenario: Overlapping circles collide
- **GIVEN** entity A at position (100.0, 100.0) with radius 20.0 and entity B at position (110.0, 100.0) with radius 20.0
- **WHEN** collision is checked
- **THEN** a collision SHALL be detected (distance 10.0 < 40.0)

#### Scenario: Non-overlapping circles do not collide
- **GIVEN** entity A at position (100.0, 100.0) with radius 10.0 and entity B at position (200.0, 200.0) with radius 10.0
- **WHEN** collision is checked
- **THEN** no collision SHALL be detected

#### Scenario: Touching circles collide
- **GIVEN** entity A at position (100.0, 100.0) with radius 10.0 and entity B at position (119.0, 100.0) with radius 10.0
- **WHEN** collision is checked
- **THEN** a collision SHALL be detected (distance 19.0 < 20.0)

### Requirement: Toroidal Distance Calculation
The system SHALL calculate distances between entities using the shortest path on a toroidal (wrapping) surface. When the direct distance exceeds half the world dimension, the wrapped distance SHALL be used instead.

#### Scenario: Direct distance is shortest
- **GIVEN** a world of size 800x600, entity A at (100.0, 100.0), entity B at (150.0, 100.0)
- **WHEN** toroidal distance is calculated
- **THEN** the distance SHALL be 50.0

#### Scenario: Wrapped horizontal distance is shorter
- **GIVEN** a world of size 800x600, entity A at (10.0, 300.0), entity B at (790.0, 300.0)
- **WHEN** toroidal distance is calculated
- **THEN** the distance SHALL be 20.0 (wrapping around: 800 - 780 = 20)

#### Scenario: Wrapped vertical distance is shorter
- **GIVEN** a world of size 800x600, entity A at (400.0, 10.0), entity B at (400.0, 590.0)
- **WHEN** toroidal distance is calculated
- **THEN** the distance SHALL be 20.0 (wrapping around: 600 - 580 = 20)

#### Scenario: Both axes wrap
- **GIVEN** a world of size 800x600, entity A at (10.0, 10.0), entity B at (790.0, 590.0)
- **WHEN** toroidal distance is calculated
- **THEN** the distance SHALL use wrapped distances on both axes (dx=20, dy=20, distance=~28.28)

### Requirement: Ship-Asteroid Collision
The system SHALL detect when the player ship collides with any asteroid. When a collision is detected and the ship is not invulnerable, the ship SHALL lose one life and enter a respawn state.

#### Scenario: Ship hits asteroid and loses life
- **GIVEN** a ship at position (400.0, 300.0) with radius 12.0 and 3 lives, not invulnerable, and an asteroid at position (405.0, 300.0) with radius 30.0
- **WHEN** collision is checked and detected
- **THEN** the ship SHALL lose one life (now 2 lives) and enter respawn state

#### Scenario: Invulnerable ship ignores asteroid collision
- **GIVEN** a ship at position (400.0, 300.0) that is invulnerable (respawn timer active) and an asteroid at position (405.0, 300.0)
- **WHEN** collision is checked
- **THEN** no life SHALL be lost and the ship SHALL remain in its current state

#### Scenario: Ship loses last life triggers game over
- **GIVEN** a ship with 1 life remaining that collides with an asteroid
- **WHEN** the collision is processed
- **THEN** the ship SHALL have 0 lives and the game state SHALL transition to GameOver

### Requirement: Bullet-Asteroid Collision
The system SHALL detect when any bullet collides with any asteroid. When a collision is detected, the bullet SHALL be destroyed and the asteroid SHALL be split or destroyed according to its size.

#### Scenario: Bullet hits large asteroid
- **GIVEN** a bullet at position (200.0, 200.0) with radius 2.0 and a large asteroid at position (205.0, 200.0) with radius 30.0
- **WHEN** collision is detected
- **THEN** the bullet SHALL be removed and the asteroid SHALL split into 2 medium asteroids

#### Scenario: Bullet hits medium asteroid
- **GIVEN** a bullet colliding with a medium asteroid
- **WHEN** collision is detected
- **THEN** the bullet SHALL be removed and the asteroid SHALL split into 2 small asteroids

#### Scenario: Bullet hits small asteroid
- **GIVEN** a bullet colliding with a small asteroid
- **WHEN** collision is detected
- **THEN** both the bullet and the asteroid SHALL be removed (small asteroids do not split)

#### Scenario: Bullet and asteroid at wrapping boundary
- **GIVEN** a world of size 800x600, a bullet at (5.0, 300.0) with radius 2.0, and an asteroid at (795.0, 300.0) with radius 30.0
- **WHEN** collision is checked using toroidal distance
- **THEN** a collision SHALL be detected (wrapped distance = 10.0 < 32.0)
