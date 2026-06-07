use sshell::terminal::engine::TerminalEngine;

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