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