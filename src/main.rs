use std::process::ExitCode;

fn main() -> ExitCode {
    match sshell::cli::parse() {
        Ok(args) => {
            let _state = sshell::app::build_initial_state(args);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err.stderr_line());
            err.exit_code().as_exit_code()
        }
    }
}
