// Leaderboard capability: command-line options, player name resolution,
// minimal http:// client, score submission, top-score retrieval, and the
// background game-over report polled by the game loop.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

/// Default leaderboard service base URL.
pub const DEFAULT_URL: &str = "http://localhost:8088";
/// Timeout applied separately to connect, read, and write.
pub const TIMEOUT: Duration = Duration::from_millis(1500);
/// The service accepts player names of 1 to 64 characters.
pub const MAX_PLAYER_LEN: usize = 64;
/// How many top scores the game-over screen asks for.
pub const TOP_N: usize = 10;
/// Give up on a response whose headers exceed this many bytes.
pub const MAX_HEADER_BYTES: usize = 64 * 1024;
/// Usage text included in argument errors.
pub const USAGE: &str =
    "usage: tuisteroids [--leaderboard] [--leaderboard-url URL] [--player NAME]";

// === Command-line options ===

/// Parsed command-line options. Without `leaderboard` the game never touches
/// the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub leaderboard: bool,
    pub url: String,
    pub player: String,
}

/// Parse the arguments after the program name. `default_player` is used when
/// `--player` is absent; every player name is normalized.
pub fn parse_args<I: IntoIterator<Item = String>>(
    args: I,
    default_player: &str,
) -> Result<Options, String> {
    let mut options = Options {
        leaderboard: false,
        url: DEFAULT_URL.to_string(),
        player: normalize_player(default_player),
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, value)) => (flag.to_string(), Some(value.to_string())),
            None => (arg.clone(), None),
        };
        match flag.as_str() {
            "--leaderboard" => options.leaderboard = true,
            "--leaderboard-url" => options.url = flag_value(&flag, inline, &mut args)?,
            "--player" => options.player = normalize_player(&flag_value(&flag, inline, &mut args)?),
            _ => return Err(format!("unknown argument: {}\n{}", arg, USAGE)),
        }
    }
    Ok(options)
}

fn flag_value(
    flag: &str,
    inline: Option<String>,
    rest: &mut impl Iterator<Item = String>,
) -> Result<String, String> {
    inline
        .or_else(|| rest.next())
        .ok_or_else(|| format!("{} requires a value\n{}", flag, USAGE))
}

// === Player name ===

/// Trim, cut to 64 characters, and fall back to "player" when empty.
pub fn normalize_player(name: &str) -> String {
    let trimmed: String = name.trim().chars().take(MAX_PLAYER_LEN).collect();
    if trimmed.is_empty() {
        "player".to_string()
    } else {
        trimmed
    }
}

/// USER, else LOGNAME, else "player" (pure; see `default_player`).
pub fn default_player_from(user: Option<String>, logname: Option<String>) -> String {
    let name = user
        .filter(|user| !user.trim().is_empty())
        .or(logname)
        .unwrap_or_default();
    normalize_player(&name)
}

/// The login name from the environment.
pub fn default_player() -> String {
    default_player_from(std::env::var("USER").ok(), std::env::var("LOGNAME").ok())
}

// === Minimal HTTP/1.1 client (http:// only) ===

/// Status code and body of an HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// Split `http://host[:port][/prefix]` into `host:port` and the path prefix
/// (without a trailing slash).
pub fn parse_base_url(url: &str) -> Result<(String, String), String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("leaderboard URL must start with http://: {}", url))?;
    let (authority, prefix) = match rest.find('/') {
        Some(slash) => (&rest[..slash], rest[slash..].trim_end_matches('/')),
        None => (rest, ""),
    };
    if authority.is_empty() {
        return Err(format!("leaderboard URL has no host: {}", url));
    }
    let authority = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{}:80", authority)
    };
    Ok((authority, prefix.to_string()))
}

/// One HTTP/1.1 request with `Connection: close`; `timeout` bounds connect,
/// read, and write separately. Never panics: every failure is an `Err`.
pub fn request(
    base_url: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
    timeout: Duration,
) -> Result<Response, String> {
    let (authority, prefix) = parse_base_url(base_url)?;
    let addr = authority
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.next())
        .ok_or_else(|| format!("cannot resolve {}", authority))?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout)
        .map_err(|e| format!("connect {}: {}", authority, e))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    let body = body.unwrap_or("");
    let mut request = format!(
        "{} {}{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept: application/json\r\n",
        method, prefix, path, authority
    );
    if !body.is_empty() {
        request.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write: {}", e))?;
    read_response(stream)
}

