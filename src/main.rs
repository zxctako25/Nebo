mod app;
mod codec;
mod config;
mod forwarding;
mod limbo;
mod monitor;
mod motd;
mod nbt;
mod network;
mod protocol;
mod security;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::bootstrap::run().await
}
