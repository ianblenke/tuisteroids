## MODIFIED Requirements

### Requirement: Menu Screen
The system SHALL display a title screen when the game is in the Menu state. When a demo game is active, the system SHALL render the demo game (asteroids, bullets, ship) as a braille background and overlay the title and start prompt text on top. The demo game's HUD (score, lives) SHALL NOT be displayed.

#### Scenario: Title screen shows game name
- **GIVEN** the game state is Menu
- **WHEN** the menu screen is rendered
- **THEN** "TUISTEROIDS" SHALL be displayed prominently on screen

#### Scenario: Title screen shows start prompt
- **GIVEN** the game state is Menu
- **WHEN** the menu screen is rendered
- **THEN** a prompt to press a key to start SHALL be visible

#### Scenario: Demo game renders behind menu text
- **GIVEN** the game state is Menu and a demo game is active
- **WHEN** the menu screen is rendered
- **THEN** asteroids, bullets, and the ship from the demo SHALL be visible as braille graphics behind the menu text

#### Scenario: No HUD in attract mode
- **GIVEN** the game state is Menu and a demo game is active
- **WHEN** the menu screen is rendered
- **THEN** no score or lives HUD SHALL be displayed for the demo game
