mod app;

use anyhow::Result;

pub(crate) use app::CtldApp;

pub(crate) fn run(app: CtldApp) -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(app.run())
}
