# Change: Add optional leaderboard reporting

## Why
Scores currently vanish when the game-over screen is dismissed. A small leaderboard service (`POST /scores`, `GET /top`) already exists; letting the game report the final score and show the top scores on the game-over screen gives players something to compete for. The feature must be opt-in: without the flag the game behaves exactly as today, with no network activity and no new required configuration.

## What Changes
- Add `leaderboard` capability (new): command-line options (`--leaderboard`, `--leaderboard-url URL`, `--player NAME`), player-name resolution from the login name, a minimal `http://`-only HTTP/1.1 client on `std::net::TcpStream` with short timeouts (no new dependency), score submission, top-score retrieval with a tiny JSON subset parser, and a background-thread report whose result is polled by the game loop
- Modify `game-loop` capability (ADDED requirement): on game over, when a leaderboard client is configured, the final score is reported in the background and the game-over screen shows the top scores (or a single quiet "Leaderboard unavailable" line); without a client the game-over screen is unchanged

## Impact
- Affected specs: leaderboard (new), game-loop (added requirement)
- Affected code: src/leaderboard.rs (new), src/game.rs (Game gains an optional client; run() takes Options), src/main.rs (argument parsing), src/lib.rs (module), README.md (usage)
- No new crate dependencies; no network calls unless `--leaderboard` is given
