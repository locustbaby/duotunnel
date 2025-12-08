use serde::Deserialize;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub server: ServerBasicConfig,
    #[serde(default)]
    pub server_egress_upstream: Option<ServerEgressUpstream>,
    #[serde(default)]
    pub tunnel_management: Option<TunnelManagement>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerBasicConfig {
    pub config_version: String,
    pub tunnel_port: u16,
    #[serde(default)]
    pub http_entry_port: Option<u16>,
    #[serde(default)]
    pub grpc_entry_port: Option<u16>,
    #[serde(default)]
    pub wss_entry_port: Option<u16>,
    pub log_level: String,
    pub trace_enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerEgressUpstream {
    #[serde(default)]
    pub upstreams: HashMap<String, UpstreamConfig>,
    #[serde(default)]
    pub rules: EgressRules,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EgressRules {
    #[serde(default)]
    pub http: Vec<EgressRule>,
    #[serde(default)]
    pub grpc: Vec<GrpcEgressRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EgressRule {
    pub match_host: String,
    pub action_upstream: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrpcEgressRule {
    pub match_host: String,
    pub match_service: String,
    pub action_upstream: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TunnelManagement {
    #[serde(default)]
    pub server_ingress_routing: Option<ServerIngressRouting>,
    #[serde(default)]
    pub client_configs: Option<ClientConfigs>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerIngressRouting {
    #[serde(default)]
    pub rules: IngressRules,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct IngressRules {
    #[serde(default)]
    pub http: Vec<IngressRule>,
    #[serde(default)]
    pub grpc: Vec<GrpcIngressRule>,
    #[serde(default)]
    pub wss: Vec<IngressRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IngressRule {
    pub match_host: String,
    pub action_client_group: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrpcIngressRule {
    pub match_host: String,
    pub match_service: String,
    pub action_client_group: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfigs {
    #[serde(default)]
    pub client_egress_routings: HashMap<String, ClientEgressRouting>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientEgressRouting {
    pub config_version: String,
    #[serde(default)]
    pub upstreams: HashMap<String, UpstreamConfig>,
    #[serde(default)]
    pub rules: ClientEgressRules,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ClientEgressRules {
    #[serde(default)]
    pub http: Vec<ClientEgressRule>,
    #[serde(default)]
    pub wss: Vec<ClientEgressRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientEgressRule {
    pub match_host: String,
    pub action_upstream: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpstreamConfig {
    pub servers: Vec<UpstreamServerConfig>,
    #[serde(default)]
    pub lb_policy: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpstreamServerConfig {
    pub address: String,
    #[serde(default)]
    pub resolve: bool,
}

impl ServerConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ServerConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    

    pub fn bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.server.tunnel_port)
    }
    

    pub fn rules(&self) -> Vec<RuleConfig> {

        let mut rules = Vec::new();
        
        if let Some(ref egress) = self.server_egress_upstream {
            for (idx, rule) in egress.rules.http.iter().enumerate() {
                rules.push(RuleConfig {
                    rule_id: format!("egress_http_{}", idx),
                    rule_type: "http".to_string(),
                    match_host: rule.match_host.clone(),
                    match_path_prefix: String::new(),
                    match_header: HashMap::new(),
                    action_proxy_pass: rule.action_upstream.clone(),
                    action_set_host: String::new(),
                });
            }
        }
        
        rules
    }
    

    pub fn upstreams(&self) -> Vec<UpstreamConfigLegacy> {
        let mut upstreams = Vec::new();
        
        if let Some(ref egress) = self.server_egress_upstream {
            for (name, upstream) in &egress.upstreams {
                upstreams.push(UpstreamConfigLegacy {
                    name: name.clone(),
                    servers: upstream.servers.clone(),
                    lb_policy: upstream.lb_policy.clone(),
                });
            }
        }
        
        upstreams
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleConfig {
    pub rule_id: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    #[serde(default)]
    pub match_host: String,
    #[serde(default)]
    pub match_path_prefix: String,
    #[serde(default)]
    pub match_header: HashMap<String, String>,
    pub action_proxy_pass: String,
    #[serde(default)]
    pub action_set_host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpstreamConfigLegacy {
    pub name: String,
    pub servers: Vec<UpstreamServerConfig>,
    #[serde(default)]
    pub lb_policy: String,
}

 