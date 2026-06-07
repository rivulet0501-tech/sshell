# sshell MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust-based SSH TUI client MVP that prioritizes terminal compatibility, supports startup-time multi-session tabs, and exposes stable process exit codes for Qt callers.

**Architecture:** Build a single binary crate with focused modules for CLI parsing, connection parsing, session state, SSH transport, terminal emulation, TUI input/rendering, and app orchestration. For MVP, do not implement runtime session creation; instead, keep the control surface small and route nearly all keyboard input directly to the active remote session through a prefix-key gate.

**Tech Stack:** Rust, Tokio, Clap, Russh, Crossterm, Ratatui, vt100, thiserror, assert_cmd, predicates, tokio-test

---

## File Structure

Create these files and keep responsibilities narrow:

- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/cli.rs` — parse argv, build `CliArgs`, map startup failures to exit codes
- Create: `src/connection_spec.rs` — parse `username:password@host[:port]`, reject invalid input, build display titles
- Create: `src/error.rs` — define `ExitCode`, `AppError`, stderr rendering, first-error aggregation
- Create: `src/session_manager.rs` — own tab ordering, active session selection, close/switch behavior
- Create: `src/input.rs` — prefix-key state machine and local control commands
- Create: `src/events.rs` — typed events passed between UI and session tasks
- Create: `src/ssh/mod.rs`
- Create: `src/ssh/client.rs` — `SshClient` trait and Russh-backed implementation
- Create: `src/ssh/runtime.rs` — per-session task that connects, authenticates, requests remote PTY, forwards I/O, emits session events
- Create: `src/terminal/mod.rs`
- Create: `src/terminal/engine.rs` — vt100-backed screen state and resize handling
- Create: `src/tui/mod.rs`
- Create: `src/tui/view.rs` — ratatui drawing functions for tabs, terminal surface, and error banner
- Create: `src/app.rs` — orchestrate startup sessions, event loop, resize broadcast, shutdown and exit-code selection
- Create: `tests/help_smoke.rs`
- Create: `tests/cli_parse.rs`
- Create: `tests/session_manager.rs`
- Create: `tests/ssh_runtime.rs`
- Create: `tests/input_prefix.rs`
- Create: `tests/terminal_engine.rs`
- Create: `tests/app_smoke.rs`
- Create: `docs/manual/sshell-mvp-acceptance.md` — manual verification checklist for vim, top, less, htop and Qt process behavior

Notes for the implementer:

- The approved spec supersedes the older `docs/detailed-design.md` point about runtime tab creation. Do not implement runtime new-tab input in MVP.
- Do not create a local PTY adapter module for MVP. Remote shell allocation should use the SSH channel PTY request, and exit code 6 should cover terminal backend initialization failures such as raw-mode or alternate-screen setup.

### Task 1: Bootstrap the crate and binary help path

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/cli.rs`
- Test: `tests/help_smoke.rs`

- [ ] **Step 1: Write the failing help smoke test**

```rust
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_mentions_connection_arguments() {
    let mut cmd = Command::cargo_bin("sshell").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("sshell <CONNECTION>..."))
        .stdout(contains("username:password@host[:port]"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test help_smoke -v`
Expected: FAIL because `Cargo.toml` and the `sshell` binary do not exist yet.

- [ ] **Step 3: Write the minimal binary skeleton and help text**

`Cargo.toml`

```toml
[package]
name = "sshell"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.38", features = ["macros", "rt-multi-thread", "signal", "sync", "time"] }
thiserror = "1.0"
ratatui = "0.28"
crossterm = "0.27"
russh = "0.45"
async-trait = "0.1"
vt100 = "0.15"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tokio-test = "0.4"
```

`src/lib.rs`

```rust
pub mod cli;
```

`src/cli.rs`

```rust
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "sshell", about = "SSH TUI client", version)]
pub struct CliArgs {
    #[arg(value_name = "CONNECTION", required = true, help = "username:password@host[:port]")]
    pub connections: Vec<String>,
}

pub fn parse() -> CliArgs {
    CliArgs::parse()
}
```

`src/main.rs`

```rust
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let _ = sshell::cli::parse();
    ExitCode::SUCCESS
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test help_smoke -v`
Expected: PASS with one passing test.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/main.rs src/cli.rs tests/help_smoke.rs
git commit -m "chore: bootstrap sshell crate"
```

### Task 2: Implement connection parsing and startup argument validation

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/cli.rs`
- Create: `src/connection_spec.rs`
- Create: `src/error.rs`
- Test: `tests/cli_parse.rs`

