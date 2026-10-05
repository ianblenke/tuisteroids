## ADDED Requirements

### Requirement: Leaderboard Command-Line Options
The system SHALL accept optional command-line arguments controlling the leaderboard: `--leaderboard` enables reporting; `--leaderboard-url <URL>` (also `--leaderboard-url=<URL>`) sets the service base URL, defaulting to `http://localhost:8088`; `--player <NAME>` (also `--player=<NAME>`) sets the player name, defaulting to the resolved login name. Without `--leaderboard` the leaderboard SHALL be disabled regardless of the other flags, and the game SHALL open no network connections. An unknown argument, or a flag given without its value, SHALL be rejected with an error message that includes the usage text.

#### Scenario: No arguments leave the leaderboard disabled
- **GIVEN** no command-line arguments
- **WHEN** the arguments are parsed with default player "ian"
- **THEN** the leaderboard SHALL be disabled, the URL SHALL be `http://localhost:8088`, and the player SHALL be "ian"

#### Scenario: --leaderboard enables reporting with defaults
- **GIVEN** the argument `--leaderboard`
- **WHEN** the arguments are parsed
- **THEN** the leaderboard SHALL be enabled with the default URL and the default player

#### Scenario: --leaderboard-url overrides the service URL
- **GIVEN** `--leaderboard --leaderboard-url http://localhost:8081` or `--leaderboard --leaderboard-url=http://localhost:8081`
- **WHEN** the arguments are parsed
- **THEN** the URL SHALL be `http://localhost:8081`

#### Scenario: --player overrides the player name
- **GIVEN** `--leaderboard --player alice` or `--leaderboard --player=alice`
- **WHEN** the arguments are parsed
- **THEN** the player SHALL be "alice"

#### Scenario: Flag missing its value is rejected
- **GIVEN** `--leaderboard --player` with nothing following
- **WHEN** the arguments are parsed
- **THEN** parsing SHALL fail with a message naming the flag and containing the usage text

#### Scenario: Unknown argument is rejected
- **GIVEN** the argument `--bogus`
- **WHEN** the arguments are parsed
- **THEN** parsing SHALL fail with a message naming `--bogus` and containing the usage text

#### Scenario: Leaderboard options without --leaderboard keep it disabled
- **GIVEN** `--player alice --leaderboard-url http://localhost:8081` without `--leaderboard`
- **WHEN** the arguments are parsed
- **THEN** the leaderboard SHALL remain disabled while the URL and player values are still recorded

### Requirement: Player Name Resolution
The default player name SHALL be the `USER` environment variable, else `LOGNAME`, else `player`. Every player name, whether defaulted or given with `--player`, SHALL be normalized: surrounding whitespace trimmed, truncated to 64 characters (Unicode scalar values, not bytes), and replaced by `player` when the result is empty.

#### Scenario: USER is preferred
- **GIVEN** USER is "ian" and LOGNAME is "other"
- **WHEN** the default player name is resolved
- **THEN** the name SHALL be "ian"

#### Scenario: LOGNAME is used when USER is unset
- **GIVEN** USER is unset or blank and LOGNAME is "other"
- **WHEN** the default player name is resolved
- **THEN** the name SHALL be "other"

#### Scenario: Fallback name when neither is set
- **GIVEN** neither USER nor LOGNAME is set
- **WHEN** the default player name is resolved
- **THEN** the name SHALL be "player"

#### Scenario: Long names are truncated to 64 characters
- **GIVEN** a player name of 70 characters (including multi-byte characters)
- **WHEN** the name is normalized
- **THEN** the result SHALL be exactly the first 64 characters

#### Scenario: Blank names fall back to the default
- **GIVEN** a player name that is empty or only whitespace
- **WHEN** the name is normalized
- **THEN** the result SHALL be "player"

#### Scenario: Environment-derived default is valid
- **GIVEN** the current process environment
- **WHEN** the default player name is resolved from it
- **THEN** the result SHALL be between 1 and 64 characters long

### Requirement: Minimal HTTP Client
The system SHALL talk to the leaderboard service through a minimal HTTP/1.1 client built on `std::net::TcpStream`, supporting only `http://` URLs. Each connection SHALL apply connect, read, and write timeouts no longer than 2 seconds (default 1.5 seconds). Requests SHALL carry `Host` and `Connection: close` headers and, when a body is sent, `Content-Type: application/json` and `Content-Length`. The client SHALL return the status code and body, honoring `Content-Length` when present and otherwise reading to the end of the stream. Every failure (unsupported scheme, malformed URL, unresolvable host, connection refused, timeout, premature close, oversized or malformed response) SHALL be returned as an error value; the client SHALL NOT panic.

