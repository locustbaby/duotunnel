use anyhow::Result;
use clap::Parser;

mod bootstrap;
mod control;
mod runtime;

pub fn run() -> Result<()> {
    runtime::run(runtime::CtldApp::new(bootstrap::cli::Args::parse()))
}