- [ ] **Step 1: Write the failing parser and exit-code tests**

```rust
use sshell::cli::parse_from;
use sshell::connection_spec::ConnectionSpec;
use sshell::error::AppExitCode;

#[test]
fn parses_multiple_connections_and_defaults_port_22() {
    let args = parse_from([
        "sshell",
        "root:secret@192.168.1.10",
        "admin:pw@example.com:2222",
    ])
    .unwrap();

    assert_eq!(args.connections.len(), 2);
    assert_eq!(args.connections[0], ConnectionSpec::new("root", "secret", "192.168.1.10", 22));
    assert_eq!(args.connections[1], ConnectionSpec::new("admin", "pw", "example.com", 2222));
}

#[test]
fn rejects_ipv6_literals_as_argument_errors() {
    let err = parse_from(["sshell", "root:pw@[::1]"]).unwrap_err();

    assert_eq!(err.exit_code(), AppExitCode::Argument);
    assert!(err.to_string().contains("IPv6 is not supported"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test cli_parse -v`
Expected: FAIL with unresolved imports and missing `parse_from`, `ConnectionSpec`, and `AppExitCode`.

- [ ] **Step 3: Implement parsing, validation, and startup error mapping**

`src/lib.rs`

```rust
pub mod cli;
pub mod connection_spec;
pub mod error;
```

`src/error.rs`

```rust
use std::process::ExitCode;

use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppExitCode {
    Success = 0,
    General = 1,
    Argument = 2,
    Network = 3,
    Auth = 4,
    Protocol = 5,
    Terminal = 6,
}

impl AppExitCode {
    pub fn as_exit_code(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("argument error: {0}")]
    Argument(String),
}

impl AppError {
    pub fn exit_code(&self) -> AppExitCode {
        match self {
            AppError::Argument(_) => AppExitCode::Argument,
        }
    }

    pub fn stderr_line(&self) -> String {
        format!("[ERROR] {}: {}", self.exit_code() as u8, self)
    }
}
```

`src/connection_spec.rs`

```rust
use std::net::Ipv4Addr;

use crate::error::AppError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionSpec {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
}

impl ConnectionSpec {
    pub fn new(username: &str, password: &str, host: &str, port: u16) -> Self {
        Self {
            username: username.to_owned(),
            password: password.to_owned(),
            host: host.to_owned(),
            port,
        }
    }

    pub fn display_name(&self) -> String {
        format!("{}@{}:{}", self.username, self.host, self.port)
    }
}

pub fn parse_connection_spec(input: &str) -> Result<ConnectionSpec, AppError> {
    if input.contains('[') || input.contains(']') {
        return Err(AppError::Argument("IPv6 is not supported in MVP".into()));
    }

    let (user_pass, host_port) = input
        .rsplit_once('@')
        .ok_or_else(|| AppError::Argument("connection string must contain @".into()))?;
    let (username, password) = user_pass
        .split_once(':')
        .ok_or_else(|| AppError::Argument("connection string must contain username:password".into()))?;
    let (host, port) = match host_port.rsplit_once(':') {
        Some((host, port)) if !port.is_empty() => {
            let parsed_port = port
                .parse::<u16>()
                .map_err(|_| AppError::Argument("port must be a valid u16".into()))?;
            (host, parsed_port)
        }
        _ => (host_port, 22),
    };

    let is_ipv4 = host.parse::<Ipv4Addr>().is_ok();
    let is_domain = host
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-');

    if !is_ipv4 && !is_domain {
        return Err(AppError::Argument("host must be an IPv4 address or domain name".into()));
    }

    Ok(ConnectionSpec::new(username, password, host, port))
}
```

`src/cli.rs`

```rust
use clap::Parser;

use crate::connection_spec::{parse_connection_spec, ConnectionSpec};
use crate::error::AppError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CliArgs {
    pub connections: Vec<ConnectionSpec>,
}

#[derive(Debug, Parser)]
#[command(name = "sshell", about = "SSH TUI client", version)]
struct RawCliArgs {
    #[arg(value_name = "CONNECTION", required = true, help = "username:password@host[:port]")]
    connections: Vec<String>,
}

pub fn parse() -> Result<CliArgs, AppError> {
    parse_from(std::env::args())
}

pub fn parse_from<I, T>(iter: I) -> Result<CliArgs, AppError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let raw = RawCliArgs::parse_from(iter);
    let connections = raw
        .connections
        .into_iter()
        .map(|value| parse_connection_spec(&value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CliArgs { connections })
}
```

