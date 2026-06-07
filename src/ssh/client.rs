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
        Self {
            mode: MockMode::AuthFails,
            resize_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn connected(log: Arc<Mutex<Vec<(u16, u16)>>>) -> Self {
        Self {
            mode: MockMode::Connected,
            resize_log: log,
        }
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

    async fn request_pty(&self, _cols: u16, _rows: u16) -> Result<(), AppError> {
        Ok(())
    }

    async fn read_chunk(&self) -> Result<Option<Vec<u8>>, AppError> {
        Ok(None)
    }

    async fn write_chunk(&self, _bytes: &[u8]) -> Result<(), AppError> {
        Ok(())
    }

    async fn resize(&self, cols: u16, rows: u16) -> Result<(), AppError> {
        self.resize_log.lock().unwrap().push((cols, rows));
        Ok(())
    }

    async fn close(&self) -> Result<(), AppError> {
        Ok(())
    }
}