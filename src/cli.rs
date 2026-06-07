use std::ffi::OsString;

use clap::{Command, CommandFactory, Parser};

use crate::connection_spec::{parse_connection_spec, ConnectionSpec};
use crate::error::AppError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CliArgs {
    pub connections: Vec<ConnectionSpec>,
}

#[derive(Debug, Parser)]
#[command(
    name = "sshell",
    about = "SSH TUI client",
    version,
    override_usage = "sshell <CONNECTION>...",
    help_template = "{name} {version}\n{about}\n\nUsage: {usage}\n\n{all-args}"
)]
struct RawCliArgs {
    #[arg(value_name = "CONNECTION", required = true, help = "username:password@host[:port]")]
    connections: Vec<String>,
}

pub fn command() -> Command {
    RawCliArgs::command()
}

pub fn parse() -> Result<CliArgs, AppError> {
    let raw = RawCliArgs::parse();
    build_args(raw)
}

pub fn parse_from<I, T>(iter: I) -> Result<CliArgs, AppError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let raw = RawCliArgs::try_parse_from(iter)
        .map_err(|err| AppError::argument(err.to_string()))?;
    build_args(raw)
}

fn build_args(raw: RawCliArgs) -> Result<CliArgs, AppError> {
    let connections = raw
        .connections
        .into_iter()
        .map(|value| parse_connection_spec(&value))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CliArgs { connections })
}
