## ADDED Requirements

### Requirement: Demo AI Target Selection
The demo AI SHALL select the nearest asteroid as its target using toroidal distance. If no asteroids exist, the AI SHALL produce a no-op InputState (no actions active).

#### Scenario: AI selects nearest asteroid
- **GIVEN** a ship at (400, 300) and asteroids at (100, 100) and (450, 310)
- **WHEN** the AI generates input
- **THEN** the AI SHALL target the asteroid at (450, 310) as it is nearest

#### Scenario: AI handles no asteroids
- **GIVEN** a ship and an empty asteroid list
- **WHEN** the AI generates input
- **THEN** all actions in the InputState SHALL be inactive

#### Scenario: AI uses toroidal distance for target selection
- **GIVEN** a ship at (10, 300) and asteroids at (200, 300) and (790, 300) in an 800x600 world
- **WHEN** the AI selects a target
- **THEN** the asteroid at (790, 300) SHALL be selected (toroidal distance 20 < direct distance 190)

### Requirement: Demo AI Rotation
The demo AI SHALL rotate the ship toward the nearest asteroid. It SHALL compute the signed angle difference between the ship's facing direction and the direction to the target, and activate RotateLeft or RotateRight accordingly. A deadzone SHALL prevent jittery oscillation when nearly aligned.

#### Scenario: AI rotates right toward target
- **GIVEN** a ship facing right (rotation=0) and the nearest asteroid is below (positive angle difference)
- **WHEN** the AI generates input
- **THEN** RotateRight SHALL be active and RotateLeft SHALL be inactive

#### Scenario: AI rotates left toward target
- **GIVEN** a ship facing right (rotation=0) and the nearest asteroid is above (negative angle difference)
- **WHEN** the AI generates input
- **THEN** RotateLeft SHALL be active and RotateRight SHALL be inactive

#### Scenario: AI does not rotate when aligned within deadzone
- **GIVEN** a ship facing directly toward the nearest asteroid (angle difference < deadzone)
- **WHEN** the AI generates input
- **THEN** both RotateLeft and RotateRight SHALL be inactive

### Requirement: Demo AI Firing
The demo AI SHALL fire when the ship is approximately aligned with the nearest asteroid. The fire action SHALL activate when the absolute angle difference is below a firing threshold.

#### Scenario: AI fires when aligned
- **GIVEN** a ship nearly aligned with the nearest asteroid (angle difference < fire threshold)
- **WHEN** the AI generates input
- **THEN** Fire SHALL be active

#### Scenario: AI does not fire when misaligned
- **GIVEN** a ship facing away from the nearest asteroid (angle difference > fire threshold)
- **WHEN** the AI generates input
- **THEN** Fire SHALL be inactive

### Requirement: Demo AI Thrust
The demo AI SHALL thrust when the ship is roughly facing the nearest asteroid. The thrust action SHALL activate when the absolute angle difference is below a thrust threshold (wider than the fire threshold).

#### Scenario: AI thrusts when roughly facing target
- **GIVEN** a ship facing within the thrust threshold of the nearest asteroid
- **WHEN** the AI generates input
- **THEN** Thrust SHALL be active

#### Scenario: AI does not thrust when facing away
- **GIVEN** a ship facing more than the thrust threshold away from the nearest asteroid
- **WHEN** the AI generates input
- **THEN** Thrust SHALL be inactive

### Requirement: Demo AI Quit Independence
The demo AI SHALL never activate the Quit action.

#### Scenario: AI never quits
- **GIVEN** any game state
- **WHEN** the AI generates input
- **THEN** Quit SHALL always be inactive
