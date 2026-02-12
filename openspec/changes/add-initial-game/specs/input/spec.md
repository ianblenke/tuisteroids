## ADDED Requirements

### Requirement: Input Action Abstraction
The system SHALL abstract raw keyboard events into game actions. The input system SHALL produce an `InputState` representing which actions are currently active (pressed) on each frame.

The defined game actions SHALL be:
- `RotateLeft` — rotate ship counterclockwise
- `RotateRight` — rotate ship clockwise
- `Thrust` — apply forward thrust
- `Fire` — shoot a bullet
- `Quit` — exit the game

#### Scenario: No keys pressed produces empty input state
- **GIVEN** no keyboard keys are pressed
- **WHEN** input is polled
- **THEN** all actions in the InputState SHALL be inactive

#### Scenario: Multiple simultaneous keys produce combined input state
- **GIVEN** the left arrow key and spacebar are both pressed simultaneously
- **WHEN** input is polled
- **THEN** both RotateLeft and Fire SHALL be active, all other actions SHALL be inactive

### Requirement: Key Mapping
The system SHALL map keyboard keys to game actions as follows:
- Left arrow key → `RotateLeft`
- Right arrow key → `RotateRight`
- Up arrow key → `Thrust`
- Spacebar → `Fire`
- "q" key → `Quit`

#### Scenario: Left arrow maps to RotateLeft
- **GIVEN** the left arrow key is pressed
- **WHEN** input is processed
- **THEN** the `RotateLeft` action SHALL be active

#### Scenario: Right arrow maps to RotateRight
- **GIVEN** the right arrow key is pressed
- **WHEN** input is processed
- **THEN** the `RotateRight` action SHALL be active

#### Scenario: Up arrow maps to Thrust
- **GIVEN** the up arrow key is pressed
- **WHEN** input is processed
- **THEN** the `Thrust` action SHALL be active

#### Scenario: Spacebar maps to Fire
- **GIVEN** the spacebar is pressed
- **WHEN** input is processed
- **THEN** the `Fire` action SHALL be active

#### Scenario: Q key maps to Quit
- **GIVEN** the "q" key is pressed
- **WHEN** input is processed
- **THEN** the `Quit` action SHALL be active

#### Scenario: Unmapped key is ignored
- **GIVEN** the "x" key is pressed
- **WHEN** input is processed
- **THEN** no game action SHALL be active

### Requirement: Non-Blocking Input Polling
The system SHALL poll for keyboard input without blocking the game loop. If no input event is available, the system SHALL return immediately with the current input state unchanged.

#### Scenario: Input polling does not block when no event
- **GIVEN** no keyboard event is pending
- **WHEN** input is polled with a zero timeout
- **THEN** the poll SHALL return immediately without blocking

#### Scenario: Input polling captures pending event
- **GIVEN** a left arrow key press event is pending
- **WHEN** input is polled
- **THEN** the event SHALL be consumed and RotateLeft SHALL be active

### Requirement: Fire Action Edge Detection
The system SHALL treat the Fire action as an edge-triggered event (activates only on key-down, not while held). Holding the spacebar SHALL fire only one bullet until the key is released and pressed again.

#### Scenario: Spacebar press fires once
- **GIVEN** the spacebar was not pressed last frame and is pressed this frame
- **WHEN** input is processed
- **THEN** the Fire action SHALL be active (edge detected)

#### Scenario: Spacebar held does not re-fire
- **GIVEN** the spacebar was pressed last frame and is still pressed this frame
- **WHEN** input is processed
- **THEN** the Fire action SHALL NOT be active (no new edge)

#### Scenario: Spacebar released and pressed again fires
- **GIVEN** the spacebar was released last frame and is pressed this frame
- **WHEN** input is processed
- **THEN** the Fire action SHALL be active (new edge detected)
