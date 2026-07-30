use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

const MAX_IDEMPOTENCY_KEY_LEN: usize = 128;

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum ClientCommand {
    /// Create a new client and print its bearer token.
    Create {
        /// Unique client name / group identifier.
        name: String,
        /// Stable request identifier to reuse when retrying the same mutation.
        #[arg(long)]
        request_id: Option<String>,
    },
    /// Revoke all tokens for a client.
    Revoke {
        /// Client name.
        name: String,
        /// Stable request identifier to reuse when retrying the same mutation.
        #[arg(long)]
        request_id: Option<String>,
    },
    /// Rotate the token for a client and print the new one.
    Rotate {
        /// Client name.
        name: String,
        /// Stable request identifier to reuse when retrying the same mutation.
        #[arg(long)]
        request_id: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum TokenCommand {
    /// List all clients and their token status.
    List,
}

#[derive(Debug, Parser)]
#[command(name = "duotunnel-ctld", about = "DuoTunnel control daemon")]
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
    Client(ClientCommand),
    #[command(subcommand)]
    Token(TokenCommand),
}

pub(crate) async fn run_client_cli(socket: &str, cmd: ClientCommand) -> Result<()> {
    let (path, query, request_id) = match cmd {
        ClientCommand::Create { name, request_id } => ("/v1/clients/create", name, request_id),
        ClientCommand::Revoke { name, request_id } => ("/v1/clients/revoke", name, request_id),
        ClientCommand::Rotate { name, request_id } => ("/v1/clients/rotate", name, request_id),
    };
    run_admin_cli(
        socket,
        "POST",
        path,
        query,
        Some(request_id.unwrap_or_else(new_request_id)),
    )
    .await
}

pub(crate) async fn run_token_cli(socket: &str, cmd: TokenCommand) -> Result<()> {
    match cmd {
        TokenCommand::List => run_admin_cli(socket, "GET", "/v1/tokens", String::new(), None).await,
    }
}

async fn run_admin_cli(
    socket: &str,
    method: &str,
    path: &str,
    query: String,
    request_id: Option<String>,
) -> Result<()> {
    let target = if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?name={}", percent_encode(&query))
    };
    let mut stream = UnixStream::connect(socket).await?;
    let idempotency_header = request_id
        .map(|request_id| format!("Idempotency-Key: {request_id}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "{method} {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n{idempotency_header}Content-Length: 0\r\n\r\n"
    );
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

fn new_request_id() -> String {
    hex::encode(rand::random::<[u8; 16]>())
}

fn parse_idempotency_key(request: &str) -> Result<Option<String>> {
    let Some(value) = request_header(request, "Idempotency-Key") else {
        return Ok(None);
    };
    if value.is_empty()
        || value.len() > MAX_IDEMPOTENCY_KEY_LEN
        || !value.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
    {
        anyhow::bail!("invalid Idempotency-Key")
    }
    Ok(Some(value.to_string()))
}

fn request_header<'a>(request: &'a str, name: &str) -> Option<&'a str> {
    let head = request
        .split_once("\r\n\r\n")
        .map_or(request, |(head, _)| head);
    head.lines().skip(1).find_map(|line| {
        let (header_name, value) = line.split_once(':')?;
        header_name.eq_ignore_ascii_case(name).then(|| value.trim())
    })
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
    let mutation = match (method, path, name.as_deref()) {
        ("POST", "/v1/config/override", None) => match serde_json::from_str(body) {
            Ok(operation) => Ok(Some(
                crate::control::service::AdminMutation::ApplyConfigOverride(operation),
            )),
            Err(error) => Err(anyhow::anyhow!("invalid ConfigOperation: {error}")),
        },
        ("POST", "/v1/config/clear", Some(key)) => match resource {
            Some(resource) => Ok(Some(
                crate::control::service::AdminMutation::ClearConfigOverride {
                    resource,
                    key: key.to_string(),
                },
            )),
            None => Err(anyhow::anyhow!("resource is required")),
        },
        ("POST", "/v1/clients/create", Some(name)) => Ok(Some(
            crate::control::service::AdminMutation::CreateClient(name.to_string()),
        )),
        ("POST", "/v1/clients/rotate", Some(name)) => Ok(Some(
            crate::control::service::AdminMutation::RotateToken(name.to_string()),
        )),
        ("POST", "/v1/clients/revoke", Some(name)) => Ok(Some(
            crate::control::service::AdminMutation::RevokeToken(name.to_string()),
        )),
        _ => Ok(None),
    };
    let mutation = match mutation {
        Ok(mutation) => mutation,
        Err(error) => return (400, error.to_string()),
    };
    if let Some(mutation) = mutation {
        let request_id = match parse_idempotency_key(request) {
            Ok(Some(request_id)) => request_id,
            Ok(None) => {
                return (
                    428,
                    "Idempotency-Key is required for admin mutations".into(),
                )
            }
            Err(error) => return (400, error.to_string()),
        };
        let fingerprint = match mutation.canonical_fingerprint() {
            Ok(fingerprint) => fingerprint,
            Err(error) => return (400, error.to_string()),
        };
        return match svc
            .execute_admin_mutation(&request_id, &fingerprint, mutation)
            .await
        {
            Ok(response) => (response.status, response.body),
            Err(error) => error
                .downcast_ref::<crate::control::service::AdminMutationError>()
                .map_or_else(
                    || (503, "admin mutation unavailable".into()),
                    |error| {
                        let status = match error.kind() {
                            crate::control::service::AdminErrorKind::InvalidRequest => 400,
                            crate::control::service::AdminErrorKind::NotFound => 404,
                            crate::control::service::AdminErrorKind::Conflict => 409,
                        };
                        (status, error.message().to_owned())
                    },
                ),
        };
    }

    let result = match (method, path, name.as_deref()) {
        ("GET", "/v1/tokens", None) => svc.list_tokens().await.map(|tokens| {
            let mut body = String::new();
            for token in tokens {
                use std::fmt::Write;
                let _ = writeln!(
                    body,
                    "{:<20} {:<10} {:<8} {:<10} {} {}",
                    token.client_name,
                    token.client_status,
                    token.token_id,
                    token
                        .token_status
                        .map(|status| status.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    token.created_at,
                    token.revoked_at.as_deref().unwrap_or("-"),
                );
            }
            body
        }),
        _ => Err(anyhow::anyhow!("unknown admin endpoint")),
    };
    match result {
        Ok(body) => (200, body),
        Err(error) if error.to_string() == "unknown admin endpoint" => (404, error.to_string()),
        Err(error) => (503, error.to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_idempotency_key_without_affecting_other_headers() {
        let request =
            "GET /v1/tokens HTTP/1.1\r\nHost: localhost\r\nIdempotency-Key: request-1\r\n\r\n";
        assert_eq!(
            parse_idempotency_key(request).unwrap().as_deref(),
            Some("request-1")
        );

        let request = "GET /v1/tokens HTTP/1.1\r\nhOsT: localhost\r\n\r\n";
        assert_eq!(parse_idempotency_key(request).unwrap(), None);
    }

    #[test]
    fn rejects_invalid_idempotency_key() {
        let request =
            "POST /v1/clients/rotate?name=alpha HTTP/1.1\r\nIdempotency-Key: bad key\r\n\r\n";
        assert!(parse_idempotency_key(request).is_err());
    }

    #[test]
    fn canonical_fingerprint_uses_typed_mutation() {
        let first = crate::control::service::AdminMutation::CreateClient("alpha".into());
        let same = crate::control::service::AdminMutation::CreateClient("alpha".into());
        let different = crate::control::service::AdminMutation::CreateClient("beta".into());
        assert_eq!(
            first.canonical_fingerprint().unwrap(),
            same.canonical_fingerprint().unwrap()
        );
        assert_ne!(
            first.canonical_fingerprint().unwrap(),
            different.canonical_fingerprint().unwrap()
        );
    }
}
