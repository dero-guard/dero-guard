use failure::Error;

mod service;
mod vpn;

use dero_guard::{setup_logger, log};
use service::Service;
use vpn::VPN;

fn main() { // TODO Refactor
    if let Err(e) = setup_logger(true, true) {
        eprintln!("Error while initializing logger: {}", e);
        return;
    }

    if std::env::args().len() < 2 {
        log::info!("Usage: dero_guard_client [--disconnect <server_ip>] <dero_address>");
        return;
    }

    if let Err(e) = start_service() {
        log::error!("Error during starting service: {}", e);
    }
}

fn start_service() -> Result<(), Error> {
    let mut vpn = VPN::new()?;
    let mut args: Vec<String> = std::env::args().collect();
    if let Some((i, _)) = args.iter().enumerate().find(|(_, s)| *s == "--disconnect") {
        if let Some(address) = args.get(i + 1) {
            vpn.disconnect(address)?;
        } else {
            log::error!("--disconnect requires the server IP address");
        }

        return Ok(());
    }

    let mut service = Service::new("http://127.0.0.1:40404/json_rpc", vpn)?;
    let providers = service.get_providers();
    for provider in providers {
        log::info!(
            "Name: {} | Location: {} | Price per GB: {} | Address: {}",
            provider.name, provider.location, provider.rate, provider.dero_address
        );
    }

    service.connect(args.remove(1), 1)?;

    Ok(())
}
