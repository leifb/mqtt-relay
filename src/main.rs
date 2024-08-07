use log::info;
use relay::Relay;

mod mappings;
mod relay;
mod config;

fn main() {
    colog::init();
    log_meta();
    Relay::run(config::auto_load());
}

fn log_meta() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    info!("Starting {} {}", name, version);
}