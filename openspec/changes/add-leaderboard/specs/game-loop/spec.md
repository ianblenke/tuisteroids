## ADDED Requirements

### Requirement: Game Over Leaderboard Reporting
When the game transitions to GameOver and a leaderboard client is configured, the system SHALL start a background leaderboard report for the final score and the game-over screen text SHALL include the leaderboard lines (indented like the other lines) between the score and the restart prompt. When no leaderboard client is configured, which is the default without `--leaderboard`, the game SHALL behave exactly as before: no report is started, no network activity occurs, and the game-over screen text is unchanged. Polling the leaderboard SHALL be a non-blocking no-op when no client is configured.

#### Scenario: Game over without a leaderboard client makes no report
- **GIVEN** a game created without a leaderboard client
- **WHEN** the game transitions to GameOver
- **THEN** no leaderboard client SHALL exist and the final score SHALL be recorded as before

#### Scenario: Game over with a leaderboard client submits the final score
- **GIVEN** a game with a leaderboard client for a stub server and a playing score of 1234
- **WHEN** the game transitions to GameOver and the leaderboard is polled to completion
- **THEN** the stub SHALL have received `POST /scores` with body `{"player":"alice","score":1234}` and the client SHALL hold the top scores

#### Scenario: Game-over screen is unchanged without a leaderboard
- **GIVEN** a game over with final score 5000 and no leaderboard client
- **WHEN** the game-over lines are built
- **THEN** they SHALL be exactly `["", "", "    GAME OVER", "", "    Score: 5000", "", "    Press any key to restart or Q to quit"]`

#### Scenario: Game-over screen includes leaderboard lines
- **GIVEN** a game over with a pending leaderboard report
- **WHEN** the game-over lines are built
- **THEN** they SHALL contain `    Leaderboard: fetching...` after the score line and before the restart prompt

#### Scenario: Polling the leaderboard without a client is a no-op
- **GIVEN** a game without a leaderboard client
- **WHEN** the leaderboard is polled
- **THEN** nothing SHALL change and the call SHALL return immediately
