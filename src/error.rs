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

#[derive(Clone, Debug, Eq, Error, PartialEq)]
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
    pub fn argument(message: impl Into<String>) -> Self {
        Self::Argument(message.into())
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol(message.into())
    }

    pub fn terminal(message: impl Into<String>) -> Self {
        Self::Terminal(message.into())
    }

    pub fn exit_code(&self) -> AppExitCode {
        match self {
            Self::Argument(_) => AppExitCode::Argument,
            Self::Network(_) => AppExitCode::Network,
            Self::Auth(_) => AppExitCode::Auth,
            Self::Protocol(_) => AppExitCode::Protocol,
            Self::Terminal(_) => AppExitCode::Terminal,
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