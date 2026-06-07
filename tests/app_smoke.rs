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