## ADDED Requirements

### Requirement: Ship Shape Definition
The system SHALL define the player ship as a triangle polygon with vertices relative to its center. The ship triangle SHALL point in the direction of the ship's current rotation angle. The ship SHALL have a bounding radius for collision detection.

#### Scenario: Ship vertices form a triangle
- **GIVEN** a ship at rotation angle 0 (pointing right)
- **WHEN** the ship's polygon vertices are calculated
- **THEN** the vertices SHALL form an isoceles triangle with the nose pointing in the rotation direction

#### Scenario: Ship vertices rotate with the ship
- **GIVEN** a ship at rotation angle PI/2 (pointing down)
- **WHEN** the ship's polygon vertices are calculated
- **THEN** the vertices SHALL be rotated PI/2 radians from the base orientation

### Requirement: Ship Rotation
The system SHALL rotate the ship at a fixed angular velocity when the RotateLeft or RotateRight input actions are active. The rotation speed SHALL be constant (e.g., 5.0 radians/second).

#### Scenario: Rotate left decreases angle
- **GIVEN** a ship with rotation angle 1.0 radian and RotateLeft is active
- **WHEN** one time step of dt = 1/60 is applied
- **THEN** the rotation angle SHALL decrease by (rotation_speed * dt)

#### Scenario: Rotate right increases angle
- **GIVEN** a ship with rotation angle 1.0 radian and RotateRight is active
- **WHEN** one time step of dt = 1/60 is applied
- **THEN** the rotation angle SHALL increase by (rotation_speed * dt)

#### Scenario: No rotation when no input
- **GIVEN** a ship with rotation angle 1.0 radian and neither RotateLeft nor RotateRight is active
- **WHEN** one time step is applied
- **THEN** the rotation angle SHALL remain 1.0 radian

### Requirement: Ship Thrust
The system SHALL apply acceleration in the ship's facing direction when the Thrust input action is active. The thrust SHALL add to the ship's current velocity (not replace it), creating momentum/inertia behavior.

#### Scenario: Thrust from standstill
- **GIVEN** a ship at angle 0 (facing right) with velocity (0.0, 0.0) and thrust acceleration 200.0
- **WHEN** Thrust is active for one time step of dt = 1/60
- **THEN** the velocity SHALL be approximately (3.33, 0.0)

#### Scenario: Thrust adds to existing velocity
- **GIVEN** a ship at angle 0 with velocity (50.0, 0.0) and thrust acceleration 200.0
- **WHEN** Thrust is active for one time step of dt = 1/60
- **THEN** the velocity SHALL be approximately (53.33, 0.0)

#### Scenario: Thrust in different direction than motion
- **GIVEN** a ship at angle PI/2 (facing down) with velocity (50.0, 0.0) and thrust acceleration 200.0
- **WHEN** Thrust is active for one time step of dt = 1/60
- **THEN** the velocity x-component SHALL remain ~50.0 and the y-component SHALL increase by ~3.33

#### Scenario: No acceleration without thrust
- **GIVEN** a ship with velocity (50.0, 0.0) and Thrust is NOT active
- **WHEN** one time step is applied
- **THEN** the velocity SHALL only change due to drag (no acceleration applied)

### Requirement: Ship Maximum Speed
The system SHALL enforce a maximum speed for the ship. If thrust would increase the ship's velocity magnitude beyond the maximum, the velocity SHALL be clamped to the maximum speed.

#### Scenario: Velocity clamped at maximum
- **GIVEN** a ship at maximum speed (e.g., 400.0) thrusting in the same direction
- **WHEN** Thrust is active for one time step
- **THEN** the velocity magnitude SHALL NOT exceed 400.0

### Requirement: Ship Lives
The system SHALL track the number of remaining lives for the player. The game SHALL start with 3 lives.

#### Scenario: Game starts with 3 lives
- **GIVEN** a new game is started
- **WHEN** the ship is initialized
- **THEN** the ship SHALL have 3 lives

#### Scenario: Ship loses one life on death
- **GIVEN** a ship with 3 lives
- **WHEN** the ship is destroyed (collides with asteroid)
- **THEN** the ship SHALL have 2 lives remaining

### Requirement: Ship Respawn
The system SHALL respawn the ship at the center of the screen after losing a life, with zero velocity, facing upward, and an invulnerability period of 3 seconds.

#### Scenario: Ship respawns at center
- **GIVEN** a ship that has just lost a life and has lives remaining
- **WHEN** respawn is triggered
- **THEN** the ship SHALL be placed at the center of the screen with zero velocity

#### Scenario: Ship is invulnerable after respawn
- **GIVEN** a ship that has just respawned
- **WHEN** 1 second has elapsed since respawn
- **THEN** the ship SHALL still be invulnerable (invulnerability lasts 3 seconds)

#### Scenario: Ship invulnerability expires
- **GIVEN** a ship that respawned 3 seconds ago
- **WHEN** invulnerability is checked
- **THEN** the ship SHALL no longer be invulnerable

### Requirement: Extra Life
The system SHALL award an extra life when the player's score reaches 10,000 points. Only one extra life SHALL be awarded per game (at the first crossing of 10,000 points).

#### Scenario: Extra life awarded at 10000 points
- **GIVEN** a player with 9,950 points and 3 lives who has not yet earned an extra life
- **WHEN** the player scores 100 points (total 10,050)
- **THEN** the player SHALL have 4 lives

#### Scenario: Extra life only awarded once
- **GIVEN** a player who has already been awarded the extra life and currently has 10,050 points
- **WHEN** the player's score increases further
- **THEN** no additional extra lives SHALL be awarded

### Requirement: Ship Screen Wrapping
The ship SHALL wrap around screen edges using toroidal wrapping, so exiting one side causes it to appear on the opposite side.

#### Scenario: Ship wraps from right to left
- **GIVEN** a ship moving right that passes the right edge of the screen
- **WHEN** position wrapping is applied
- **THEN** the ship SHALL appear at the left edge of the screen at the same vertical position
