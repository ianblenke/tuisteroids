## ADDED Requirements

### Requirement: Attract Mode Demo Game
The system SHALL maintain a demo PlayingState during the Menu state. The demo SHALL use AI-generated input to control the ship. The demo game SHALL be updated each frame using the same fixed-timestep loop as normal gameplay.

#### Scenario: Demo game starts on application launch
- **GIVEN** the application launches and enters Menu state
- **WHEN** the Game is initialized
- **THEN** a demo PlayingState SHALL exist

#### Scenario: Demo game updates with AI input
- **GIVEN** the game is in Menu state with a demo running
- **WHEN** a frame update occurs
- **THEN** the demo PlayingState SHALL be updated with AI-generated input and the ship SHALL have moved

#### Scenario: Demo resets on game over
- **GIVEN** the demo game's ship has lost all lives
- **WHEN** the demo transitions to GameOver
- **THEN** a new demo PlayingState SHALL be created to replace the old one

#### Scenario: Demo discarded on game start
- **GIVEN** the game is in Menu state with a demo running
- **WHEN** the player presses a key to start
- **THEN** the demo PlayingState SHALL be dropped

#### Scenario: Demo restarts when returning to menu from game over
- **GIVEN** the game transitions from GameOver to Menu
- **WHEN** the Menu state is entered
- **THEN** a new demo PlayingState SHALL be created

#### Scenario: Demo restarts when returning to menu from playing
- **GIVEN** the game transitions from Playing to Menu via quit
- **WHEN** the Menu state is entered
- **THEN** a new demo PlayingState SHALL be created