/// Parse the status line and headers, then read the body: exactly
/// `Content-Length` bytes when given, otherwise everything until the peer
/// closes the connection.
pub fn read_response<R: Read>(mut reader: R) -> Result<Response, String> {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = read_head(&mut reader, &mut buf, &mut chunk)?;
    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or_default();
    let status = status_line
        .split(' ')
        .nth(1)
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| format!("malformed status line: {:?}", status_line))?;
    let content_length = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok());
    let mut body = buf[header_end + 4..].to_vec();
    match content_length {
        Some(len) => {
            let mut open = true;
            while open && body.len() < len {
                let n = read_chunk(&mut reader, &mut chunk)?;
                body.extend_from_slice(&chunk[..n]);
                open = n > 0;
            }
            body.truncate(len);
        }
        None => {
            reader
                .read_to_end(&mut body)
                .map_err(|e| format!("read: {}", e))?;
        }
    }
    let body = String::from_utf8_lossy(&body).to_string();
    Ok(Response { status, body })
}

/// Fill `buf` until the blank line ending the headers; returns its offset.
fn read_head<R: Read>(
    reader: &mut R,
    buf: &mut Vec<u8>,
    chunk: &mut [u8],
) -> Result<usize, String> {
    loop {
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            return Ok(pos);
        }
        if buf.len() > MAX_HEADER_BYTES {
            return Err("response headers too large".to_string());
        }
        let n = read_chunk(reader, chunk)?;
        if n == 0 {
            return Err("connection closed before headers were complete".to_string());
        }
        buf.extend_from_slice(&chunk[..n]);
    }
}

fn read_chunk<R: Read>(reader: &mut R, chunk: &mut [u8]) -> Result<usize, String> {
    reader.read(chunk).map_err(|e| format!("read: {}", e))
}

// === Score submission ===

/// Escape a string for inclusion inside JSON double quotes.
pub fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// `POST /scores`; the service keeps each player's best score.
pub fn submit_score(
    base_url: &str,
    player: &str,
    score: u32,
    timeout: Duration,
) -> Result<(), String> {
    let body = format!(
        "{{\"player\":\"{}\",\"score\":{}}}",
        json_escape(player),
        score
    );
    let response = request(base_url, "POST", "/scores", Some(body.as_str()), timeout)?;
    match response.status {
        204 => Ok(()),
        status => Err(format!("POST /scores: {} {}", status, response.body.trim())),
    }
}

// === Top scores ===

/// One leaderboard row as returned by the service.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub player: String,
    pub score: f64,
}

/// `GET /top?n=N`, highest score first.
pub fn fetch_top(base_url: &str, n: usize, timeout: Duration) -> Result<Vec<Entry>, String> {
    let response = request(base_url, "GET", &format!("/top?n={}", n), None, timeout)?;
    if response.status != 200 {
        return Err(format!(
            "GET /top: {} {}",
            response.status,
            response.body.trim()
        ));
    }
    parse_entries(&response.body)
}

/// Parse `[{"player": "...", "score": 1.0}, ...]`; extra fields are ignored.
pub fn parse_entries(body: &str) -> Result<Vec<Entry>, String> {
    let items = match parse_json(body)? {
        Json::Arr(items) => items,
        _ => return Err("expected a JSON array of scores".to_string()),
    };
    items
        .into_iter()
        .map(|item| {
            let fields = match item {
                Json::Obj(fields) => fields,
                _ => return Err("expected a score object".to_string()),
            };
            let player = match field(&fields, "player") {
                Some(Json::Str(player)) => player.clone(),
                _ => return Err("score object has no player string".to_string()),
            };
            let score = match field(&fields, "score") {
                Some(Json::Num(score)) => *score,
                _ => return Err("score object has no numeric score".to_string()),
            };
            Ok(Entry { player, score })
        })
        .collect()
}

fn field<'a>(fields: &'a [(String, Json)], name: &str) -> Option<&'a Json> {
    fields.iter().find(|(key, _)| key == name).map(|(_, v)| v)
}

/// Whole numbers print without a fractional part.
pub fn format_score(score: f64) -> String {
    if score.fract() == 0.0 {
        (score as i64).to_string()
    } else {
        score.to_string()
    }
}