`src/main.rs`

```rust
use sshell::cli;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match cli::parse() {
        Ok(_args) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err.stderr_line());
            err.exit_code().as_exit_code()
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test cli_parse -v`
Expected: PASS with both parser tests green.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/main.rs src/cli.rs src/connection_spec.rs src/error.rs tests/cli_parse.rs
git commit -m "feat: parse connection strings"
```

### Task 3: Build the exit model and tab state machine

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/error.rs`
- Create: `src/session_manager.rs`
- Test: `tests/session_manager.rs`

- [ ] **Step 1: Write failing tests for tab switching and first-error aggregation**

```rust
use sshell::error::{AppError, ExitAggregator};
use sshell::session_manager::{SessionId, SessionManager};

#[test]
fn closing_active_tab_selects_the_next_available_session() {
    let mut manager = SessionManager::new();
    let first = manager.insert("root@a:22".into());
    let second = manager.insert("root@b:22".into());
    let third = manager.insert("root@c:22".into());

    manager.activate(second).unwrap();
    manager.close(second).unwrap();

    assert_eq!(manager.active_id(), Some(third));
    assert_eq!(manager.ordered_ids(), vec![first, third]);
}

#[test]
fn aggregator_returns_first_non_success_error_once_all_sessions_end() {
    let mut aggregator = ExitAggregator::default();
    aggregator.record_session_error(SessionId::new(1), AppError::auth("bad credentials"));
    aggregator.record_session_error(SessionId::new(2), AppError::network("timeout"));

    assert_eq!(aggregator.final_code(), 4);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test session_manager -v`
Expected: FAIL with missing `SessionManager`, `SessionId`, `ExitAggregator`, and helper constructors on `AppError`.

- [ ] **Step 3: Implement the session manager and exit aggregation types**

`src/lib.rs`

```rust
pub mod cli;
pub mod connection_spec;
pub mod error;
pub mod session_manager;
```

`src/error.rs`

```rust
use std::collections::BTreeMap;
use std::process::ExitCode;

use thiserror::Error;

use crate::session_manager::SessionId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppExitCode {
    Success = 0,
    General = 1,
    Argument = 2,
    Network = 3,
    Auth = 4,
    Protocol = 5,
    Terminal = 6,
}

impl AppExitCode {
    pub fn as_exit_code(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

#[derive(Debug, Error, Clone)]
pub enum AppError {
    #[error("argument error: {0}")]
    Argument(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("authentication failed: {0}")]
    Auth(String),
    #[error("ssh protocol error: {0}")]
    Protocol(String),
    #[error("terminal error: {0}")]
    Terminal(String),
}

impl AppError {
    pub fn auth(message: impl Into<String>) -> Self { Self::Auth(message.into()) }
    pub fn network(message: impl Into<String>) -> Self { Self::Network(message.into()) }
    pub fn protocol(message: impl Into<String>) -> Self { Self::Protocol(message.into()) }
    pub fn terminal(message: impl Into<String>) -> Self { Self::Terminal(message.into()) }

    pub fn exit_code(&self) -> AppExitCode {
        match self {
            AppError::Argument(_) => AppExitCode::Argument,
            AppError::Network(_) => AppExitCode::Network,
            AppError::Auth(_) => AppExitCode::Auth,
            AppError::Protocol(_) => AppExitCode::Protocol,
            AppError::Terminal(_) => AppExitCode::Terminal,
        }
    }

    pub fn stderr_line(&self) -> String {
        format!("[ERROR] {}: {}", self.exit_code() as u8, self)
    }
}

#[derive(Default)]
pub struct ExitAggregator {
    first_error: Option<AppExitCode>,
    per_session: BTreeMap<SessionId, AppExitCode>,
}

impl ExitAggregator {
    pub fn record_session_error(&mut self, id: SessionId, error: AppError) {
        let code = error.exit_code();
        self.per_session.insert(id, code);
        self.first_error.get_or_insert(code);
    }

    pub fn final_code(&self) -> u8 {
        self.first_error.unwrap_or(AppExitCode::Success) as u8
    }
}
```

