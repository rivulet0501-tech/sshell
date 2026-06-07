use crate::connection_spec::ConnectionSpec;
use crate::error::AppError;
use crate::events::SessionEvent;
use crate::ssh::client::SshClient;

pub async fn run_session_once<C: SshClient>(
    client: C,
    session_id: u64,
    spec: ConnectionSpec,
) -> Result<SessionEvent, AppError> {
    match client.connect(&spec).await {
        Ok(()) => {
            client.request_pty(80, 24).await?;
            Ok(SessionEvent::Connected {
                session_id,
                title: format!("{}@{}:{}", spec.username, spec.host, spec.port),
            })
        }
        Err(err) => Ok(SessionEvent::Errored {
            session_id,
            exit_code: err.exit_code() as u8,
        }),
    }
}