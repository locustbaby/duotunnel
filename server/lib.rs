use clap::Parser;

mod app;
mod bootstrap;
mod cli;
pub(crate) mod config;
pub(crate) mod control_client;
pub(crate) mod egress;
pub(crate) mod handlers;
pub(crate) mod hot_reload;
pub(crate) mod listener_mgr;
pub(crate) mod local_auth;
pub(crate) mod metrics;
pub(crate) mod null_stores;
pub(crate) mod plugins;
pub(crate) mod registry;
mod runtime;
pub(crate) mod service;
mod supervisor;
pub(crate) mod tunnel_handler;
pub(crate) mod tunnel_service;

pub fn run() -> anyhow::Result<()> {
    runtime::run(app::ServerApp::new(cli::Cli::parse()).run())
}

pub(crate) use bootstrap::{build_routing_snapshot, RoutingSnapshot, ServerState};
pub(crate) use listener_mgr::{sync_all_listeners, sync_listener_subset, sync_listeners};
pub(crate) use tunnel_lib::PeekBufPool;
