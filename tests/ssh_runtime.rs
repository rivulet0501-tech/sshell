use std::sync::{Arc, Mutex};

use sshell::connection_spec::ConnectionSpec;
use sshell::events::SessionEvent;
use sshell::ssh::client::{MockSshClient, SshClient};
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