// Just enough JSON for the service's responses: arrays, objects, strings
// (with escapes), and numbers. Anything else is an error.
#[derive(Debug, Clone, PartialEq)]
enum Json {
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

fn parse_json(text: &str) -> Result<Json, String> {
    let mut parser = Parser { text, pos: 0 };
    let value = parser.value()?;
    parser.skip_whitespace();
    if parser.pos != parser.text.len() {
        return Err(parser.error("trailing characters"));
    }
    Ok(value)
}

impl Parser<'_> {
    fn error(&self, what: &str) -> String {
        format!("invalid JSON at byte {}: {}", self.pos, what)
    }

    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        self.skip_whitespace();
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(&format!("expected {:?}", byte as char)))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'[') => self.array(),
            Some(b'{') => self.object(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.error("expected a value")),
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.pos += 1; // '['
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(self.error("expected ',' or ']'")),
            }
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.pos += 1; // '{'
        let mut fields = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Json::Obj(fields));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.error("expected a string key"));
            }
            let key = self.string()?;
            self.expect(b':')?;
            let value = self.value()?;
            fields.push((key, value));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Json::Obj(fields));
                }
                _ => return Err(self.error("expected ',' or '}'")),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.pos += 1; // opening quote
        let mut out: Vec<u8> = Vec::new();
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| self.error("unterminated string"))?;
            self.pos += 1;
            match byte {
                b'"' => return Ok(String::from_utf8_lossy(&out).to_string()),
                b'\\' => {
                    let mut utf8 = [0u8; 4];
                    out.extend_from_slice(self.escape()?.encode_utf8(&mut utf8).as_bytes());
                }
                _ => out.push(byte),
            }
        }
    }

    fn escape(&mut self) -> Result<char, String> {
        let byte = self
            .peek()
            .ok_or_else(|| self.error("unterminated escape"))?;
        self.pos += 1;
        Ok(match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => {
                let code = self
                    .text
                    .get(self.pos..self.pos + 4)
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .ok_or_else(|| self.error("bad \\u escape"))?;
                self.pos += 4;
                // Lone surrogates are not chars; show a replacement mark.
                char::from_u32(code).unwrap_or('\u{FFFD}')
            }
            _ => return Err(self.error("unknown escape")),
        })
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        while matches!(
            self.peek(),
            Some(b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')
        ) {
            self.pos += 1;
        }
        self.text[start..self.pos]
            .parse::<f64>()
            .map(Json::Num)
            .map_err(|_| self.error("bad number"))
    }
}

// === Background game-over report ===

/// Outcome of a report: the top scores, or why the leaderboard is unavailable.
pub type ReportResult = Result<Vec<Entry>, String>;

/// Where the current report stands.
#[derive(Debug, Clone, PartialEq)]
pub enum ReportState {
    Idle,
    Pending,
    Done(ReportResult),
}

/// Submits the score and fetches the top scores on a background thread; the
/// game loop polls it every frame while on the game-over screen.
pub struct LeaderboardClient {
    pub url: String,
    pub player: String,
    pub timeout: Duration,
    pub state: ReportState,
    pub rx: Option<Receiver<ReportResult>>,
}

impl LeaderboardClient {
    pub fn new(url: &str, player: &str) -> Self {
        Self {
            url: url.to_string(),
            player: player.to_string(),
            timeout: TIMEOUT,
            state: ReportState::Idle,
            rx: None,
        }
    }

    /// Start a report for `score`. Any earlier in-flight report is discarded.
    pub fn start(&mut self, score: u32) {
        let (tx, rx) = mpsc::channel();
        let (url, player, timeout) = (self.url.clone(), self.player.clone(), self.timeout);
        thread::spawn(move || {
            let _ = tx.send(report(&url, &player, score, timeout));
        });
        self.rx = Some(rx);
        self.state = ReportState::Pending;
    }

    /// Pick up the result if it has arrived. Never blocks.
    pub fn poll(&mut self) {
        let received = match &self.rx {
            Some(rx) => rx.try_recv(),
            None => return,
        };
        match received {
            Ok(result) => self.state = ReportState::Done(result),
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                self.state = ReportState::Done(Err("leaderboard thread ended".to_string()));
            }
        }
        self.rx = None;
    }

    /// Lines for the game-over screen (unindented).
    pub fn lines(&self) -> Vec<String> {
        match &self.state {
            ReportState::Idle => Vec::new(),
            ReportState::Pending => vec!["Leaderboard: fetching...".to_string()],
            ReportState::Done(Err(reason)) => vec![format!("Leaderboard unavailable: {}", reason)],
            ReportState::Done(Ok(entries)) if entries.is_empty() => {
                vec!["Leaderboard: no scores yet".to_string()]
            }
            ReportState::Done(Ok(entries)) => {
                let mut lines = vec!["Top scores:".to_string()];
                for (i, entry) in entries.iter().enumerate() {
                    lines.push(format!(
                        "{:>2}. {:<20} {:>8}",
                        i + 1,
                        entry.player,
                        format_score(entry.score)
                    ));
                }
                lines
            }
        }
    }
}

/// Submit, then fetch the top scores. Fails fast: a failed submit skips the fetch.
pub fn report(base_url: &str, player: &str, score: u32, timeout: Duration) -> ReportResult {
    submit_score(base_url, player, score, timeout)?;
    fetch_top(base_url, TOP_N, timeout)
}

