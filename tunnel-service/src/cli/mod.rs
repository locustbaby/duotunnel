/// Admin CLI subcommands for tunnel-ctld.
///
/// These are issued as one-shot operations against the running ctld over
/// a local admin socket (or directly via the same ControlService when the
/// binary is run in "cli" mode).  For now the CLI handler reuses the same
/// SQLite pool as the server process so commands can be run from the same
/// binary without a separate RPC layer.
use anyhow::Result;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum CliCommand {
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

pub async fn run_cli(cmd: CliCommand, svc: &crate::service::ControlService) -> Result<()> {
    match cmd {
        CliCommand::CreateClient { name } => {
            let token = svc.create_client(&name).await?;
            println!("Created client '{name}'\nToken: {token}");
        }
        CliCommand::RevokeClient { name } => {
            svc.revoke_token(&name).await?;
            println!("Revoked token for '{name}'");
        }
        CliCommand::RotateToken { name } => {
            let token = svc.rotate_token(&name).await?;
            println!("New token for '{name}': {token}");
        }
        CliCommand::ListTokens => {
            let tokens = svc.list_tokens().await?;
            if tokens.is_empty() {
                println!("No clients registered.");
            } else {
                println!("{:<20} {:<10} {:<8} {:<10} Created At", "Client", "Client Status", "Token ID", "Token Status");
                println!("{}", "-".repeat(75));
                for t in &tokens {
                    println!(
                        "{:<20} {:<14} {:<8} {:<12} {}",
                        t.client_name,
                        t.client_status,
                        t.token_id,
                        t.token_status,
                        t.created_at,
                    );
                }
            }
        }
    }
    Ok(())
}
