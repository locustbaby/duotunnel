use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::Row;
use tracing::info;
use crate::rules::{
    ClientGroup, ClientUpstream, EgressUpstreamDef, EgressVhostRule, IngressListener,
    IngressListenerMode, IngressVhostRule, RoutingData, RuleStore, UpstreamServer,
};
pub struct SqliteRuleStore {
    pool: sqlx::sqlite::SqlitePool,
}
impl SqliteRuleStore {
    pub fn new(pool: sqlx::sqlite::SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ingress_listeners (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                port      INTEGER NOT NULL UNIQUE,
                mode      TEXT NOT NULL CHECK(mode IN ('http','tcp')),
                tcp_group TEXT,
                tcp_proxy TEXT
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ingress_vhost_rules (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                listener_id INTEGER NOT NULL REFERENCES ingress_listeners(id) ON DELETE CASCADE,
                match_host  TEXT NOT NULL,
                group_id    TEXT NOT NULL,
                proxy_name  TEXT NOT NULL,
                UNIQUE(listener_id, match_host)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_vhost_listener ON ingress_vhost_rules(listener_id)",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS client_groups (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id       TEXT UNIQUE NOT NULL,
                config_version TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS client_upstreams (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id  TEXT NOT NULL REFERENCES client_groups(group_id) ON DELETE CASCADE,
                name      TEXT NOT NULL,
                lb_policy TEXT NOT NULL DEFAULT 'round_robin',
                UNIQUE(group_id, name)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS client_upstream_servers (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                upstream_id INTEGER NOT NULL REFERENCES client_upstreams(id) ON DELETE CASCADE,
                address     TEXT NOT NULL,
                resolve     INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS egress_upstreams (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                name      TEXT UNIQUE NOT NULL,
                lb_policy TEXT NOT NULL DEFAULT 'round_robin'
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS egress_upstream_servers (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                upstream_id INTEGER NOT NULL REFERENCES egress_upstreams(id) ON DELETE CASCADE,
                address     TEXT NOT NULL,
                resolve     INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS egress_vhost_rules (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                match_host     TEXT UNIQUE NOT NULL,
                action_upstream TEXT NOT NULL
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
        let listener_rows = sqlx::query(
            "SELECT l.id, l.port, l.mode, l.tcp_group, l.tcp_proxy,
                    r.match_host, r.group_id, r.proxy_name
             FROM ingress_listeners l
             LEFT JOIN ingress_vhost_rules r ON r.listener_id = l.id
             ORDER BY l.id, r.id",
        )
        .fetch_all(&self.pool)
        .await
        .context("load ingress listeners")?;
        let mut listeners: Vec<IngressListener> = Vec::new();
        for row in &listener_rows {
            let id: i64 = row.get("id");
            let port: i64 = row.get("port");
            let mode: String = row.get("mode");
            if let Some(last) = listeners.last_mut() {
                if last.id == id {
                    if mode == "http" {
                        let match_host: Option<String> = row.try_get("match_host").ok();
                        if let Some(host) = match_host {
                            if let IngressListenerMode::Http { vhost } = &mut last.mode {
                                vhost.push(IngressVhostRule {
                                    match_host: host,
                                    group_id: row.get("group_id"),
                                    proxy_name: row.get("proxy_name"),
                                });
                            }
                        }
                    }
                    continue;
                }
            }
            let listener_mode = if mode == "tcp" {
                IngressListenerMode::Tcp {
                    group_id: row.get("tcp_group"),
                    proxy_name: row.get("tcp_proxy"),
                }
            } else {
                let mut vhost = Vec::new();
                let match_host: Option<String> = row.try_get("match_host").ok();
                if let Some(host) = match_host {
                    vhost.push(IngressVhostRule {
                        match_host: host,
                        group_id: row.get("group_id"),
                        proxy_name: row.get("proxy_name"),
                    });
                }
                IngressListenerMode::Http { vhost }
            };
            listeners.push(IngressListener { id, port: port as u16, mode: listener_mode });
        }
        let group_rows = sqlx::query(
            "SELECT g.group_id, g.config_version,
                    u.id AS upstream_id, u.name AS upstream_name, u.lb_policy,
                    s.address, s.resolve
             FROM client_groups g
             LEFT JOIN client_upstreams u ON u.group_id = g.group_id
             LEFT JOIN client_upstream_servers s ON s.upstream_id = u.id
             ORDER BY g.id, u.id, s.id",
        )
        .fetch_all(&self.pool)
        .await
        .context("load client groups")?;
        let mut groups: Vec<ClientGroup> = Vec::new();
        for row in &group_rows {
            let group_id: String = row.get("group_id");
            let upstream_id: Option<i64> = row.try_get("upstream_id").ok();
            if let Some(last_group) = groups.last_mut() {
                if last_group.group_id == group_id {
                    if let Some(uid) = upstream_id {
                        let address: Option<String> = row.try_get("address").ok();
                        if let Some(addr) = address {
                            let resolve: i64 = row.get("resolve");
                            if let Some(upstream) =
                                last_group.upstreams.iter_mut().find(|u| {
                                    let upstream_name: String = row.get("upstream_name");
                                    u.name == upstream_name
                                })
                            {
                                upstream.servers.push(UpstreamServer {
                                    address: addr,
                                    resolve: resolve != 0,
                                });
                            } else {
                                let upstream_name: String = row.get("upstream_name");
                                let lb_policy: String = row.get("lb_policy");
                                let resolve: i64 = row.get("resolve");
                                last_group.upstreams.push(ClientUpstream {
                                    name: upstream_name,
                                    lb_policy,
                                    servers: vec![UpstreamServer {
                                        address: addr,
                                        resolve: resolve != 0,
                                    }],
                                });
                            }
                            let _ = uid;
                        }
                    }
                    continue;
                }
            }
            let config_version: String = row.get("config_version");
            let mut upstreams = Vec::new();
            if let Some(_uid) = upstream_id {
                let address: Option<String> = row.try_get("address").ok();
                if let Some(addr) = address {
                    let upstream_name: String = row.get("upstream_name");
                    let lb_policy: String = row.get("lb_policy");
                    let resolve: i64 = row.get("resolve");
                    upstreams.push(ClientUpstream {
                        name: upstream_name,
                        lb_policy,
                        servers: vec![UpstreamServer { address: addr, resolve: resolve != 0 }],
                    });
                }
            }
            groups.push(ClientGroup { group_id, config_version, upstreams });
        }
        let egress_rows = sqlx::query(
            "SELECT u.id AS upstream_id, u.name, u.lb_policy, s.address, s.resolve
             FROM egress_upstreams u
             LEFT JOIN egress_upstream_servers s ON s.upstream_id = u.id
             ORDER BY u.id, s.id",
        )
        .fetch_all(&self.pool)
        .await
        .context("load egress upstreams")?;
        let mut egress_upstreams: Vec<EgressUpstreamDef> = Vec::new();
        for row in &egress_rows {
            let name: String = row.get("name");
            let address: Option<String> = row.try_get("address").ok();
            if let Some(last) = egress_upstreams.last_mut() {
                if last.name == name {
                    if let Some(addr) = address {
                        let resolve: i64 = row.get("resolve");
                        last.servers.push(UpstreamServer { address: addr, resolve: resolve != 0 });
                    }
                    continue;
                }
            }
            let lb_policy: String = row.get("lb_policy");
            let mut servers = Vec::new();
            if let Some(addr) = address {
                let resolve: i64 = row.get("resolve");
                servers.push(UpstreamServer { address: addr, resolve: resolve != 0 });
            }
            egress_upstreams.push(EgressUpstreamDef { name, lb_policy, servers });
        }
        let vhost_rows = sqlx::query(
            "SELECT match_host, action_upstream FROM egress_vhost_rules ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("load egress vhost rules")?;
        let egress_vhost_rules = vhost_rows
            .iter()
            .map(|r| EgressVhostRule {
                match_host: r.get("match_host"),
                action_upstream: r.get("action_upstream"),
            })
            .collect();
        Ok(RoutingData { ingress_listeners: listeners, client_groups: groups, egress_upstreams, egress_vhost_rules })
    }
    async fn save_routing(&self, data: &RoutingData) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM ingress_listeners").execute(&mut *tx).await?;
        for l in &data.ingress_listeners {
            match &l.mode {
                IngressListenerMode::Http { vhost } => {
                    let lid: i64 = sqlx::query_scalar(
                        "INSERT INTO ingress_listeners (port, mode) VALUES (?, 'http') RETURNING id",
                    )
                    .bind(l.port as i64)
                    .fetch_one(&mut *tx)
                    .await
                    .context("insert http listener")?;
                    for r in vhost {
                        sqlx::query(
                            "INSERT INTO ingress_vhost_rules (listener_id, match_host, group_id, proxy_name) VALUES (?, ?, ?, ?)",
                        )
                        .bind(lid)
                        .bind(&r.match_host)
                        .bind(&r.group_id)
                        .bind(&r.proxy_name)
                        .execute(&mut *tx)
                        .await
                        .context("insert vhost rule")?;
                    }
                }
                IngressListenerMode::Tcp { group_id, proxy_name } => {
                    sqlx::query(
                        "INSERT INTO ingress_listeners (port, mode, tcp_group, tcp_proxy) VALUES (?, 'tcp', ?, ?)",
                    )
                    .bind(l.port as i64)
                    .bind(group_id)
                    .bind(proxy_name)
                    .execute(&mut *tx)
                    .await
                    .context("insert tcp listener")?;
                }
            }
        }
        sqlx::query("DELETE FROM client_groups").execute(&mut *tx).await?;
        for g in &data.client_groups {
            sqlx::query(
                "INSERT INTO client_groups (group_id, config_version) VALUES (?, ?)",
            )
            .bind(&g.group_id)
            .bind(&g.config_version)
            .execute(&mut *tx)
            .await
            .context("insert client group")?;
            for u in &g.upstreams {
                let uid: i64 = sqlx::query_scalar(
                    "INSERT INTO client_upstreams (group_id, name, lb_policy) VALUES (?, ?, ?) RETURNING id",
                )
                .bind(&g.group_id)
                .bind(&u.name)
                .bind(&u.lb_policy)
                .fetch_one(&mut *tx)
                .await
                .context("insert client upstream")?;
                for s in &u.servers {
                    sqlx::query(
                        "INSERT INTO client_upstream_servers (upstream_id, address, resolve) VALUES (?, ?, ?)",
                    )
                    .bind(uid)
                    .bind(&s.address)
                    .bind(s.resolve as i64)
                    .execute(&mut *tx)
                    .await
                    .context("insert upstream server")?;
                }
            }
        }
        sqlx::query("DELETE FROM egress_upstreams").execute(&mut *tx).await?;
        for u in &data.egress_upstreams {
            let uid: i64 = sqlx::query_scalar(
                "INSERT INTO egress_upstreams (name, lb_policy) VALUES (?, ?) RETURNING id",
            )
            .bind(&u.name)
            .bind(&u.lb_policy)
            .fetch_one(&mut *tx)
            .await
            .context("insert egress upstream")?;
            for s in &u.servers {
                sqlx::query(
                    "INSERT INTO egress_upstream_servers (upstream_id, address, resolve) VALUES (?, ?, ?)",
                )
                .bind(uid)
                .bind(&s.address)
                .bind(s.resolve as i64)
                .execute(&mut *tx)
                .await
                .context("insert egress server")?;
            }
        }
        sqlx::query("DELETE FROM egress_vhost_rules").execute(&mut *tx).await?;
        for r in &data.egress_vhost_rules {
            sqlx::query(
                "INSERT INTO egress_vhost_rules (match_host, action_upstream) VALUES (?, ?)",
            )
            .bind(&r.match_host)
            .bind(&r.action_upstream)
            .execute(&mut *tx)
            .await
            .context("insert egress vhost rule")?;
        }
        tx.commit().await?;
        info!("routing data saved to DB");
        Ok(())
    }
    async fn is_routing_empty(&self) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ingress_listeners")
                .fetch_one(&self.pool)
                .await
                .context("check ingress_listeners count")?;
        Ok(count == 0)
    }
}