#### Scenario: Successful request returns status and body
- **GIVEN** a stub server that answers `HTTP/1.1 200 OK` with a `Content-Length` body of `hello`
- **WHEN** a GET request is made
- **THEN** the result SHALL have status 200 and body `hello`

#### Scenario: Request is well-formed HTTP/1.1
- **GIVEN** a stub server recording what it receives
- **WHEN** a POST to `/scores` with body `{"a":1}` is made
- **THEN** the request SHALL start with `POST /scores HTTP/1.1`, include `Host`, `Connection: close`, `Content-Type: application/json`, `Content-Length: 7`, and end with the body

#### Scenario: Response without Content-Length is read to end of stream
- **GIVEN** a stub server that answers without a `Content-Length` header and then closes the connection
- **WHEN** a request is made
- **THEN** the body SHALL contain everything sent before the close

#### Scenario: Short body ends at end of stream
- **GIVEN** a stub server that declares `Content-Length: 100` but sends only `abc` before closing
- **WHEN** a request is made
- **THEN** the body SHALL be `abc` and no error SHALL be raised

#### Scenario: Body arriving in several reads is reassembled
- **GIVEN** a response stream delivered one byte per read, with and without `Content-Length`
- **WHEN** the response is read
- **THEN** the status SHALL be 200 and the body SHALL be the complete `hello` in both cases

#### Scenario: Connection refused yields an error
- **GIVEN** a URL pointing at a local port with no listener
- **WHEN** a request is made
- **THEN** the result SHALL be an error mentioning the connection failure

#### Scenario: Unresponsive server yields a timeout error
- **GIVEN** a stub server that accepts the connection but never responds, and a client timeout of 300 ms
- **WHEN** a request is made
- **THEN** the result SHALL be an error within a few seconds rather than hanging

#### Scenario: Server closing early yields an error
- **GIVEN** a stub server that closes the connection before sending any headers
- **WHEN** a request is made
- **THEN** the result SHALL be an error

#### Scenario: Oversized headers yield an error
- **GIVEN** a response stream with more than 64 KiB and no end of headers
- **WHEN** the response is read
- **THEN** the result SHALL be an error

#### Scenario: Non-http URL is rejected
- **GIVEN** the base URL `https://localhost:8088`
- **WHEN** a request is made
- **THEN** the result SHALL be an error stating that only `http://` is supported, without opening a connection

#### Scenario: Malformed URL is rejected
- **GIVEN** the base URL `http://` (no host) or `http://localhost:notaport`
- **WHEN** a request is made
- **THEN** the result SHALL be an error

#### Scenario: Malformed status line yields an error
- **GIVEN** a stub server that answers `garbage` instead of an HTTP status line
- **WHEN** a request is made
- **THEN** the result SHALL be an error mentioning the status line

#### Scenario: Base URL path prefix is honored
- **GIVEN** the base URL `http://127.0.0.1:PORT/api/` (trailing slash)
- **WHEN** a GET of `/top?n=10` is made
- **THEN** the request line SHALL be `GET /api/top?n=10 HTTP/1.1`

### Requirement: Score Submission
When enabled, the system SHALL submit the final score with `POST /scores` and the JSON body `{"player":"<name>","score":<score>}`, where the player name is JSON-escaped (quotes, backslashes, and control characters). A 204 response SHALL be treated as success. Any other status SHALL be returned as an error that includes the status code and the response body text.

#### Scenario: Score is submitted as JSON
- **GIVEN** a stub server answering 204
- **WHEN** player "alice" submits score 1234
- **THEN** the stub SHALL receive `POST /scores` with body exactly `{"player":"alice","score":1234}` and the result SHALL be success

#### Scenario: Special characters in the player name are escaped
- **GIVEN** the player name `a"b\c` followed by a newline
- **WHEN** it is JSON-escaped
- **THEN** the result SHALL be `a\"b\\c\u000a`

#### Scenario: Rejected score is reported
- **GIVEN** a stub server answering 400 with body `want {"player": ...}`
- **WHEN** a score is submitted
- **THEN** the result SHALL be an error containing `400` and the body text

#### Scenario: Unavailable store is reported
- **GIVEN** a stub server answering 503 with body `store unreachable`
- **WHEN** a score is submitted
- **THEN** the result SHALL be an error containing `503`

### Requirement: Top Scores Retrieval
When enabled, the system SHALL request `GET /top?n=10` and parse the JSON array of `{"player": <string>, "score": <number>}` objects into entries, preserving the service's order (highest first). An empty array SHALL yield an empty list. Objects MAY carry extra fields, which SHALL be ignored. JSON string escapes SHALL be decoded. A non-200 status or a body that is not an array of such objects SHALL be returned as an error.

