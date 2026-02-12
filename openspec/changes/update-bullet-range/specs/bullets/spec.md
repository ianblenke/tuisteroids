## MODIFIED Requirements

### Requirement: Bullet Lifetime
The system SHALL destroy bullets after they have traveled a distance equal to 80% of the world width, matching the original Asteroids arcade game's bullet range. Bullets SHALL track cumulative distance traveled each frame and expire once the threshold is reached.

#### Scenario: Bullet exists within travel distance
- **GIVEN** a world of width 800.0 and a bullet that has traveled 300.0 units
- **WHEN** the bullet's status is checked
- **THEN** the bullet SHALL still be alive (300.0 < 640.0 = 80% of 800.0)

#### Scenario: Bullet destroyed after exceeding travel distance
- **GIVEN** a world of width 800.0 and a bullet that has traveled 640.0 units
- **WHEN** the bullet's status is checked
- **THEN** the bullet SHALL be marked for removal (640.0 >= 80% of 800.0)

#### Scenario: Distance threshold scales with world width
- **GIVEN** a world of width 1000.0
- **WHEN** a bullet has traveled 800.0 units
- **THEN** the bullet SHALL be marked for removal (800.0 >= 80% of 1000.0)
