#[cfg(feature = "server-config")]
pub mod config;
pub mod db;
pub mod rules;
mod token;
mod traits;

#[cfg(feature = "sqlite")]
pub use db::sqlite::open_sqlite_pool;
pub use duotunnel_core::{ClientStatus, TokenStatus};
pub use rules::{
    ClientGroup, ClientUpstream, EgressUpstreamDef, EgressVhostRule, IngressListener,
    IngressListenerMode, IngressVhostRule, RoutingData, RuleStore, UpstreamServer,
};
pub use token::{generate_token, hash_token, hash_token_bytes};
pub use traits::{AuthError, AuthResult, AuthStore, TokenListEntry};

#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use crate::db::sqlite::{open_sqlite_pool, SqliteAuthStore};
}

#[cfg(feature = "sqlite")]
pub mod sqlite_rules {
    pub use crate::db::rules::SqliteRuleStore;
}

#[cfg(feature = "server-config")]
pub mod server_config {
    pub use crate::config::*;
}
