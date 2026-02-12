## ADDED Requirements

### Requirement: Two-Dimensional Vector Type
The system SHALL provide a `Vec2` type representing a 2D vector with `x` and `y` components as 64-bit floating point numbers.

The `Vec2` type SHALL support the following operations:
- Addition of two vectors
- Subtraction of two vectors
- Scalar multiplication
- Magnitude (length) calculation
- Normalization (unit vector)
- Dot product
- Creation from angle (unit vector pointing in direction of angle in radians)

#### Scenario: Vector addition
- **GIVEN** vector A = (3.0, 4.0) and vector B = (1.0, 2.0)
- **WHEN** A is added to B
- **THEN** the result SHALL be (4.0, 6.0)

#### Scenario: Vector subtraction
- **GIVEN** vector A = (3.0, 4.0) and vector B = (1.0, 2.0)
- **WHEN** B is subtracted from A
- **THEN** the result SHALL be (2.0, 2.0)

#### Scenario: Scalar multiplication
- **GIVEN** vector A = (3.0, 4.0) and scalar s = 2.0
- **WHEN** A is multiplied by s
- **THEN** the result SHALL be (6.0, 8.0)

#### Scenario: Vector magnitude
- **GIVEN** vector A = (3.0, 4.0)
- **WHEN** the magnitude is calculated
- **THEN** the result SHALL be 5.0

#### Scenario: Vector normalization
- **GIVEN** vector A = (3.0, 4.0)
- **WHEN** the vector is normalized
- **THEN** the result SHALL be (0.6, 0.8) and the magnitude SHALL be 1.0

#### Scenario: Zero vector normalization
- **GIVEN** vector A = (0.0, 0.0)
- **WHEN** the vector is normalized
- **THEN** the result SHALL be (0.0, 0.0)

#### Scenario: Dot product
- **GIVEN** vector A = (1.0, 0.0) and vector B = (0.0, 1.0)
- **WHEN** the dot product is calculated
- **THEN** the result SHALL be 0.0

#### Scenario: Vector from angle zero
- **GIVEN** an angle of 0 radians
- **WHEN** a unit vector is created from that angle
- **THEN** the result SHALL be (1.0, 0.0) (pointing right)

#### Scenario: Vector from angle 90 degrees
- **GIVEN** an angle of PI/2 radians
- **WHEN** a unit vector is created from that angle
- **THEN** the result SHALL be approximately (0.0, 1.0) (pointing down)

### Requirement: Motion Integration
The system SHALL update entity positions by integrating velocity over a fixed time step using Euler integration: `position = position + velocity * dt`.

#### Scenario: Position update with constant velocity
- **GIVEN** an entity at position (100.0, 100.0) with velocity (60.0, 0.0)
- **WHEN** one time step of dt = 1/60 seconds is applied
- **THEN** the position SHALL be (101.0, 100.0)

#### Scenario: Position update with zero velocity
- **GIVEN** an entity at position (50.0, 50.0) with velocity (0.0, 0.0)
- **WHEN** one time step is applied
- **THEN** the position SHALL remain (50.0, 50.0)

### Requirement: Velocity Dampening
The system SHALL apply a drag coefficient to entity velocities each frame to simulate friction: `velocity = velocity * drag_factor` where drag_factor is a value slightly less than 1.0 (e.g., 0.99).

#### Scenario: Velocity decreases over time with drag
- **GIVEN** an entity with velocity (100.0, 0.0) and drag factor 0.99
- **WHEN** one time step is applied
- **THEN** the velocity magnitude SHALL be 99.0

#### Scenario: Stationary entity stays stationary
- **GIVEN** an entity with velocity (0.0, 0.0) and drag factor 0.99
- **WHEN** one time step is applied
- **THEN** the velocity SHALL remain (0.0, 0.0)

### Requirement: Toroidal Wrapping
The system SHALL wrap entity positions around screen boundaries so that an entity leaving one edge appears at the opposite edge. The world coordinate space SHALL be defined by a width and height.

#### Scenario: Wrap right edge to left
- **GIVEN** a world of size 800x600 and an entity at position (801.0, 300.0)
- **WHEN** wrapping is applied
- **THEN** the position SHALL be (1.0, 300.0)

#### Scenario: Wrap left edge to right
- **GIVEN** a world of size 800x600 and an entity at position (-1.0, 300.0)
- **WHEN** wrapping is applied
- **THEN** the position SHALL be (799.0, 300.0)

#### Scenario: Wrap bottom edge to top
- **GIVEN** a world of size 800x600 and an entity at position (400.0, 601.0)
- **WHEN** wrapping is applied
- **THEN** the position SHALL be (400.0, 1.0)

#### Scenario: Wrap top edge to bottom
- **GIVEN** a world of size 800x600 and an entity at position (400.0, -1.0)
- **WHEN** wrapping is applied
- **THEN** the position SHALL be (400.0, 599.0)

#### Scenario: No wrapping when inside bounds
- **GIVEN** a world of size 800x600 and an entity at position (400.0, 300.0)
- **WHEN** wrapping is applied
- **THEN** the position SHALL remain (400.0, 300.0)

### Requirement: Angular Rotation
The system SHALL support rotating an angle by a given angular velocity over a time step: `angle = angle + angular_velocity * dt`. Angles SHALL be normalized to the range [0, 2*PI).

#### Scenario: Rotate clockwise
- **GIVEN** an angle of 0.0 radians and angular velocity of PI radians/second
- **WHEN** one time step of dt = 1.0 second is applied
- **THEN** the angle SHALL be PI radians

#### Scenario: Angle wraps past 2*PI
- **GIVEN** an angle of 6.0 radians and angular velocity of 1.0 radian/second
- **WHEN** one time step of dt = 1.0 second is applied
- **THEN** the angle SHALL be normalized to (7.0 - 2*PI) radians
