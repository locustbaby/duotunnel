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
    /// Seed routing data from a server.yaml config file.
    ///
    /// Reads tunnel_management and server_egress_upstream sections and saves
    /// them to the ctld database.  Overwrites any existing routing data.
    ImportRouting {
        /// Path to the server YAML config file.
        #[arg(long, short)]
        from: String,
    },
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
                println!("{:<20} {:<10} {:<8} {:<10} {}", "Client", "Client Status", "Token ID", "Token Status", "Created At");
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
        CliCommand::ImportRouting { from } => {
            let data = import::load_routing_from_server_yaml(&from)?;
            let n_listeners = data.ingress_listeners.len();
            let n_groups = data.client_groups.len();
            let n_upstreams = data.egress_upstreams.len();
            svc.save_routing(&data).await?;
            println!(
                "Imported routing from '{from}': {n_listeners} listeners, \
                 {n_groups} client groups, {n_upstreams} egress upstreams"
            );
        }
    }
    Ok(())
}

// ── Import helpers ────────────────────────────────────────────────────────────

mod import {
    use anyhow::{Context, Result};
    use figment::{providers::{Format, Yaml}, Figment};
    use serde::Deserialize;
    use std::collections::HashMap;
    use tunnel_store::rules::{
        ClientGroup, ClientUpstream, EgressUpstreamDef, EgressVhostRule, IngressListener,
        IngressListenerMode, IngressVhostRule, RoutingData, UpstreamServer,
    };

    // ── Minimal server.yaml schema (only the sections we need) ────────────────

    #[derive(Debug, Deserialize)]
    pub struct ServerYaml {
        #[serde(default)]
        pub server_egress_upstream: EgressSection,
        #[serde(default)]
        pub tunnel_management: ManagementSection,
    }

    #[derive(Debug, Deserialize, Default)]
    pub struct EgressSection {
        #[serde(default)]
        pub upstreams: HashMap<String, UpstreamDef>,
        #[serde(default)]
        pub rules: EgressRules,
    }

    #[derive(Debug, Deserialize, Default)]
    pub struct EgressRules {
        #[serde(default)]
        pub vhost: Vec<EgressVhostRuleYaml>,
    }

    #[derive(Debug, Deserialize)]
    pub struct EgressVhostRuleYaml {
        pub match_host: String,
        pub action_upstream: String,
    }

    #[derive(Debug, Deserialize, Default)]
    pub struct ManagementSection {
        #[serde(default)]
        pub server_ingress_routing: IngressRoutingSection,
        #[serde(default)]
        pub client_configs: ClientConfigsSection,
    }

    #[derive(Debug, Deserialize, Default)]
    pub struct IngressRoutingSection {
        #[serde(default)]
        pub listeners: Vec<IngressListenerYaml>,
    }

    #[derive(Debug, Deserialize)]
    pub struct IngressListenerYaml {
        pub port: u16,
        #[serde(flatten)]
        pub mode: IngressModeYaml,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "mode", rename_all = "lowercase")]
    pub enum IngressModeYaml {
        Http(HttpListenerYaml),
        Tcp(TcpListenerYaml),
    }

    #[derive(Debug, Deserialize, Default)]
    pub struct HttpListenerYaml {
        #[serde(default)]
        pub vhost: Vec<VhostRuleYaml>,
    }

    #[derive(Debug, Deserialize)]
    pub struct TcpListenerYaml {
        pub client_group: String,
        pub proxy_name: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct VhostRuleYaml {
        pub match_host: String,
        pub client_group: String,
        pub proxy_name: String,
    }

    #[derive(Debug, Deserialize, Default)]
    pub struct ClientConfigsSection {
        #[serde(default)]
        pub groups: HashMap<String, GroupConfigYaml>,
    }

    #[derive(Debug, Deserialize)]
    pub struct GroupConfigYaml {
        #[serde(default)]
        pub config_version: String,
        #[serde(default)]
        pub upstreams: HashMap<String, UpstreamDef>,
    }

    #[derive(Debug, Deserialize)]
    pub struct UpstreamDef {
        pub servers: Vec<ServerDef>,
        #[serde(default = "default_lb")]
        pub lb_policy: String,
    }

    fn default_lb() -> String {
        "round_robin".to_string()
    }

    #[derive(Debug, Deserialize)]
    pub struct ServerDef {
        pub address: String,
        #[serde(default)]
        pub resolve: bool,
    }

    // ── Conversion ────────────────────────────────────────────────────────────

    pub fn load_routing_from_server_yaml(path: &str) -> Result<RoutingData> {
        let yaml: ServerYaml = Figment::new()
            .merge(Yaml::file(path))
            .extract()
            .with_context(|| format!("failed to parse server config '{path}'"))?;

        // Ingress listeners — assign sequential IDs (DB will reassign on save,
        // but RoutingData.id is required for the type).
        let ingress_listeners: Vec<IngressListener> = yaml
            .tunnel_management
            .server_ingress_routing
            .listeners
            .into_iter()
            .enumerate()
            .map(|(i, l)| {
                let mode = match l.mode {
                    IngressModeYaml::Http(h) => IngressListenerMode::Http {
                        vhost: h
                            .vhost
                            .into_iter()
                            .map(|v| IngressVhostRule {
                                match_host: v.match_host,
                                group_id: v.client_group,
                                proxy_name: v.proxy_name,
                            })
                            .collect(),
                    },
                    IngressModeYaml::Tcp(t) => IngressListenerMode::Tcp {
                        group_id: t.client_group,
                        proxy_name: t.proxy_name,
                    },
                };
                IngressListener {
                    id: (i + 1) as i64,
                    port: l.port,
                    mode,
                }
            })
            .collect();

        // Client groups
        let client_groups: Vec<ClientGroup> = yaml
            .tunnel_management
            .client_configs
            .groups
            .into_iter()
            .map(|(group_id, g)| {
                let upstreams = g
                    .upstreams
                    .into_iter()
                    .map(|(name, u)| ClientUpstream {
                        name,
                        lb_policy: u.lb_policy,
                        servers: u
                            .servers
                            .into_iter()
                            .map(|s| UpstreamServer {
                                address: s.address,
                                resolve: s.resolve,
                            })
                            .collect(),
                    })
                    .collect();
                ClientGroup {
                    group_id,
                    config_version: g.config_version,
                    upstreams,
                }
            })
            .collect();

        // Egress upstreams
        let egress_upstreams: Vec<EgressUpstreamDef> = yaml
            .server_egress_upstream
            .upstreams
            .into_iter()
            .map(|(name, u)| EgressUpstreamDef {
                name,
                lb_policy: u.lb_policy,
                servers: u
                    .servers
                    .into_iter()
                    .map(|s| UpstreamServer {
                        address: s.address,
                        resolve: s.resolve,
                    })
                    .collect(),
            })
            .collect();

        // Egress vhost rules
        let egress_vhost_rules: Vec<EgressVhostRule> = yaml
            .server_egress_upstream
            .rules
            .vhost
            .into_iter()
            .map(|r| EgressVhostRule {
                match_host: r.match_host,
                action_upstream: r.action_upstream,
            })
            .collect();

        Ok(RoutingData {
            ingress_listeners,
            client_groups,
            egress_upstreams,
            egress_vhost_rules,
        })
    }
}
