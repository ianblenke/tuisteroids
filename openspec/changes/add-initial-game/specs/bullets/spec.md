## ADDED Requirements

### Requirement: Bullet Creation
The system SHALL create a bullet when the player activates the Fire action. The bullet SHALL spawn at the ship's nose position (tip of the triangle) and travel in the ship's current facing direction.

#### Scenario: Bullet spawns at ship nose
- **GIVEN** a ship at position (400.0, 300.0) with rotation angle 0 (facing right)
- **WHEN** the Fire action is triggered
- **THEN** a bullet SHALL be created at the ship's nose vertex position, moving in the direction of angle 0

#### Scenario: Bullet direction matches ship rotation
- **GIVEN** a ship at rotation angle PI/2 (facing down)
- **WHEN** the Fire action is triggered
- **THEN** the bullet SHALL travel in the downward direction (angle PI/2)

### Requirement: Bullet Speed
The system SHALL give bullets a fixed speed (e.g., 500.0 units/second) relative to the world, independent of the ship's velocity. Bullets SHALL NOT inherit the ship's velocity.

#### Scenario: Bullet travels at fixed speed
- **GIVEN** a bullet is fired from a stationary ship facing right
- **WHEN** the bullet's velocity is measured
- **THEN** the velocity magnitude SHALL be 500.0 units/second

#### Scenario: Bullet speed is independent of ship velocity
- **GIVEN** a ship moving at velocity (100.0, 0.0) facing right fires a bullet
- **WHEN** the bullet's velocity is measured
- **THEN** the bullet velocity SHALL be (500.0, 0.0), NOT (600.0, 0.0)

### Requirement: Bullet Lifetime
The system SHALL destroy bullets after a fixed lifetime (e.g., 1.0 seconds or 60 frames at 60 FPS). Bullets SHALL not persist indefinitely.

#### Scenario: Bullet exists within lifetime
- **GIVEN** a bullet that was created 30 frames ago with a lifetime of 60 frames
- **WHEN** the bullet's status is checked
- **THEN** the bullet SHALL still be alive

#### Scenario: Bullet destroyed after lifetime expires
- **GIVEN** a bullet that was created 60 frames ago with a lifetime of 60 frames
- **WHEN** the bullet's status is checked
- **THEN** the bullet SHALL be marked for removal

### Requirement: Maximum Bullets on Screen
The system SHALL limit the number of active bullets to a maximum of 4 at any time. If 4 bullets are already active, the Fire action SHALL be ignored until a bullet is destroyed or expires.

#### Scenario: Can fire when under bullet limit
- **GIVEN** 3 active bullets on screen
- **WHEN** the Fire action is triggered
- **THEN** a new bullet SHALL be created (4 total)

#### Scenario: Cannot fire when at bullet limit
- **GIVEN** 4 active bullets on screen
- **WHEN** the Fire action is triggered
- **THEN** no new bullet SHALL be created

#### Scenario: Can fire again after bullet expires
- **GIVEN** 4 active bullets and one expires
- **WHEN** the Fire action is triggered
- **THEN** a new bullet SHALL be created (back to 4 total)

### Requirement: Bullet Screen Wrapping
Bullets SHALL wrap around screen edges using the same toroidal wrapping as all other entities.

#### Scenario: Bullet wraps from right to left
- **GIVEN** a world of size 800x600 and a bullet at position (799.0, 300.0) moving right
- **WHEN** the bullet crosses the right edge
- **THEN** the bullet SHALL appear at the left edge

### Requirement: Bullet Visual Representation
The system SHALL render bullets as small dots (1-2 pixel radius in braille coordinates).

#### Scenario: Bullet has small collision radius
- **GIVEN** a bullet is created
- **WHEN** its radius is queried
- **THEN** the radius SHALL be approximately 2.0 units
