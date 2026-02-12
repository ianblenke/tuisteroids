## ADDED Requirements

### Requirement: Braille Character Rasterization
The system SHALL render vector-like line segments using Unicode Braille characters (U+2800 through U+28FF). Each terminal cell SHALL map to a 2x4 dot grid, providing sub-cell resolution for line drawing.

#### Scenario: Empty cell renders as blank braille
- **GIVEN** a terminal cell with no dots set
- **WHEN** the cell is rendered
- **THEN** it SHALL display the empty braille character (U+2800) or a space

#### Scenario: Single dot renders correct braille character
- **GIVEN** a terminal cell with only the top-left dot set (position 0,0)
- **WHEN** the cell is rendered
- **THEN** it SHALL display the braille character U+2801

#### Scenario: Multiple dots combine in single cell
- **GIVEN** a terminal cell with dots at positions (0,0) and (1,0)
- **WHEN** the cell is rendered
- **THEN** the dots SHALL be combined into a single braille character using bitwise OR of dot positions

### Requirement: Line Rasterization
The system SHALL rasterize line segments from world coordinates to braille dot positions using Bresenham's line algorithm. The rasterizer SHALL convert floating-point world coordinates to integer braille dot coordinates.

#### Scenario: Horizontal line renders across cells
- **GIVEN** a line from world coordinate (0.0, 0.0) to (10.0, 0.0)
- **WHEN** the line is rasterized
- **THEN** braille dots SHALL be set along a horizontal path spanning the appropriate terminal cells

#### Scenario: Vertical line renders down cells
- **GIVEN** a line from world coordinate (0.0, 0.0) to (0.0, 10.0)
- **WHEN** the line is rasterized
- **THEN** braille dots SHALL be set along a vertical path

#### Scenario: Diagonal line renders across cells
- **GIVEN** a line from (0.0, 0.0) to (10.0, 10.0)
- **WHEN** the line is rasterized
- **THEN** braille dots SHALL be set along a diagonal path using Bresenham's algorithm

#### Scenario: Short line within single cell
- **GIVEN** a line segment that fits entirely within one terminal cell
- **WHEN** the line is rasterized
- **THEN** only dots within that single cell SHALL be set

### Requirement: Polygon Rendering
The system SHALL render polygons by drawing line segments between consecutive vertices, with the last vertex connected back to the first. Polygons are used for ship and asteroid shapes.

#### Scenario: Triangle renders as three line segments
- **GIVEN** a polygon with vertices [(0,0), (10,0), (5,8)]
- **WHEN** the polygon is rendered
- **THEN** three line segments SHALL be drawn: (0,0)→(10,0), (10,0)→(5,8), (5,8)→(0,0)

#### Scenario: Polygon vertices are transformed by position and rotation
- **GIVEN** a polygon defined in local space, an entity at position (100.0, 100.0) with rotation PI/4
- **WHEN** the polygon is rendered
- **THEN** all vertices SHALL be rotated by PI/4 and translated by (100.0, 100.0) before rasterization

### Requirement: Braille Buffer
The system SHALL maintain a buffer of braille dot data sized to the terminal dimensions. The buffer SHALL support setting individual dots, clearing the entire buffer, and converting the buffer to a grid of braille Unicode characters for display.

#### Scenario: Buffer clears to empty
- **GIVEN** a braille buffer with some dots set
- **WHEN** the buffer is cleared
- **THEN** all dots SHALL be unset and all cells SHALL render as empty braille

#### Scenario: Buffer converts to character grid
- **GIVEN** a braille buffer with known dot patterns
- **WHEN** the buffer is converted to characters
- **THEN** each cell SHALL produce the correct Unicode braille character encoding its 2x4 dot pattern

#### Scenario: Buffer respects terminal dimensions
- **GIVEN** a terminal of 80 columns by 24 rows
- **WHEN** a braille buffer is created
- **THEN** the buffer SHALL represent 160 horizontal dots (80*2) by 96 vertical dots (24*4)

### Requirement: HUD Display
The system SHALL render a heads-up display showing the current score and remaining lives. The score SHALL be displayed at the top-left of the screen. Remaining lives SHALL be shown as small ship icons at the top-left below or beside the score.

#### Scenario: Score displays at top-left
- **GIVEN** a player with score 1250
- **WHEN** the HUD is rendered
- **THEN** the text "1250" SHALL appear at the top-left area of the screen

#### Scenario: Lives display as ship icons
- **GIVEN** a player with 3 remaining lives
- **WHEN** the HUD is rendered
- **THEN** 3 small ship-shaped icons SHALL be displayed near the top-left

#### Scenario: Score updates in real time
- **GIVEN** a player who just scored 100 points (old score 500, new score 600)
- **WHEN** the next frame is rendered
- **THEN** the HUD SHALL display the updated score 600

### Requirement: Game Over Screen
The system SHALL display a "GAME OVER" message centered on the screen when the player loses all lives. The screen SHALL also display the final score and a prompt to restart or quit.

#### Scenario: Game over text is centered
- **GIVEN** the game state is GameOver with a final score of 5000
- **WHEN** the game over screen is rendered
- **THEN** "GAME OVER" SHALL be centered on screen with the score displayed

#### Scenario: Game over shows restart prompt
- **GIVEN** the game state is GameOver
- **WHEN** the game over screen is rendered
- **THEN** a prompt to press a key to restart or Q to quit SHALL be visible

### Requirement: Menu Screen
The system SHALL display a title screen when the game is in the Menu state. The title screen SHALL show the game name "TUISTEROIDS" and a prompt to start the game.

#### Scenario: Title screen shows game name
- **GIVEN** the game state is Menu
- **WHEN** the menu screen is rendered
- **THEN** "TUISTEROIDS" SHALL be displayed prominently on screen

#### Scenario: Title screen shows start prompt
- **GIVEN** the game state is Menu
- **WHEN** the menu screen is rendered
- **THEN** a prompt to press a key to start SHALL be visible

### Requirement: Ship Invulnerability Visual Feedback
The system SHALL provide visual feedback when the ship is invulnerable by blinking the ship (alternating between visible and invisible at a regular interval).

#### Scenario: Invulnerable ship blinks
- **GIVEN** a ship that is invulnerable after respawn
- **WHEN** frames are rendered
- **THEN** the ship SHALL alternate between visible and invisible approximately every 100ms

#### Scenario: Vulnerable ship renders normally
- **GIVEN** a ship that is not invulnerable
- **WHEN** a frame is rendered
- **THEN** the ship SHALL always be visible

### Requirement: Thrust Visual Feedback
The system SHALL render a thrust flame behind the ship when the Thrust action is active. The flame SHALL be a small flickering triangle behind the ship's base.

#### Scenario: Flame visible during thrust
- **GIVEN** a ship with Thrust action active
- **WHEN** the frame is rendered
- **THEN** a small triangle SHALL be drawn behind the ship's base, opposite to the facing direction

#### Scenario: No flame without thrust
- **GIVEN** a ship with Thrust action inactive
- **WHEN** the frame is rendered
- **THEN** no flame triangle SHALL be drawn behind the ship