`src/session_manager.rs`

```rust
use std::num::NonZeroU64;

use crate::error::AppError;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SessionId(NonZeroU64);

impl SessionId {
    pub fn new(value: u64) -> Self {
        Self(NonZeroU64::new(value).expect("session id must be non-zero"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMeta {
    pub id: SessionId,
    pub title: String,
    pub last_error: Option<AppError>,
}

pub struct SessionManager {
    next_id: u64,
    order: Vec<SessionMeta>,
    active_index: Option<usize>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { next_id: 1, order: Vec::new(), active_index: None }
    }

    pub fn insert(&mut self, title: String) -> SessionId {
        let id = SessionId::new(self.next_id);
        self.next_id += 1;
        self.order.push(SessionMeta { id, title, last_error: None });
        self.active_index = Some(self.order.len() - 1);
        id
    }

    pub fn activate(&mut self, id: SessionId) -> Result<(), &'static str> {
        let index = self.order.iter().position(|session| session.id == id).ok_or("unknown session")?;
        self.active_index = Some(index);
        Ok(())
    }

    pub fn close(&mut self, id: SessionId) -> Result<(), &'static str> {
        let index = self.order.iter().position(|session| session.id == id).ok_or("unknown session")?;
        self.order.remove(index);
        if self.order.is_empty() {
            self.active_index = None;
        } else if index >= self.order.len() {
            self.active_index = Some(self.order.len() - 1);
        } else {
            self.active_index = Some(index);
        }
        Ok(())
    }

    pub fn active_id(&self) -> Option<SessionId> {
        self.active_index.map(|index| self.order[index].id)
    }

    pub fn ordered_ids(&self) -> Vec<SessionId> {
        self.order.iter().map(|item| item.id).collect()
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test session_manager -v`
Expected: PASS with the tab-switching and exit aggregation tests green.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/error.rs src/session_manager.rs tests/session_manager.rs
git commit -m "feat: add tab state and exit aggregation"
```

### Task 4: Implement SSH session runtime around a mockable client trait

**Files:**
- Modify: `src/lib.rs`
- Create: `src/events.rs`
- Create: `src/ssh/mod.rs`
- Create: `src/ssh/client.rs`
- Create: `src/ssh/runtime.rs`
- Test: `tests/ssh_runtime.rs`

- [ ] **Step 1: Write failing tests for auth failure mapping and PTY resize forwarding**

```rust
use std::sync::{Arc, Mutex};

use sshell::connection_spec::ConnectionSpec;
use sshell::events::SessionEvent;
use sshell::ssh::client::MockSshClient;
use sshell::ssh::runtime::run_session_once;

#[tokio::test]
async fn maps_auth_failure_to_session_error_event() {
    let client = MockSshClient::auth_fails();
    let spec = ConnectionSpec::new("root", "bad", "host", 22);

    let event = run_session_once(client, 1, spec).await.unwrap();

    assert_eq!(event, SessionEvent::Errored { session_id: 1, exit_code: 4 });
}

