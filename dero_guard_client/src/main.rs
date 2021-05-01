mod vpn;
mod service;

use failure::Error;
use tokio;
use service::Service;
use vpn::VPN;

#[tokio::main]
async fn main() {
    if std::env::args().len() < 2 {
        println!("Usage: dero_guard_client <dero_address>");
        return;
    }

    if let Err(e) = start_service().await {
        eprintln!("Error during starting service: {}", e);
    }
}

async fn start_service() -> Result<(), Error> {
    let mut service = Service::new("http://127.0.0.1:40404/json_rpc", VPN::new()?).await?;
    service.run().await
}