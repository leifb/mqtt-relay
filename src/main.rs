use log::info;
use relay::Relay;

mod mappings;
mod relay;
mod config;

#[tokio::main]
async fn main() {
    colog::init();
    log_meta();
    Relay::run(config::auto_load()).await;
}

fn log_meta() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    info!("Starting {} {}", name, version);
}