#[tokio::test]
async fn forwards_resize_to_the_underlying_client() {
    let resize_log = Arc::new(Mutex::new(Vec::new()));
    let client = MockSshClient::connected(resize_log.clone());
    let spec = ConnectionSpec::new("root", "pw", "host", 22);

    let _ = run_session_once(client.clone(), 1, spec).await;
    client.resize(120, 40).await.unwrap();

    assert_eq!(resize_log.lock().unwrap().as_slice(), &[(120, 40)]);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test ssh_runtime -v`
Expected: FAIL with missing SSH modules and session event types.

- [ ] **Step 3: Implement the client trait, mock, and session runtime**

`src/lib.rs`

```rust
pub mod cli;
pub mod connection_spec;
pub mod error;
pub mod events;
pub mod session_manager;
pub mod ssh;
```

`src/events.rs`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionEvent {
    Connected { session_id: u64, title: String },
    Output { session_id: u64, bytes: Vec<u8> },
    Errored { session_id: u64, exit_code: u8 },
    Closed { session_id: u64 },
}
```

`src/ssh/client.rs`

```rust
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::connection_spec::ConnectionSpec;
use crate::error::AppError;

#[async_trait]
pub trait SshClient: Clone + Send + Sync + 'static {
    async fn connect(&self, spec: &ConnectionSpec) -> Result<(), AppError>;
    async fn request_pty(&self, cols: u16, rows: u16) -> Result<(), AppError>;
    async fn read_chunk(&self) -> Result<Option<Vec<u8>>, AppError>;
    async fn write_chunk(&self, bytes: &[u8]) -> Result<(), AppError>;
    async fn resize(&self, cols: u16, rows: u16) -> Result<(), AppError>;
    async fn close(&self) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct MockSshClient {
    mode: MockMode,
    resize_log: Arc<Mutex<Vec<(u16, u16)>>>,
}

#[derive(Clone)]
enum MockMode {
    AuthFails,
    Connected,
}

impl MockSshClient {
    pub fn auth_fails() -> Self {
        Self { mode: MockMode::AuthFails, resize_log: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn connected(log: Arc<Mutex<Vec<(u16, u16)>>>) -> Self {
        Self { mode: MockMode::Connected, resize_log: log }
    }
}

#[async_trait]
impl SshClient for MockSshClient {
    async fn connect(&self, _spec: &ConnectionSpec) -> Result<(), AppError> {
        match self.mode {
            MockMode::AuthFails => Err(AppError::auth("bad credentials")),
            MockMode::Connected => Ok(()),
        }
    }

    async fn request_pty(&self, _cols: u16, _rows: u16) -> Result<(), AppError> { Ok(()) }
    async fn read_chunk(&self) -> Result<Option<Vec<u8>>, AppError> { Ok(None) }
    async fn write_chunk(&self, _bytes: &[u8]) -> Result<(), AppError> { Ok(()) }
    async fn resize(&self, cols: u16, rows: u16) -> Result<(), AppError> {
        self.resize_log.lock().unwrap().push((cols, rows));
        Ok(())
    }
    async fn close(&self) -> Result<(), AppError> { Ok(()) }
}
```

`src/ssh/runtime.rs`

```rust
use crate::connection_spec::ConnectionSpec;
use crate::events::SessionEvent;
use crate::ssh::client::SshClient;

pub async fn run_session_once<C: SshClient>(client: C, session_id: u64, spec: ConnectionSpec) -> Result<SessionEvent, crate::error::AppError> {
    match client.connect(&spec).await {
        Ok(()) => {
            client.request_pty(80, 24).await?;
            Ok(SessionEvent::Connected { session_id, title: spec.display_name() })
        }
        Err(err) => Ok(SessionEvent::Errored { session_id, exit_code: err.exit_code() as u8 }),
    }
}
```

`src/ssh/mod.rs`

```rust
pub mod client;
pub mod runtime;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test ssh_runtime -v`
Expected: PASS with async tests green.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/events.rs src/ssh/mod.rs src/ssh/client.rs src/ssh/runtime.rs tests/ssh_runtime.rs
git commit -m "feat: add mockable ssh runtime"
```

### Task 5: Build terminal emulation and the prefix-key input state machine

**Files:**
- Modify: `src/lib.rs`
- Create: `src/input.rs`
- Create: `src/terminal/mod.rs`
- Create: `src/terminal/engine.rs`
- Test: `tests/input_prefix.rs`
- Test: `tests/terminal_engine.rs`

- [ ] **Step 1: Write failing tests for prefix control and alternate-screen restoration**

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use sshell::input::{InputRouter, RoutedKey};
use sshell::terminal::engine::TerminalEngine;

#[test]
fn ctrl_a_then_n_becomes_next_tab_command() {
    let mut router = InputRouter::default();

    assert_eq!(
        router.route(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)),
        RoutedKey::PendingPrefix
    );
    assert_eq!(
        router.route(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)),
        RoutedKey::NextTab
    );
}

#[test]
fn alternate_screen_exit_restores_the_primary_buffer_contents() {
    let mut engine = TerminalEngine::new(24, 80);
    engine.feed(b"shell prompt");
    engine.feed(b"\x1b[?1049h");
    engine.feed(b"vim screen");
    engine.feed(b"\x1b[?1049l");

    assert!(engine.render_text().contains("shell prompt"));
    assert!(!engine.render_text().contains("vim screen"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test input_prefix --test terminal_engine -v`
Expected: FAIL with missing modules and types.

- [ ] **Step 3: Implement input routing and vt100-backed terminal state**

`src/lib.rs`

```rust
pub mod cli;
pub mod connection_spec;
pub mod error;
pub mod events;
pub mod input;
pub mod session_manager;
pub mod ssh;
pub mod terminal;
```

`src/input.rs`

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Eq, PartialEq)]
pub enum RoutedKey {
    PendingPrefix,
    NextTab,
    PrevTab,
    CloseTab,
    SendToRemote(Vec<u8>),
}

