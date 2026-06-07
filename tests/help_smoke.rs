use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_mentions_connection_arguments() {
    let mut cmd = Command::cargo_bin("sshell").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("sshell <CONNECTION>..."))
        .stdout(contains("username:password@host[:port]"));
}