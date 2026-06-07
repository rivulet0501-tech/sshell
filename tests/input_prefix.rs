use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use sshell::input::{InputRouter, RoutedKey};

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