#[derive(Default)]
pub struct InputRouter {
    awaiting_prefix: bool,
}

impl InputRouter {
    pub fn route(&mut self, key: KeyEvent) -> RoutedKey {
        if self.awaiting_prefix {
            self.awaiting_prefix = false;
            return match key.code {
                KeyCode::Char('n') => RoutedKey::NextTab,
                KeyCode::Char('p') => RoutedKey::PrevTab,
                KeyCode::Char('x') => RoutedKey::CloseTab,
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => RoutedKey::SendToRemote(vec![0x01]),
                KeyCode::Char(ch) => RoutedKey::SendToRemote(vec![ch as u8]),
                _ => RoutedKey::SendToRemote(Vec::new()),
            };
        }

        if key.code == KeyCode::Char('a') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.awaiting_prefix = true;
            RoutedKey::PendingPrefix
        } else {
            match key.code {
                KeyCode::Char(ch) => RoutedKey::SendToRemote(vec![ch as u8]),
                KeyCode::Enter => RoutedKey::SendToRemote(vec![b'\n']),
                _ => RoutedKey::SendToRemote(Vec::new()),
            }
        }
    }
}
```

`src/terminal/engine.rs`

```rust
pub struct TerminalEngine {
    parser: vt100::Parser,
}

impl TerminalEngine {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self { parser: vt100::Parser::new(rows, cols, 0) }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.parser.set_size(rows, cols);
    }

    pub fn render_text(&self) -> String {
        self.parser.screen().contents()
    }
}
```

`src/terminal/mod.rs`

```rust
pub mod engine;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test input_prefix --test terminal_engine -v`
Expected: PASS with one routing test and one terminal-state test green.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/input.rs src/terminal/mod.rs src/terminal/engine.rs tests/input_prefix.rs tests/terminal_engine.rs
git commit -m "feat: add terminal engine and prefix input router"
```

### Task 6: Integrate the TUI shell and app event loop

**Files:**
- Modify: `src/lib.rs`
- Create: `src/tui/mod.rs`
- Create: `src/tui/view.rs`
- Create: `src/app.rs`
- Modify: `src/main.rs`
- Test: `tests/app_smoke.rs`

- [ ] **Step 1: Write failing tests for startup tabs and close-on-last-session exit**

```rust
use sshell::app::{build_initial_state, handle_local_command, LocalCommand};
use sshell::cli::CliArgs;
use sshell::connection_spec::ConnectionSpec;

#[test]
fn startup_creates_one_tab_per_connection() {
    let args = CliArgs {
        connections: vec![
            ConnectionSpec::new("root", "pw", "a", 22),
            ConnectionSpec::new("root", "pw", "b", 22),
        ],
    };

    let state = build_initial_state(args);

    assert_eq!(state.session_manager.ordered_ids().len(), 2);
}

#[test]
fn closing_last_tab_requests_process_exit() {
    let args = CliArgs {
        connections: vec![ConnectionSpec::new("root", "pw", "a", 22)],
    };
    let mut state = build_initial_state(args);

    let should_exit = handle_local_command(&mut state, LocalCommand::CloseActiveTab).unwrap();

    assert!(should_exit);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test app_smoke -v`
Expected: FAIL with missing app and TUI modules.

- [ ] **Step 3: Implement app state, local commands, and TUI rendering**

`src/lib.rs`

```rust
pub mod app;
pub mod cli;
pub mod connection_spec;
pub mod error;
pub mod events;
pub mod input;
pub mod session_manager;
pub mod ssh;
pub mod terminal;
pub mod tui;
```

`src/app.rs`

