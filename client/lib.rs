use clap::Parser;

pub(crate) mod bootstrap;
pub(crate) mod egress;
pub(crate) mod ingress;
pub(crate) mod metrics;
pub(crate) mod plugins;
pub(crate) mod runtime;
pub(crate) mod tunnel;

pub fn run() -> anyhow::Result<()> {
    runtime::run(runtime::ClientApp::new(bootstrap::cli::Args::parse()).run())
}
