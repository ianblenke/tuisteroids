## ADDED Requirements

### Requirement: Asteroid Sizes
The system SHALL support three asteroid sizes: Large, Medium, and Small. Each size SHALL have a defined bounding radius and point value.

- Large: radius ~40.0, worth 20 points
- Medium: radius ~20.0, worth 50 points
- Small: radius ~10.0, worth 100 points

#### Scenario: Large asteroid properties
- **GIVEN** a large asteroid is created
- **WHEN** its properties are queried
- **THEN** it SHALL have a radius of approximately 40.0 and a point value of 20

#### Scenario: Medium asteroid properties
- **GIVEN** a medium asteroid is created
- **WHEN** its properties are queried
- **THEN** it SHALL have a radius of approximately 20.0 and a point value of 50

#### Scenario: Small asteroid properties
- **GIVEN** a small asteroid is created
- **WHEN** its properties are queried
- **THEN** it SHALL have a radius of approximately 10.0 and a point value of 100

### Requirement: Asteroid Shape Generation
The system SHALL generate each asteroid with a random irregular polygon shape. The polygon SHALL be created by placing vertices at random distances from the center at evenly spaced angles, producing a jagged rock-like appearance. Each asteroid SHALL have a unique shape.

#### Scenario: Asteroid has polygon vertices
- **GIVEN** a new asteroid is created
- **WHEN** its shape is queried
- **THEN** it SHALL have between 8 and 12 polygon vertices forming a closed shape

#### Scenario: Asteroid vertices vary per instance
- **GIVEN** two asteroids are created with different random seeds
- **WHEN** their shapes are compared
- **THEN** the vertex positions SHALL differ (not identical shapes)

#### Scenario: Vertices are within radius bounds
- **GIVEN** an asteroid with radius R
- **WHEN** its vertex distances from center are measured
- **THEN** all vertex distances SHALL be between 0.5*R and 1.2*R

### Requirement: Asteroid Splitting
The system SHALL split asteroids when they are destroyed by a bullet. Large asteroids SHALL split into 2 Medium asteroids. Medium asteroids SHALL split into 2 Small asteroids. Small asteroids SHALL be destroyed without splitting.

#### Scenario: Large asteroid splits into two medium
- **GIVEN** a large asteroid at position (300.0, 300.0) is hit by a bullet
- **WHEN** the asteroid is destroyed
- **THEN** 2 medium asteroids SHALL be created at approximately the same position

#### Scenario: Medium asteroid splits into two small
- **GIVEN** a medium asteroid at position (300.0, 300.0) is hit by a bullet
- **WHEN** the asteroid is destroyed
- **THEN** 2 small asteroids SHALL be created at approximately the same position

#### Scenario: Small asteroid is destroyed completely
- **GIVEN** a small asteroid is hit by a bullet
- **WHEN** the asteroid is destroyed
- **THEN** no new asteroids SHALL be created

#### Scenario: Split asteroids inherit modified velocity
- **GIVEN** a large asteroid with velocity (10.0, 5.0) is split
- **WHEN** 2 medium asteroids are created
- **THEN** each child asteroid SHALL have a velocity derived from the parent's velocity with a random angular offset and slightly increased speed

### Requirement: Asteroid Movement
Asteroids SHALL move at a constant velocity (no acceleration or drag) and SHALL wrap around screen edges using toroidal wrapping. Each asteroid SHALL also rotate at a constant angular velocity for visual effect.

#### Scenario: Asteroid moves at constant velocity
- **GIVEN** an asteroid with velocity (30.0, 20.0)
- **WHEN** multiple time steps are applied without any collisions
- **THEN** the velocity SHALL remain (30.0, 20.0) each step

#### Scenario: Asteroid wraps around screen edge
- **GIVEN** a world of size 800x600 and an asteroid at position (799.0, 300.0) with velocity (5.0, 0.0)
- **WHEN** one time step of dt = 1/60 is applied and wrapping occurs
- **THEN** the asteroid SHALL appear near the left edge of the screen

#### Scenario: Asteroid rotates visually
- **GIVEN** an asteroid with angular velocity 0.5 radians/second
- **WHEN** one second of time steps are applied
- **THEN** the asteroid's rotation angle SHALL have increased by 0.5 radians

### Requirement: Wave System
The system SHALL spawn asteroids in waves. When all asteroids are destroyed, a new wave SHALL begin after a short delay. Wave N SHALL spawn (N + 3) large asteroids, where N starts at 1. Asteroids SHALL spawn at random positions along the screen edges, ensuring they do not spawn too close to the player ship.

#### Scenario: First wave spawns 4 large asteroids
- **GIVEN** a new game is started (wave 1)
- **WHEN** the wave begins
- **THEN** 4 large asteroids SHALL be spawned (1 + 3 = 4)

#### Scenario: Second wave spawns 5 large asteroids
- **GIVEN** wave 1 is cleared and wave 2 begins
- **WHEN** the wave spawns
- **THEN** 5 large asteroids SHALL be spawned (2 + 3 = 5)

#### Scenario: Asteroids spawn away from ship
- **GIVEN** a ship at center position (400.0, 300.0) and a new wave is spawning
- **WHEN** asteroid spawn positions are chosen
- **THEN** all asteroid positions SHALL be at least 150.0 units away from the ship

#### Scenario: New wave after delay when all asteroids destroyed
- **GIVEN** all asteroids have been destroyed
- **WHEN** a delay of approximately 2 seconds has elapsed
- **THEN** the next wave SHALL begin spawning

### Requirement: Asteroid Scoring
The system SHALL award points to the player when an asteroid is destroyed by a bullet. Points SHALL be awarded based on asteroid size: Large = 20, Medium = 50, Small = 100.

#### Scenario: Destroying large asteroid awards 20 points
- **GIVEN** a player with score 0 who destroys a large asteroid
- **WHEN** the score is updated
- **THEN** the player's score SHALL be 20

#### Scenario: Destroying medium asteroid awards 50 points
- **GIVEN** a player with score 20 who destroys a medium asteroid
- **WHEN** the score is updated
- **THEN** the player's score SHALL be 70

#### Scenario: Destroying small asteroid awards 100 points
- **GIVEN** a player with score 70 who destroys a small asteroid
- **WHEN** the score is updated
- **THEN** the player's score SHALL be 170

#### Scenario: Score accumulates across wave
- **GIVEN** a player destroys a large asteroid (20 pts), its 2 medium children (50 pts each), and all 4 small children (100 pts each)
- **WHEN** the total is calculated
- **THEN** the score for that sequence SHALL be 520 points (20 + 100 + 400)
