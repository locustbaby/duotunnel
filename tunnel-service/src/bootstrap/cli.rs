use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum CliCommand {
    /// Create a new client and print its bearer token.
    CreateClient {
        /// Unique client name / group identifier.
        name: String,
    },
    /// Revoke all tokens for a client.
    RevokeClient {
        /// Client name.
        name: String,
    },
    /// Rotate the token for a client and print the new one.
    RotateToken {
        /// Client name.
        name: String,
    },
    /// List all clients and their token status.
    ListTokens,
}

#[derive(Debug, Parser)]
#[command(name = "tunnel-ctld", about = "Tunnel control daemon")]
pub(crate) struct Args {
    #[arg(short, long, default_value = "ctld.yaml")]
    pub(crate) config: String,

    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum Command {
    Serve,
    #[command(subcommand)]
    Client(CliCommand),
}

pub(crate) async fn run_cli(socket: &str, cmd: CliCommand) -> Result<()> {
    let (path, query) = match cmd {
        CliCommand::CreateClient { name } => ("/v1/clients/create", name),
        CliCommand::RevokeClient { name } => ("/v1/clients/revoke", name),
        CliCommand::RotateToken { name } => ("/v1/clients/rotate", name),
        CliCommand::ListTokens => ("/v1/tokens", String::new()),
    };
    let target = if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?name={}", percent_encode(&query))
    };
    let mut stream = UnixStream::connect(socket).await?;
    let request = format!("GET {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    let response = String::from_utf8(response)?;
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| anyhow::anyhow!("invalid admin response"))?;
    if !head.starts_with("HTTP/1.1 200") {
        return Err(anyhow::anyhow!(body.trim().to_string()));
    }
    match path {
        "/v1/clients/create" => println!("Created client '{query}'\nToken: {}", body.trim()),
        "/v1/clients/rotate" => println!("New token for '{query}': {}", body.trim()),
        "/v1/clients/revoke" => println!("Revoked token for '{query}'"),
        "/v1/tokens" => print!("{body}"),
        _ => unreachable!(),
    }
    Ok(())
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => vec![byte as char],
            byte => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

pub(crate) async fn handle_admin_request(
    request: &str,
    svc: &crate::control::service::ControlService,
) -> (u16, String) {
    let method = request.split_whitespace().next().unwrap_or("");
    let target = request.split_whitespace().nth(1).unwrap_or("");
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let query_param = |key: &str| {
        query
            .split('&')
            .find_map(|part| {
                part.strip_prefix(key)
                    .and_then(|value| value.strip_prefix('='))
            })
            .map(percent_decode)
    };
    let name = query_param("name");
    let resource = query_param("resource");
    let body = request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("");
    let result = match (method, path, name.as_deref()) {
        ("POST", "/v1/config/override", None) => match serde_json::from_str(body) {
            Ok(operation) => svc
                .apply_config_override(&operation)
                .await
                .map(|_| "ok".into()),
            Err(error) => Err(anyhow::anyhow!("invalid ConfigOperation: {error}")),
        },
        ("POST", "/v1/config/clear", Some(key)) => match resource {
            Some(resource) => svc
                .clear_config_override(&resource, key)
                .await
                .map(|_| "ok".into()),
            None => Err(anyhow::anyhow!("resource is required")),
        },
        ("GET", "/v1/clients/create", Some(name)) => svc.create_client(name).await,
        ("GET", "/v1/clients/rotate", Some(name)) => svc.rotate_token(name).await,
        ("GET", "/v1/clients/revoke", Some(name)) => {
            svc.revoke_token(name).await.map(|_| "ok".into())
        }
        ("GET", "/v1/tokens", None) => svc.list_tokens().await.map(|tokens| {
            let mut body = String::new();
            for token in tokens {
                use std::fmt::Write;
                let _ = writeln!(
                    body,
                    "{:<20} {:<10} {:<8} {:<10} {}",
                    token.client_name,
                    token.client_status,
                    token.token_id,
                    token
                        .token_status
                        .map(|status| status.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    token.created_at,
                );
            }
            body
        }),
        _ => Err(anyhow::anyhow!("unknown admin endpoint")),
    };
    match result {
        Ok(body) => (200, body),
        Err(error) => (400, error.to_string()),
    }
}

fn percent_decode(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                result.push(byte as char);
                index += 3;
                continue;
            }
        }
        result.push(bytes[index] as char);
        index += 1;
    }
    result
}
