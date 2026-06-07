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

#[cfg(test)]
mod tests {
    use crate::cli::{command, parse_from};
    use crate::connection_spec::ConnectionSpec;

    #[test]
    fn test_cli_parse_basic() {
        let result = parse_from([
            "sshell",
            "user:pass@localhost",
        ]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(
            args.connections,
            vec![ConnectionSpec::new("user", "pass", "localhost", 22)]
        );
    }

    #[test]
    fn test_cli_parse_multiple_connections() {
        let result = parse_from([
            "sshell",
            "user1:pass1@host1",
            "user2:pass2@host2:2222",
        ]);
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(
            args.connections,
            vec![
                ConnectionSpec::new("user1", "pass1", "host1", 22),
                ConnectionSpec::new("user2", "pass2", "host2", 2222),
            ]
        );
    }

    #[test]
    fn test_help_text_format() {
        let mut cmd = command();
        let help_text = cmd.render_help().to_string();
        assert!(help_text.contains("sshell"));
        assert!(help_text.contains("CONNECTION") || help_text.contains("connection"));
    }
}


