## 1. Spec Deltas
- [x] 1.1 Create leaderboard spec (new capability: options, player name, HTTP client, submit, top, background report)
- [x] 1.2 Create game-loop spec delta (ADDED: Game Over Leaderboard Reporting)
- [x] 1.3 Create design.md with architectural decisions
- [x] 1.4 Validate with openspec validate add-leaderboard --strict

## 2. Tests From Scenarios (red)
- [x] 2.1 Add `pub mod leaderboard;` to src/lib.rs and a test-only stub HTTP server on 127.0.0.1:0 in src/leaderboard.rs
- [x] 2.2 Write tests for Leaderboard Command-Line Options scenarios
- [x] 2.3 Write tests for Player Name Resolution scenarios
- [x] 2.4 Write tests for Minimal HTTP Client scenarios (success, request shape, no Content-Length, short body, refused, timeout, early close, oversized headers, bad scheme, bad URL, bad status line, path prefix)
- [x] 2.5 Write tests for Score Submission scenarios (204, escaping, 400, 503)
- [x] 2.6 Write tests for Top Scores Retrieval scenarios (order, empty, 503, unparsable, extra fields, escapes, malformed)
- [x] 2.7 Write tests for Non-Blocking Game-Over Reporting scenarios (pending, done, formatting, empty, unavailable, timeout, thread ended, restart, idle)
- [x] 2.8 Write tests in src/game.rs for Game Over Leaderboard Reporting scenarios
- [x] 2.9 Run cargo test --lib and confirm the new tests fail to compile/pass

## 3. Implementation (green)
- [x] 3.1 Options, USAGE, parse_args, normalize_player, default_player(_from)
- [x] 3.2 parse_base_url, read_response, request (TcpStream, timeouts, Connection: close)
- [x] 3.3 json_escape, submit_score
- [x] 3.4 JSON subset parser, parse_entries, fetch_top, format_score
- [x] 3.5 LeaderboardClient { url, player, timeout, state, rx } with start/poll/lines; report()
- [x] 3.6 Game: leaderboard field, game_over() starts report, poll_leaderboard(), game_over_lines()
- [x] 3.7 run(options): build client when enabled, poll each GameOver frame, render game_over_lines()
- [x] 3.8 main.rs: parse std::env::args with default_player(); print usage error and exit 2 on bad args

## 4. Verification
- [x] 4.1 cargo fmt -- --check
- [x] 4.2 cargo clippy --lib --bins -- -D warnings
- [x] 4.3 cargo test --lib (all pass, no network)
- [x] 4.4 cargo tarpaulin --lib --timeout 120 --skip-clean: src/leaderboard.rs and src/game.rs at 100%
- [x] 4.5 README: document --leaderboard, --leaderboard-url, --player
