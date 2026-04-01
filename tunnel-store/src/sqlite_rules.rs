use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::Row;
use tracing::info;

use crate::rules::{
    EgressVhostRule, GroupConfig, IngressTcpRule, IngressVhostRule, RoutingData, RuleStore,
    UpstreamDef,
};

/// Sentinel key in `routing_server_egress` that carries the server-level vhost rules JSON.
/// All other rows in that table represent named upstream definitions.
const EGRESS_VHOST_SENTINEL: &str = "__egress_rules__";

/// SQLite-backed RuleStore.
///
/// Schema (4 tables):
///
/// ```sql
/// routing_ingress_vhost   (id, match_host, action_client_group, action_proxy_name)
/// routing_ingress_tcp     (id, match_port, action_client_group, action_proxy_name)
/// routing_server_egress   (id, upstream_name, upstream_json, vhost_rules_json)
///   -- upstream_name = EGRESS_VHOST_SENTINEL row holds server-level vhost rules JSON
/// routing_client_groups   (id, group_id, config_version, upstreams_json, vhost_rules_json)
/// ```
pub struct SqliteRuleStore {
    pool: sqlx::sqlite::SqlitePool,
}

impl SqliteRuleStore {
    pub fn new(pool: sqlx::sqlite::SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS routing_ingress_vhost (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                match_host          TEXT NOT NULL,
                action_client_group TEXT NOT NULL,
                action_proxy_name   TEXT
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS routing_ingress_tcp (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                match_port          INTEGER NOT NULL,
                action_client_group TEXT NOT NULL,
                action_proxy_name   TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        // One row per upstream name; vhost_rules_json stores the server-level egress rules
        // (only needed for the "default" / server-egress scope).
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS routing_server_egress (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                upstream_name   TEXT UNIQUE NOT NULL,
                upstream_json   TEXT NOT NULL,
                vhost_rules_json TEXT NOT NULL DEFAULT '[]'
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS routing_client_groups (
                id               INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id         TEXT UNIQUE NOT NULL,
                config_version   TEXT NOT NULL DEFAULT '',
                upstreams_json   TEXT NOT NULL DEFAULT '{}',
                vhost_rules_json TEXT NOT NULL DEFAULT '[]'
            )",
        )
        .execute(&self.pool)
        .await?;

        info!("routing rule migrations applied");
        Ok(())
    }
}

#[async_trait]
impl RuleStore for SqliteRuleStore {
    async fn load_routing(&self) -> Result<RoutingData> {
        let (vhost_rows, tcp_rows, egress_rows, group_rows): (
            Vec<sqlx::sqlite::SqliteRow>,
            Vec<sqlx::sqlite::SqliteRow>,
            Vec<sqlx::sqlite::SqliteRow>,
            Vec<sqlx::sqlite::SqliteRow>,
        ) = tokio::try_join!(
            sqlx::query(
                "SELECT match_host, action_client_group, action_proxy_name
                 FROM routing_ingress_vhost ORDER BY id",
            )
            .fetch_all(&self.pool),
            sqlx::query(
                "SELECT match_port, action_client_group, action_proxy_name
                 FROM routing_ingress_tcp ORDER BY id",
            )
            .fetch_all(&self.pool),
            sqlx::query(
                "SELECT upstream_name, upstream_json, vhost_rules_json
                 FROM routing_server_egress ORDER BY id",
            )
            .fetch_all(&self.pool),
            sqlx::query(
                "SELECT group_id, config_version, upstreams_json, vhost_rules_json
                 FROM routing_client_groups ORDER BY id",
            )
            .fetch_all(&self.pool),
        )
        .context("load routing tables")?;

        let ingress_vhost: Vec<IngressVhostRule> = vhost_rows
            .iter()
            .map(|r| IngressVhostRule {
                match_host: r.get("match_host"),
                action_client_group: r.get("action_client_group"),
                action_proxy_name: r.get("action_proxy_name"),
            })
            .collect();

        let ingress_tcp: Vec<IngressTcpRule> = tcp_rows
            .iter()
            .map(|r| {
                let port: i64 = r.get("match_port");
                IngressTcpRule {
                    match_port: port as u16,
                    action_client_group: r.get("action_client_group"),
                    action_proxy_name: r.get("action_proxy_name"),
                }
            })
            .collect();

        let mut server_egress_upstreams = std::collections::HashMap::new();
        let mut server_egress_vhost_rules: Vec<EgressVhostRule> = Vec::new();

        for row in &egress_rows {
            let name: String = row.get("upstream_name");
            let upstream_json: String = row.get("upstream_json");
            let vhost_json: String = row.get("vhost_rules_json");

            if name == EGRESS_VHOST_SENTINEL {
                // Sentinel row: carries server-level vhost rules, not an upstream definition.
                let rules: Vec<EgressVhostRule> =
                    serde_json::from_str(&vhost_json).context("deserialize egress vhost rules")?;
                server_egress_vhost_rules = rules;
            } else {
                let def: UpstreamDef =
                    serde_json::from_str(&upstream_json).context("deserialize upstream")?;
                server_egress_upstreams.insert(name, def);
            }
        }

        let mut client_groups = std::collections::HashMap::new();
        for row in &group_rows {
            let group_id: String = row.get("group_id");
            let config_version: String = row.get("config_version");
            let upstreams_json: String = row.get("upstreams_json");
            let vhost_json: String = row.get("vhost_rules_json");

            let upstreams: std::collections::HashMap<String, UpstreamDef> =
                serde_json::from_str(&upstreams_json).context("deserialize group upstreams")?;
            let vhost_rules: Vec<EgressVhostRule> =
                serde_json::from_str(&vhost_json).context("deserialize group vhost rules")?;

            client_groups.insert(
                group_id,
                GroupConfig {
                    config_version,
                    upstreams,
                    vhost_rules,
                },
            );
        }

        Ok(RoutingData {
            ingress_vhost,
            ingress_tcp,
            server_egress_upstreams,
            server_egress_vhost_rules,
            client_groups,
        })
    }

    async fn save_routing(&self, data: &RoutingData) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // -- ingress vhost (replace all) --
        sqlx::query("DELETE FROM routing_ingress_vhost")
            .execute(&mut *tx)
            .await?;
        for r in &data.ingress_vhost {
            sqlx::query(
                "INSERT INTO routing_ingress_vhost
                 (match_host, action_client_group, action_proxy_name)
                 VALUES (?, ?, ?)",
            )
            .bind(&r.match_host)
            .bind(&r.action_client_group)
            .bind(&r.action_proxy_name)
            .execute(&mut *tx)
            .await?;
        }

        // -- ingress tcp (replace all) --
        sqlx::query("DELETE FROM routing_ingress_tcp")
            .execute(&mut *tx)
            .await?;
        for r in &data.ingress_tcp {
            sqlx::query(
                "INSERT INTO routing_ingress_tcp
                 (match_port, action_client_group, action_proxy_name)
                 VALUES (?, ?, ?)",
            )
            .bind(r.match_port as i64)
            .bind(&r.action_client_group)
            .bind(&r.action_proxy_name)
            .execute(&mut *tx)
            .await?;
        }

        // -- server egress upstreams --
        sqlx::query("DELETE FROM routing_server_egress")
            .execute(&mut *tx)
            .await?;
        let vhost_json =
            serde_json::to_string(&data.server_egress_vhost_rules).context("serialize egress vhost")?;
        sqlx::query(
            "INSERT INTO routing_server_egress (upstream_name, upstream_json, vhost_rules_json)
             VALUES (?, '{}', ?)",
        )
        .bind(EGRESS_VHOST_SENTINEL)
        .bind(&vhost_json)
        .execute(&mut *tx)
        .await?;
        for (name, def) in &data.server_egress_upstreams {
            let upstream_json = serde_json::to_string(def).context("serialize upstream")?;
            sqlx::query(
                "INSERT INTO routing_server_egress (upstream_name, upstream_json, vhost_rules_json)
                 VALUES (?, ?, '[]')",
            )
            .bind(name)
            .bind(&upstream_json)
            .execute(&mut *tx)
            .await?;
        }

        // -- client groups --
        sqlx::query("DELETE FROM routing_client_groups")
            .execute(&mut *tx)
            .await?;
        for (group_id, group) in &data.client_groups {
            let upstreams_json =
                serde_json::to_string(&group.upstreams).context("serialize group upstreams")?;
            let vhost_json =
                serde_json::to_string(&group.vhost_rules).context("serialize group vhost rules")?;
            sqlx::query(
                "INSERT INTO routing_client_groups
                 (group_id, config_version, upstreams_json, vhost_rules_json)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(group_id)
            .bind(&group.config_version)
            .bind(&upstreams_json)
            .bind(&vhost_json)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        info!("routing data saved to DB");
        Ok(())
    }

    async fn is_routing_empty(&self) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM routing_ingress_vhost")
            .fetch_one(&self.pool)
            .await
            .context("check routing_ingress_vhost count")?;
        Ok(count == 0)
    }
}