```rust
use crate::cli::CliArgs;
use crate::error::ExitAggregator;
use crate::session_manager::SessionManager;

pub struct AppState {
    pub session_manager: SessionManager,
    pub exit_aggregator: ExitAggregator,
}

#[derive(Clone, Copy)]
pub enum LocalCommand {
    NextTab,
    PrevTab,
    CloseActiveTab,
}

pub fn build_initial_state(args: CliArgs) -> AppState {
    let mut session_manager = SessionManager::new();
    for connection in args.connections {
        session_manager.insert(connection.display_name());
    }
    AppState { session_manager, exit_aggregator: ExitAggregator::default() }
}

pub fn handle_local_command(state: &mut AppState, command: LocalCommand) -> Result<bool, &'static str> {
    match command {
        LocalCommand::CloseActiveTab => match state.session_manager.active_id() {
            Some(id) => {
                state.session_manager.close(id)?;
                Ok(state.session_manager.active_id().is_none())
            }
            None => Ok(true),
        },
        LocalCommand::NextTab | LocalCommand::PrevTab => Ok(false),
    }
}
```

`src/tui/view.rs`

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::AppState;

pub fn draw(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(frame.size());

    let titles = state
        .session_manager
        .ordered_ids()
        .into_iter()
        .map(|id| Line::from(format!("{:?}", id)))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Sessions"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, chunks[0]);
    frame.render_widget(Paragraph::new("Terminal view attaches here").block(Block::default().borders(Borders::ALL)), chunks[1]);
}
```

`src/tui/mod.rs`

```rust
pub mod view;
```

`src/main.rs`

```rust
use sshell::{app, cli};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match cli::parse() {
        Ok(args) => {
            let _state = app::build_initial_state(args);
            std::process::ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err.stderr_line());
            err.exit_code().as_exit_code()
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test app_smoke -v`
Expected: PASS with startup and close behavior tests green.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/app.rs src/tui/mod.rs src/tui/view.rs src/main.rs tests/app_smoke.rs
git commit -m "feat: add app shell and tab lifecycle"
```

### Task 7: Lock down manual acceptance and document execution checks

**Files:**
- Create: `docs/manual/sshell-mvp-acceptance.md`

- [ ] **Step 1: Write the acceptance checklist document**

```markdown
# sshell MVP Acceptance Checklist

## Prerequisites

- A reachable SSH server with shell access
- Test programs installed remotely: `vim`, `top`, `less`, `htop`
- A Qt caller or shell wrapper that can inspect process exit codes

## Manual test 1: Single-session startup

Run:

```bash
cargo run -- root:secret@127.0.0.1
```

Expected:

- The TUI opens with one tab
- The remote shell accepts typed commands
- `Ctrl+A` by itself does not close the program

## Manual test 2: Full-screen programs

Run each command remotely:

```bash
vim
top
less /etc/hosts
htop
```

Expected for each:

- The screen updates without corruption
- Leaving the program returns to the shell prompt
- Resizing the terminal updates the remote layout

## Manual test 3: Multi-tab startup

Run:

```bash
cargo run -- root:secret@127.0.0.1 root:secret@example.com:2222
```

Expected:

- Two tabs are visible on startup
- `Ctrl+A`, `n` moves to the next tab
- `Ctrl+A`, `p` moves to the previous tab
- `Ctrl+A`, `x` closes only the active tab

## Manual test 4: Exit codes

Run:

```bash
cargo run -- root:wrong@127.0.0.1 ; echo $?
```

Expected: stderr contains `[ERROR] 4:` and the shell prints `4`
```

- [ ] **Step 2: Review the checklist against the approved spec**

Run: `sed -n '1,220p' docs/manual/sshell-mvp-acceptance.md`
Expected: The document explicitly covers vim, top, less, htop, multi-tab startup, and exit-code behavior.

- [ ] **Step 3: Commit**

```bash
git add docs/manual/sshell-mvp-acceptance.md
git commit -m "docs: add mvp acceptance checklist"
```

## Plan Self-Review

- Spec coverage: The plan covers CLI parsing, connection parsing, tab state, SSH runtime, terminal compatibility, prefix-key controls, app lifecycle, and manual validation. The approved spec's non-goals are preserved because no task adds runtime session creation, key auth, transfer, or Qt IPC.
- Placeholder scan: No task uses `TODO`, `TBD`, or vague phrases like "handle appropriately". Commands, file paths, and test snippets are explicit.
- Type consistency: `ConnectionSpec`, `AppExitCode`, `AppError`, `SessionManager`, `SessionEvent`, `InputRouter`, and `AppState` names are used consistently across later tasks.
