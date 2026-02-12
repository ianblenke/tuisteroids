## ADDED Requirements

### Requirement: Fixed Timestep Game Loop
The system SHALL run a game loop using a fixed timestep of 1/60 seconds (~16.67ms). The loop SHALL use an accumulator pattern: elapsed real time is accumulated, and the game state is updated in fixed-size steps until the accumulator is drained below one timestep.

#### Scenario: Single update per frame at 60 FPS
- **GIVEN** the game loop is running and exactly 16.67ms of real time has elapsed
- **WHEN** the loop iterates
- **THEN** exactly one game update SHALL be performed

#### Scenario: Multiple updates when frame is slow
- **GIVEN** the game loop is running and 50ms of real time has elapsed (3x the timestep)
- **WHEN** the loop iterates
- **THEN** 3 game updates SHALL be performed to catch up

#### Scenario: No update when insufficient time elapsed
- **GIVEN** the game loop is running and only 5ms has elapsed since last iteration
- **WHEN** the loop iterates
- **THEN** no game update SHALL be performed (accumulator < timestep)

#### Scenario: Accumulator retains remainder
- **GIVEN** 40ms of real time elapsed (2.4 timesteps)
- **WHEN** 2 updates are performed
- **THEN** the remaining ~6.67ms SHALL be retained in the accumulator for the next iteration

### Requirement: Update-Render Separation
The system SHALL separate game state updates from rendering. Updates SHALL happen at fixed timestep intervals. Rendering SHALL happen once per loop iteration after all updates are complete.

#### Scenario: Render happens after updates
- **GIVEN** the game loop performs 2 updates in one iteration
- **WHEN** the iteration completes
- **THEN** exactly 1 render call SHALL be made after both updates

#### Scenario: Render happens even with zero updates
- **GIVEN** the game loop has insufficient accumulated time for an update
- **WHEN** the iteration completes
- **THEN** 1 render call SHALL still be made (to maintain display responsiveness)

### Requirement: Game State Machine
The system SHALL manage game state as a state machine with three states: Menu, Playing, and GameOver. Transitions SHALL occur based on game events and player input.

State transitions:
- Menu → Playing: when player presses any key (except Q)
- Playing → GameOver: when player loses last life
- Playing → Menu: when player presses Q (quit)
- GameOver → Menu: when player presses any key

#### Scenario: Game starts in Menu state
- **GIVEN** the application is launched
- **WHEN** the initial state is set
- **THEN** the game state SHALL be Menu

#### Scenario: Menu transitions to Playing on key press
- **GIVEN** the game state is Menu
- **WHEN** the player presses any key except Q
- **THEN** the game state SHALL transition to Playing and a new game SHALL be initialized

#### Scenario: Playing transitions to GameOver on last life lost
- **GIVEN** the game state is Playing and the ship has 1 life
- **WHEN** the ship is destroyed
- **THEN** the game state SHALL transition to GameOver

#### Scenario: Playing transitions to Menu on quit
- **GIVEN** the game state is Playing
- **WHEN** the player presses Q
- **THEN** the game state SHALL transition to Menu (or the application exits)

#### Scenario: GameOver transitions to Menu on key press
- **GIVEN** the game state is GameOver
- **WHEN** the player presses any key
- **THEN** the game state SHALL transition to Menu

### Requirement: Game Update Sequence
During the Playing state, each fixed-timestep update SHALL process game logic in the following order:
1. Process input
2. Update ship (rotation, thrust, position)
3. Update bullets (position, lifetime)
4. Update asteroids (position)
5. Check collisions (bullet-asteroid, ship-asteroid)
6. Process collision results (scoring, splitting, lives)
7. Check wave completion and spawn new wave if needed

#### Scenario: Input processed before movement
- **GIVEN** the player presses Thrust on this frame
- **WHEN** the update runs
- **THEN** the ship's velocity SHALL be updated by thrust before the position is updated

#### Scenario: Collisions checked after movement
- **GIVEN** a bullet and asteroid are moving toward each other
- **WHEN** the update runs
- **THEN** collisions SHALL be checked against the new (post-movement) positions

#### Scenario: Scoring happens after collision detection
- **GIVEN** a bullet hits an asteroid
- **WHEN** the update processes collisions
- **THEN** the score SHALL be updated after the collision is detected and the asteroid is marked for splitting

### Requirement: Quit Handling
The system SHALL exit the game when the player presses the Q key. The terminal SHALL be restored to its original state before the process exits.

#### Scenario: Q key exits from menu
- **GIVEN** the game state is Menu
- **WHEN** the player presses Q
- **THEN** the game loop SHALL terminate and the terminal SHALL be restored

#### Scenario: Q key exits from playing
- **GIVEN** the game state is Playing
- **WHEN** the player presses Q
- **THEN** the game loop SHALL terminate and the terminal SHALL be restored

#### Scenario: Terminal restored on exit
- **GIVEN** the game is running in raw/alternate screen mode
- **WHEN** the game exits (by Q or game over + quit)
- **THEN** the terminal SHALL be restored to its original mode (cooked mode, main screen)

### Requirement: Frame Rate Limiting
The system SHALL sleep for remaining time after update and render to avoid consuming 100% CPU. If an iteration completes faster than the target frame time (~16.67ms), the system SHALL sleep for the remaining duration.

#### Scenario: Sleep when frame completes early
- **GIVEN** an iteration completes in 5ms
- **WHEN** the iteration ends
- **THEN** the system SHALL sleep for approximately 11.67ms before the next iteration

#### Scenario: No sleep when frame is slow
- **GIVEN** an iteration takes 20ms (longer than one timestep)
- **WHEN** the iteration ends
- **THEN** the system SHALL NOT sleep and SHALL immediately start the next iteration
