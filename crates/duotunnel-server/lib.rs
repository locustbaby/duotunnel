use clap::Parser;

mod bootstrap;
pub(crate) mod control;
pub(crate) mod egress;
pub(crate) mod ingress;
mod runtime;

pub fn run() -> anyhow::Result<()> {
    runtime::run(runtime::ServerApp::new(bootstrap::cli::Cli::parse()).run())
}

pub(crate) use bootstrap::{build_routing_snapshot_with_health, RuntimeGeneration, ServerState};
pub(crate) use duotunnel_core::PeekBufPool;
