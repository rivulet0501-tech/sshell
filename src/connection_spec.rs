use std::net::Ipv4Addr;

use crate::error::AppError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionSpec {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
}

impl ConnectionSpec {
    pub fn new(username: &str, password: &str, host: &str, port: u16) -> Self {
        Self {
            username: username.to_owned(),
            password: password.to_owned(),
            host: host.to_owned(),
            port,
        }
    }

    pub fn display_name(&self) -> String {
        format!("{}@{}:{}", self.username, self.host, self.port)
    }
}

pub fn parse_connection_spec(input: &str) -> Result<ConnectionSpec, AppError> {
    if input.contains('[') || input.contains(']') {
        return Err(AppError::argument("IPv6 is not supported in MVP"));
    }

    let (user_pass, host_port) = input
        .rsplit_once('@')
        .ok_or_else(|| AppError::argument("connection string must contain @"))?;
    let (username, password) = user_pass
        .split_once(':')
        .ok_or_else(|| AppError::argument("connection string must contain username:password"))?;

    if username.is_empty() || password.is_empty() {
        return Err(AppError::argument("username and password must not be empty"));
    }

    let (host, port) = match host_port.rsplit_once(':') {
        Some((host, port_text)) if !port_text.is_empty() => {
            let port = port_text
                .parse::<u16>()
                .map_err(|_| AppError::argument("port must be a valid u16"))?;
            (host, port)
        }
        _ => (host_port, 22),
    };

    if host.is_empty() {
        return Err(AppError::argument("host must not be empty"));
    }

    let is_ipv4 = host.parse::<Ipv4Addr>().is_ok();
    let is_domain = host
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-');

    if !is_ipv4 && !is_domain {
        return Err(AppError::argument(
            "host must be an IPv4 address or domain name",
        ));
    }

    Ok(ConnectionSpec::new(username, password, host, port))
}