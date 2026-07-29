#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;
use anyhow::Result;

fn main() -> Result<()> {
    duotunnel_server::run()
}
