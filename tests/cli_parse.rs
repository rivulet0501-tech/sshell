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
    assert_eq!(
        args.connections[0],
        ConnectionSpec::new("root", "secret", "192.168.1.10", 22)
    );
    assert_eq!(
        args.connections[1],
        ConnectionSpec::new("admin", "pw", "example.com", 2222)
    );
}

#[test]
fn rejects_ipv6_literals_as_argument_errors() {
    let err = parse_from(["sshell", "root:pw@[::1]"]).unwrap_err();

    assert_eq!(err.exit_code(), AppExitCode::Argument);
    assert!(err.to_string().contains("IPv6 is not supported"));
}