/// Test-only HTTP stub shared with game.rs tests. Binds 127.0.0.1:0, so tests
/// never touch the network.
#[cfg(test)]
pub mod test_stub {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    pub const NO_CONTENT: &str = "HTTP/1.1 204 No Content\r\ncontent-length: 0\r\n\r\n";

    /// Build a response with a correct Content-Length.
    pub fn http(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
            status,
            body.len(),
            body
        )
    }

    /// Serve one connection per element: Some(response) is written back,
    /// Some("") closes without answering, None stalls and never answers.
    /// Returns the base URL and the raw requests seen, in order.
    pub fn stub(responses: Vec<Option<String>>) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                log.lock().unwrap().push(read_request(&mut stream));
                match response {
                    Some(bytes) => {
                        let _ = stream.write_all(bytes.as_bytes());
                    }
                    None => thread::sleep(Duration::from_secs(2)),
                }
            }
        });
        (url, seen)
    }

    /// A URL for a local port that was just released, so connecting is refused.
    pub fn refused_url() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        format!("http://{}", addr)
    }

    /// Read one request: headers plus Content-Length bytes of body.
    fn read_request(stream: &mut TcpStream) -> String {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            let n = stream.read(&mut chunk).unwrap_or(0);
            buf.extend_from_slice(&chunk[..n]);
            let complete = match buf.windows(4).position(|w| w == b"\r\n\r\n") {
                Some(pos) => buf.len() >= pos + 4 + content_length(&buf[..pos]),
                None => false,
            };
            if complete || n == 0 {
                break;
            }
        }
        String::from_utf8_lossy(&buf).to_string()
    }

    fn content_length(head: &[u8]) -> usize {
        String::from_utf8_lossy(head)
            .lines()
            .filter_map(|l| l.split_once(':'))
            .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, v)| v.trim().parse::<usize>().ok())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::test_stub::{http, refused_url, stub, NO_CONTENT};
    use super::*;
    use std::thread;
    use std::time::{Duration, Instant};

    const FAST: Duration = Duration::from_millis(300);

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn entry(player: &str, score: f64) -> Entry {
        Entry {
            player: player.to_string(),
            score,
        }
    }

    fn poll_until_done(client: &mut LeaderboardClient) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.state == ReportState::Pending {
            assert!(Instant::now() < deadline, "report did not finish in time");
            client.poll();
            thread::sleep(Duration::from_millis(5));
        }
    }

    // === Requirement: Leaderboard Command-Line Options ===

    // Scenario: No arguments leave the leaderboard disabled
    #[test]
    fn test_no_args_disables_leaderboard() {
        let opts = parse_args(args(&[]), "ian").unwrap();
        assert_eq!(
            opts,
            Options {
                leaderboard: false,
                url: DEFAULT_URL.to_string(),
                player: "ian".to_string(),
            }
        );
        assert_eq!(DEFAULT_URL, "http://localhost:8088");
    }

    // Scenario: --leaderboard enables reporting with defaults
    #[test]
    fn test_leaderboard_flag_enables() {
        let opts = parse_args(args(&["--leaderboard"]), "ian").unwrap();
        assert!(opts.leaderboard);
        assert_eq!(opts.url, DEFAULT_URL);
        assert_eq!(opts.player, "ian");
    }

    // Scenario: --leaderboard-url overrides the service URL
    #[test]
    fn test_leaderboard_url_override() {
        let a = parse_args(
            args(&[
                "--leaderboard",
                "--leaderboard-url",
                "http://localhost:8081",
            ]),
            "ian",
        )
        .unwrap();
        let b = parse_args(
            args(&["--leaderboard", "--leaderboard-url=http://localhost:8081"]),
            "ian",
        )
        .unwrap();
        assert_eq!(a.url, "http://localhost:8081");
        assert!(a.leaderboard);
        assert_eq!(b, a);
    }

    // Scenario: --player overrides the player name
    #[test]
    fn test_player_override() {
        let a = parse_args(args(&["--leaderboard", "--player", "alice"]), "ian").unwrap();
        let b = parse_args(args(&["--leaderboard", "--player=alice"]), "ian").unwrap();
        assert_eq!(a.player, "alice");
        assert_eq!(b, a);
    }

    // Scenario: Flag missing its value is rejected
    #[test]
    fn test_missing_value_rejected() {
        let err = parse_args(args(&["--leaderboard", "--player"]), "ian").unwrap_err();
        assert!(err.contains("--player"), "{}", err);
        assert!(err.contains(USAGE), "{}", err);
        let err = parse_args(args(&["--leaderboard-url"]), "ian").unwrap_err();
        assert!(err.contains("--leaderboard-url"), "{}", err);
    }

    // Scenario: Unknown argument is rejected
    #[test]
    fn test_unknown_argument_rejected() {
        let err = parse_args(args(&["--bogus"]), "ian").unwrap_err();
        assert!(err.contains("--bogus"), "{}", err);
        assert!(err.contains(USAGE), "{}", err);
    }

    // Scenario: Leaderboard options without --leaderboard keep it disabled
    #[test]
    fn test_options_without_flag_stay_disabled() {
        let opts = parse_args(
            args(&[
                "--player",
                "alice",
                "--leaderboard-url",
                "http://localhost:8081",
            ]),
            "ian",
        )
        .unwrap();
        assert!(!opts.leaderboard);
        assert_eq!(opts.player, "alice");
        assert_eq!(opts.url, "http://localhost:8081");
    }

    // === Requirement: Player Name Resolution ===

    // Scenario: USER is preferred
    #[test]
    fn test_user_preferred() {
        assert_eq!(
            default_player_from(Some("ian".to_string()), Some("other".to_string())),
            "ian"
        );
    }

    // Scenario: LOGNAME is used when USER is unset
    #[test]
    fn test_logname_when_user_unset() {
        assert_eq!(
            default_player_from(None, Some("other".to_string())),
            "other"
        );
        assert_eq!(
            default_player_from(Some("  ".to_string()), Some("other".to_string())),
            "other"
        );
    }

    // Scenario: Fallback name when neither is set
    #[test]
    fn test_fallback_player() {
        assert_eq!(default_player_from(None, None), "player");
    }

    // Scenario: Long names are truncated to 64 characters
    #[test]
    fn test_long_name_truncated() {
        let name = normalize_player(&"é".repeat(70));
        assert_eq!(name.chars().count(), 64);
        assert_eq!(name, "é".repeat(64));
        assert_eq!(MAX_PLAYER_LEN, 64);
    }

    // Scenario: Blank names fall back to the default
    #[test]
    fn test_blank_name_falls_back() {
        assert_eq!(normalize_player(""), "player");
        assert_eq!(normalize_player("   "), "player");
        assert_eq!(normalize_player("  bob  "), "bob");
    }

    // Scenario: Environment-derived default is valid
    #[test]
    fn test_env_default_is_valid() {
        let name = default_player();
        assert!((1..=64).contains(&name.chars().count()), "{:?}", name);
    }

    // === Requirement: Minimal HTTP Client ===

    // Scenario: Successful request returns status and body
    #[test]
    fn test_request_success() {
        let (url, _) = stub(vec![Some(http("200 OK", "hello"))]);
        let r = request(&url, "GET", "/", None, FAST).unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.body, "hello");
    }

    // Scenario: Request is well-formed HTTP/1.1
    #[test]
    fn test_request_is_well_formed() {
        let (url, seen) = stub(vec![Some(NO_CONTENT.to_string())]);
        let r = request(&url, "POST", "/scores", Some("{\"a\":1}"), FAST).unwrap();
        assert_eq!(r.status, 204);
        assert_eq!(r.body, "");
        let seen = seen.lock().unwrap();
        let req = &seen[0];
        assert!(req.starts_with("POST /scores HTTP/1.1\r\n"), "{}", req);
        let host = &url["http://".len()..];
        assert!(req.contains(&format!("Host: {}\r\n", host)), "{}", req);
        assert!(req.contains("Connection: close\r\n"), "{}", req);
        assert!(
            req.contains("Content-Type: application/json\r\n"),
            "{}",
            req
        );
        assert!(req.contains("Content-Length: 7\r\n"), "{}", req);
        assert!(req.ends_with("\r\n\r\n{\"a\":1}"), "{}", req);
    }

    // Scenario: Response without Content-Length is read to end of stream
    #[test]
    fn test_response_without_content_length() {
        let (url, _) = stub(vec![Some("HTTP/1.1 200 OK\r\n\r\nstreamed".to_string())]);
        let r = request(&url, "GET", "/", None, FAST).unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.body, "streamed");
    }

    // Scenario: Short body ends at end of stream
    #[test]
    fn test_short_body_ends_at_eof() {
        let (url, _) = stub(vec![Some(
            "HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nabc".to_string(),
        )]);
        let r = request(&url, "GET", "/", None, FAST).unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.body, "abc");
    }

    /// Delivers one byte per read, like a slow peer.
    struct Trickle<'a>(&'a [u8]);

    impl std::io::Read for Trickle<'_> {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            match self.0.split_first() {
                Some((first, rest)) if !out.is_empty() => {
                    out[0] = *first;
                    self.0 = rest;
                    Ok(1)
                }
                _ => Ok(0),
            }
        }
    }

    // Scenario: Body arriving in several reads is reassembled
    #[test]
    fn test_body_in_several_reads() {
        let with_length = read_response(Trickle(
            b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello",
        ))
        .unwrap();
        assert_eq!(with_length.status, 200);
        assert_eq!(with_length.body, "hello");
        let to_eof = read_response(Trickle(b"HTTP/1.1 200 OK\r\n\r\nhello")).unwrap();
        assert_eq!(to_eof.status, 200);
        assert_eq!(to_eof.body, "hello");
    }

    // Scenario: Connection refused yields an error
    #[test]
    fn test_connection_refused() {
        let err = request(&refused_url(), "GET", "/", None, FAST).unwrap_err();
        assert!(err.contains("connect"), "{}", err);
    }

    // Scenario: Unresponsive server yields a timeout error
    #[test]
    fn test_unresponsive_server_times_out() {
        let (url, _) = stub(vec![None]);
        let started = Instant::now();
        let err = request(&url, "GET", "/", None, FAST).unwrap_err();
        let took = started.elapsed();
        assert!(took < Duration::from_secs(3), "took {:?}", took);
        assert!(err.contains("read"), "{}", err);
    }

    // Scenario: Server closing early yields an error
    #[test]
    fn test_server_closes_early() {
        let (url, _) = stub(vec![Some(String::new())]);
        let err = request(&url, "GET", "/", None, FAST).unwrap_err();
        assert!(err.contains("closed"), "{}", err);
    }

    // Scenario: Oversized headers yield an error
    #[test]
    fn test_oversized_headers_rejected() {
        let huge = vec![b'a'; MAX_HEADER_BYTES + 10];
        let err = read_response(&huge[..]).unwrap_err();
        assert!(err.contains("headers"), "{}", err);
    }

    // Scenario: Non-http URL is rejected
    #[test]
    fn test_non_http_url_rejected() {
        let err = request("https://localhost:8088", "GET", "/", None, FAST).unwrap_err();
        assert!(err.contains("http://"), "{}", err);
    }

    // Scenario: Malformed URL is rejected
    #[test]
    fn test_malformed_url_rejected() {
        assert!(request("http://", "GET", "/", None, FAST).is_err());
        assert!(request("http://localhost:notaport", "GET", "/", None, FAST).is_err());
    }

    // Scenario: Malformed status line yields an error
    #[test]
    fn test_malformed_status_line() {
        let (url, _) = stub(vec![Some("garbage\r\n\r\n".to_string())]);
        let err = request(&url, "GET", "/", None, FAST).unwrap_err();
        assert!(err.contains("status line"), "{}", err);
        let err = read_response(&b"HTTP/1.1 abc OK\r\n\r\n"[..]).unwrap_err();
        assert!(err.contains("status line"), "{}", err);
    }

    // Scenario: Base URL path prefix is honored
    #[test]
    fn test_base_url_path_prefix() {
        let (url, seen) = stub(vec![Some(http("200 OK", "[]"))]);
        request(&format!("{}/api/", url), "GET", "/top?n=10", None, FAST).unwrap();
        let seen = seen.lock().unwrap();
        assert!(
            seen[0].starts_with("GET /api/top?n=10 HTTP/1.1\r\n"),
            "{}",
            seen[0]
        );
        assert_eq!(
            parse_base_url("http://example.com/x/").unwrap(),
            ("example.com:80".to_string(), "/x".to_string())
        );
        assert_eq!(
            parse_base_url("http://localhost:8088").unwrap(),
            ("localhost:8088".to_string(), String::new())
        );
    }

    // === Requirement: Score Submission ===

    // Scenario: Score is submitted as JSON
    #[test]
    fn test_submit_score_json() {
        let (url, seen) = stub(vec![Some(NO_CONTENT.to_string())]);
        submit_score(&url, "alice", 1234, FAST).unwrap();
        let seen = seen.lock().unwrap();
        assert!(
            seen[0].starts_with("POST /scores HTTP/1.1\r\n"),
            "{}",
            seen[0]
        );
        assert!(
            seen[0].ends_with("\r\n\r\n{\"player\":\"alice\",\"score\":1234}"),
            "{}",
            seen[0]
        );
    }

    // Scenario: Special characters in the player name are escaped
    #[test]
    fn test_json_escape() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\u000a");
        assert_eq!(json_escape("plain é"), "plain é");
    }

    // Scenario: Rejected score is reported
    #[test]
    fn test_submit_rejected_400() {
        let (url, _) = stub(vec![Some(http(
            "400 Bad Request",
            "want {\"player\": ...}",
        ))]);
        let err = submit_score(&url, "alice", 1, FAST).unwrap_err();
        assert!(err.contains("400"), "{}", err);
        assert!(err.contains("want {\"player\": ...}"), "{}", err);
    }

    // Scenario: Unavailable store is reported
    #[test]
    fn test_submit_unavailable_503() {
        let (url, _) = stub(vec![Some(http(
            "503 Service Unavailable",
            "store unreachable",
        ))]);
        let err = submit_score(&url, "alice", 1, FAST).unwrap_err();
        assert!(err.contains("503"), "{}", err);
    }

    // === Requirement: Top Scores Retrieval ===

    // Scenario: Top scores are returned highest first
    #[test]
    fn test_fetch_top_order() {
        let body = "[{\"player\":\"john\",\"score\":30.0},{\"player\":\"ian\",\"score\":20}]";
        let (url, seen) = stub(vec![Some(http("200 OK", body))]);
        let top = fetch_top(&url, 10, FAST).unwrap();
        assert_eq!(top, vec![entry("john", 30.0), entry("ian", 20.0)]);
        let seen = seen.lock().unwrap();
        assert!(
            seen[0].starts_with("GET /top?n=10 HTTP/1.1\r\n"),
            "{}",
            seen[0]
        );
    }

    // Scenario: Empty leaderboard yields an empty list
    #[test]
    fn test_fetch_top_empty() {
        let (url, _) = stub(vec![Some(http("200 OK", "[]"))]);
        assert_eq!(fetch_top(&url, 10, FAST).unwrap(), Vec::<Entry>::new());
    }

    // Scenario: Non-200 status is reported
    #[test]
    fn test_fetch_top_503() {
        let (url, _) = stub(vec![Some(http(
            "503 Service Unavailable",
            "store unreachable",
        ))]);
        let err = fetch_top(&url, 10, FAST).unwrap_err();
        assert!(err.contains("503"), "{}", err);
    }

    // Scenario: Unparsable body is reported
    #[test]
    fn test_fetch_top_unparsable() {
        let (url, _) = stub(vec![Some(http("200 OK", "not json"))]);
        assert!(fetch_top(&url, 10, FAST).is_err());
    }

    // Scenario: Entries with extra fields are accepted
    #[test]
    fn test_parse_entries_extra_fields() {
        let body = "[{\"rank\":1,\"player\":\"x\",\"score\":1,\"meta\":{\"tags\":[\"a\"]}}]";
        assert_eq!(parse_entries(body).unwrap(), vec![entry("x", 1.0)]);
        assert_eq!(parse_entries(" [ ] ").unwrap(), Vec::<Entry>::new());
    }

    // Scenario: Escaped characters in player names are decoded
    #[test]
    fn test_parse_entries_decodes_escapes() {
        let body =
            "[{\"player\":\"a\\\"b\\\\c\\/\\b\\f\\n\\r\\t\\u00e9\\ud83d\",\"score\":-1.5e1}]";
        let entries = parse_entries(body).unwrap();
        assert_eq!(entries[0].player, "a\"b\\c/\u{8}\u{c}\n\r\té\u{FFFD}");
        assert_eq!(entries[0].score, -15.0);
    }

    // Scenario: Malformed entries are rejected
    #[test]
    fn test_parse_entries_malformed() {
        for body in [
            "{}",
            "[1]",
            "[{\"score\":1}]",
            "[{\"player\":\"x\"}]",
            "[{\"player\":\"x\",\"score\":\"1\"}]",
            "[{\"player\":1,\"score\":1}]",
            "[] x",
            "[true]",
            "[\"unterminated",
            "[\"\\q\"]",
            "[\"\\u12\"]",
            "[\"\\u1",
            "[\"\\",
            "[-]",
            "[{\"player\" 1}]",
            "[1 2]",
            "",
            "[",
            "{\"a\":1",
            "[{\"a\":1,}]",
            "[{1:2}]",
            "[{\"a\":1 \"b\":2}]",
        ] {
            assert!(
                parse_entries(body).is_err(),
                "{:?} should be rejected",
                body
            );
        }
    }

    // === Requirement: Non-Blocking Game-Over Reporting ===

    // Scenario: Report runs in the background
    #[test]
    fn test_report_runs_in_background() {
        let body = "[{\"player\":\"alice\",\"score\":42}]";
        let (url, seen) = stub(vec![
            Some(NO_CONTENT.to_string()),
            Some(http("200 OK", body)),
        ]);
        let mut client = LeaderboardClient::new(&url, "alice");
        client.timeout = FAST;
        assert_eq!(client.state, ReportState::Idle);
        client.start(42);
        assert_eq!(client.state, ReportState::Pending);
        poll_until_done(&mut client);
        assert_eq!(
            client.state,
            ReportState::Done(Ok(vec![entry("alice", 42.0)]))
        );
        let seen = seen.lock().unwrap();
        assert!(
            seen[0].starts_with("POST /scores HTTP/1.1\r\n"),
            "{}",
            seen[0]
        );
        assert!(
            seen[0].ends_with("{\"player\":\"alice\",\"score\":42}"),
            "{}",
            seen[0]
        );
        assert!(
            seen[1].starts_with("GET /top?n=10 HTTP/1.1\r\n"),
            "{}",
            seen[1]
        );
    }

    // Scenario: Pending report shows a fetching line
    #[test]
    fn test_lines_pending() {
        let mut client = LeaderboardClient::new(DEFAULT_URL, "ian");
        client.state = ReportState::Pending;
        assert_eq!(client.lines(), vec!["Leaderboard: fetching...".to_string()]);
    }

    // Scenario: Completed report shows the top scores
    #[test]
    fn test_lines_done() {
        let mut client = LeaderboardClient::new(DEFAULT_URL, "ian");
        client.state = ReportState::Done(Ok(vec![entry("john", 30.0), entry("ian", 20.0)]));
        let lines = client.lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Top scores:");
        assert!(lines[1].starts_with(" 1. john"), "{}", lines[1]);
        assert!(lines[1].ends_with("30"), "{}", lines[1]);
        assert!(lines[2].starts_with(" 2. ian"), "{}", lines[2]);
        assert!(lines[2].ends_with("20"), "{}", lines[2]);
    }

    // Scenario: Whole-number scores display without a fraction
    #[test]
    fn test_format_score() {
        assert_eq!(format_score(30.0), "30");
        assert_eq!(format_score(12.5), "12.5");
    }

    // Scenario: Empty leaderboard shows a no-scores line
    #[test]
    fn test_lines_empty() {
        let mut client = LeaderboardClient::new(DEFAULT_URL, "ian");
        client.state = ReportState::Done(Ok(Vec::new()));
        assert_eq!(
            client.lines(),
            vec!["Leaderboard: no scores yet".to_string()]
        );
    }

    // Scenario: Unavailable service shows one quiet line
    #[test]
    fn test_unavailable_single_line() {
        let mut client = LeaderboardClient::new(&refused_url(), "ian");
        client.timeout = FAST;
        client.start(7);
        poll_until_done(&mut client);
        let lines = client.lines();
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].starts_with("Leaderboard unavailable:"),
            "{}",
            lines[0]
        );
    }

    // Scenario: Timeout does not stall the game
    #[test]
    fn test_timeout_does_not_stall() {
        let (url, _) = stub(vec![None]);
        let mut client = LeaderboardClient::new(&url, "ian");
        client.timeout = FAST;
        client.start(1);
        let started = Instant::now();
        let mut slowest = Duration::ZERO;
        while client.state == ReportState::Pending {
            assert!(started.elapsed() < Duration::from_secs(5), "report hung");
            let t = Instant::now();
            client.poll();
            slowest = slowest.max(t.elapsed());
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            slowest < Duration::from_millis(100),
            "poll blocked {:?}",
            slowest
        );
        assert!(matches!(client.state, ReportState::Done(Err(_))));
    }

    // Scenario: Ended background thread is reported as unavailable
    #[test]
    fn test_thread_ended_reported() {
        let mut client = LeaderboardClient::new(DEFAULT_URL, "ian");
        let (tx, rx) = std::sync::mpsc::channel();
        drop(tx);
        client.rx = Some(rx);
        client.state = ReportState::Pending;
        client.poll();
        let ended = ReportState::Done(Err("leaderboard thread ended".to_string()));
        assert_eq!(client.state, ended);
        assert!(client.rx.is_none());
        client.poll(); // no receiver left: nothing changes
        assert_eq!(client.state, ended);
    }

    // Scenario: A new game over replaces an earlier report
    #[test]
    fn test_restart_replaces_report() {
        let body = "[{\"player\":\"ian\",\"score\":2}]";
        let (url, _) = stub(vec![
            Some(NO_CONTENT.to_string()),
            Some(http("200 OK", body)),
        ]);
        let mut client = LeaderboardClient::new(&refused_url(), "ian");
        client.timeout = FAST;
        client.start(1);
        client.url = url;
        client.start(2);
        assert_eq!(client.state, ReportState::Pending);
        poll_until_done(&mut client);
        assert_eq!(client.state, ReportState::Done(Ok(vec![entry("ian", 2.0)])));
    }

    // Scenario: Idle client shows nothing
    #[test]
    fn test_lines_idle() {
        let client = LeaderboardClient::new(DEFAULT_URL, "ian");
        assert_eq!(client.state, ReportState::Idle);
        assert!(client.lines().is_empty());
        assert_eq!(client.timeout, TIMEOUT);
        assert!(TIMEOUT <= Duration::from_secs(2));
        assert_eq!(client.url, DEFAULT_URL);
        assert_eq!(client.player, "ian");
    }
}
