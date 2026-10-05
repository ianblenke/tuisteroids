## Context

The game loop in `game::run()` is UI-blocking: it polls input, updates at 60 Hz, and renders every frame. Any network call made inline would freeze rendering for the duration of the timeout. The leaderboard service is a FastAPI app with two routes: `POST /scores` (JSON `{"player": "<1-64 chars>", "score": <finite number>}` -> 204, 400 on bad input, 503 when its store is down) and `GET /top?n=10` (JSON array of `{"player", "score"}` highest first, `[]` when empty, 400 for bad `n`, 503 when the store is down). No authentication. The crate currently has no HTTP or JSON dependency.

## Goals / Non-Goals

- **Goals:**
  - Opt-in via `--leaderboard`; identical behaviour without it (no sockets opened, no new config)
  - Submit the final score at game over and show the top ten on the game-over screen
  - Never crash or stall the game on network failure; one quiet line instead
  - Tests run fully offline against a local stub `TcpListener`
  - Keep the change small: one new module, no new crates

- **Non-Goals:**
  - HTTPS, redirects, keep-alive, chunked transfer decoding, proxies
  - Authentication, retries, offline queueing of scores
  - A general-purpose JSON library (only the shape the service returns)
  - In-game name entry UI (the name comes from `--player` or the login name)

## Decisions

### HTTP client: hand-rolled HTTP/1.1 over std::net::TcpStream
- **Decision**: Write ~60 lines: resolve host, `TcpStream::connect_timeout`, set read/write timeouts, send a request with `Connection: close`, parse the status line and `Content-Length`, read the body (or to end of stream when no length is given).
- **Why**: The task forbids adding an HTTP crate when none exists. The service speaks plain HTTP with small JSON bodies, so a tiny client suffices and keeps compile time and the dependency tree unchanged.
- **Alternative**: `ureq`/`reqwest` — rejected (new dependency, heavier build).

### JSON: minimal subset parser plus a string escaper
- **Decision**: Parse arrays, objects, strings (with the standard escapes incl. `\uXXXX`), and numbers; anything else is an error. Extract `player` (string) and `score` (number) from each object; extra fields are ignored. Outgoing bodies are built with `format!` and a small escaper for `"`, `\` and control characters.
- **Why**: `serde_json` would be another dependency. The response shape is fixed and small.
- **Trade-off**: Lone UTF-16 surrogate escapes decode to U+FFFD rather than being combined; the service emits raw UTF-8, so `\u` escapes appear only for control characters in practice.

### Concurrency: one background thread per game over, polled through a channel
- **Decision**: `LeaderboardClient::start(score)` spawns a `std::thread` that runs submit-then-fetch and sends the `Result<Vec<Entry>, String>` over an `mpsc` channel. The loop calls `poll()` (non-blocking `try_recv`) each frame while in GameOver and renders the current state: pending, top scores, or unavailable. A new game over replaces the receiver, so a late result from an earlier thread is dropped.
- **Why**: Rendering never waits on the network. A thread per game over is trivially cheap and avoids a long-lived worker. Timeouts bound the thread's lifetime (connect + write + read, each 1.5 s, twice).
- **Alternative**: Do the calls synchronously at game over with a tight timeout — rejected, a 1.5 s freeze on every death is noticeable.
- **Alternative**: Submit asynchronously but skip fetching — rejected, showing the top scores is the point.

### Fail-fast report
- **Decision**: If `POST /scores` fails, `GET /top` is not attempted and the screen shows `Leaderboard unavailable: <reason>`.
- **Why**: A failed submit almost always means the service is down; one bounded round-trip is enough, and one line is all the screen needs.

### Player name
- **Decision**: `--player NAME`, defaulting to `$USER`, then `$LOGNAME`, then `player`. Names are trimmed, truncated to 64 characters (code points, matching the service's `max_length`), and an empty result becomes `player`.
- **Why**: Matches the service's 1-64 character rule without an error path the player would have to fix at launch.

### Code placement
- **Decision**: All leaderboard logic lives in `src/leaderboard.rs`, including argument parsing (the only flags the binary has are leaderboard flags). `Game` gains `pub leaderboard: Option<LeaderboardClient>`; `Game::game_over()` starts the report; `Game::game_over_lines()` builds the game-over text so it is testable with and without a client. `run()` (already excluded from coverage) only polls and renders.

## Risks / Trade-offs

- **DNS resolution has no timeout** → `to_socket_addrs` is called on the background thread, so even a slow resolver cannot stall rendering; the default URL is `localhost`.
- **Worst-case thread lifetime ~9 s** (two requests, three 1.5 s timeouts each) → bounded, off the render thread, and dropped on the next game over.
- **No chunked decoding** → uvicorn sends `Content-Length` for small JSON responses; if a proxy chunks the body, parsing fails and the screen shows "unavailable" rather than crashing.
- **Port reuse race in the connection-refused test** → the test binds `127.0.0.1:0`, records the port, and drops the listener; another process grabbing that port within microseconds is negligible.

## Migration Plan
Additive and opt-in. `game::run()` now takes `Options`; `main.rs` is the only caller. No data migration; no rollback concerns beyond reverting the change.

## Open Questions
- None. The live service currently answers 404 for `/scores` and `/top` (image pending a CI fix); until that lands, only the stub-based tests exercise the client.
