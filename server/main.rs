#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;
use anyhow::Result;

fn main() -> Result<()> {
    server::run()
}
