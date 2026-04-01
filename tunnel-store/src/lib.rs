mod token;
mod traits;
pub mod rules;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "sqlite")]
pub mod sqlite_rules;

pub use token::{generate_token, hash_token};
pub use traits::{AuthError, AuthResult, AuthStore, TokenListEntry};
pub use rules::{EgressVhostRule, GroupConfig, IngressTcpRule, IngressVhostRule, RoutingData, RuleStore, UpstreamDef, UpstreamServer};

#[cfg(feature = "sqlite")]
pub use sqlite::open_sqlite_pool;
