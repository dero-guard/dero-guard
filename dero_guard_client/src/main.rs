use failure::Error;
use tokio;

mod vpn;
mod service;

use service::Service;
use vpn::VPN;

#[tokio::main]
async fn main() {
    if std::env::args().len() < 2 {
        println!("Usage: dero_guard_client [--disconnect <server_ip>]] <dero_address>");
        return;
    }

    if let Err(e) = start_service().await {
        eprintln!("Error during starting service: {}", e);
    }
}

async fn start_service() -> Result<(), Error> {
    let mut vpn = VPN::new()?;

    let args: Vec<String> = std::env::args().collect();
    if let Some((i, _)) = args.iter().enumerate().find(|(_, s)| *s == "--disconnect") {
        if let Some(address) = args.get(i + 1) {
            vpn.disconnect(address)?;
        } else {
            eprintln!("--disconnect requires the server IP address");
        }

        return Ok(());
    }

    let mut service = Service::new("http://127.0.0.1:40404/json_rpc", vpn).await?;
    service.run().await
}