#### Scenario: Top scores are returned highest first
- **GIVEN** a stub server answering `[{"player":"john","score":30.0},{"player":"ian","score":20}]`
- **WHEN** the top scores are fetched
- **THEN** the result SHALL be the two entries in that order with scores 30 and 20

#### Scenario: Empty leaderboard yields an empty list
- **GIVEN** a stub server answering `[]`
- **WHEN** the top scores are fetched
- **THEN** the result SHALL be an empty list

#### Scenario: Non-200 status is reported
- **GIVEN** a stub server answering 503
- **WHEN** the top scores are fetched
- **THEN** the result SHALL be an error containing `503`

#### Scenario: Unparsable body is reported
- **GIVEN** a stub server answering 200 with body `not json`
- **WHEN** the top scores are fetched
- **THEN** the result SHALL be an error

#### Scenario: Entries with extra fields are accepted
- **GIVEN** the body `[{"rank":1,"player":"x","score":1,"meta":{"tags":["a"]}}]`
- **WHEN** it is parsed
- **THEN** the result SHALL be one entry for "x" with score 1

#### Scenario: Escaped characters in player names are decoded
- **GIVEN** a player name encoded as `"a\"b\\c\/\b\f\n\r\té\ud83d"`
- **WHEN** it is parsed
- **THEN** the decoded name SHALL contain the quote, backslash, slash, control characters, `é`, and U+FFFD for the lone surrogate

#### Scenario: Malformed entries are rejected
- **GIVEN** bodies such as `{}`, `[1]`, `[{"score":1}]`, `[{"player":"x","score":"1"}]`, `[] x`, `[true]`, `["unterminated`, `["\q"]`, `["\u12"]`, `[-]`, `[{"player" 1}]`, `[1 2]`, and an empty body
- **WHEN** each is parsed
- **THEN** each SHALL yield an error

### Requirement: Non-Blocking Game-Over Reporting
When enabled, score submission followed by top-score retrieval SHALL run on a background thread started at game over, and SHALL never block the game loop. Polling for the result SHALL be non-blocking. The game-over screen lines SHALL show `Leaderboard: fetching...` while the report is in flight; `Top scores:` followed by one line per entry (rank, player, score) when retrieved; `Leaderboard: no scores yet` when the list is empty; and a single line `Leaderboard unavailable: <reason>` on any failure. Whole-number scores SHALL be shown without a fractional part. The system SHALL NOT retry. Starting a new report SHALL discard any earlier in-flight result.

#### Scenario: Report runs in the background
- **GIVEN** a client for a stub server that answers 204 then a top list
- **WHEN** the report is started for score 42
- **THEN** the client SHALL be pending immediately, SHALL become done after polling, and the stub SHALL have received `POST /scores` with score 42 followed by `GET /top?n=10`

#### Scenario: Pending report shows a fetching line
- **GIVEN** a client whose report is pending
- **WHEN** the leaderboard lines are built
- **THEN** they SHALL be exactly `["Leaderboard: fetching..."]`

#### Scenario: Completed report shows the top scores
- **GIVEN** a completed report with entries john 30 and ian 20
- **WHEN** the leaderboard lines are built
- **THEN** the first line SHALL be `Top scores:` followed by ` 1. john` and ` 2. ian` lines carrying their scores

#### Scenario: Whole-number scores display without a fraction
- **GIVEN** scores 30.0 and 12.5
- **WHEN** they are formatted
- **THEN** they SHALL read `30` and `12.5`

#### Scenario: Empty leaderboard shows a no-scores line
- **GIVEN** a completed report with no entries
- **WHEN** the leaderboard lines are built
- **THEN** they SHALL be exactly `["Leaderboard: no scores yet"]`

#### Scenario: Unavailable service shows one quiet line
- **GIVEN** a client pointed at a port with no listener
- **WHEN** the report is started and polled to completion
- **THEN** the lines SHALL be a single line starting with `Leaderboard unavailable:`

#### Scenario: Timeout does not stall the game
- **GIVEN** a client with a 300 ms timeout pointed at a stub that never responds
- **WHEN** the report is started and polled every few milliseconds
- **THEN** each poll SHALL return immediately and the report SHALL end as unavailable within a few seconds

#### Scenario: Ended background thread is reported as unavailable
- **GIVEN** a pending client whose result channel has been closed without a result
- **WHEN** it is polled
- **THEN** the report SHALL be done with an unavailable reason

#### Scenario: A new game over replaces an earlier report
- **GIVEN** a client whose first report targets a refused port
- **WHEN** a second report is started against a working stub and polled to completion
- **THEN** the client SHALL be pending right after the second start and the final state SHALL hold the stub's top scores

#### Scenario: Idle client shows nothing
- **GIVEN** a client that has never started a report
- **WHEN** the leaderboard lines are built
- **THEN** they SHALL be empty
