mod vpn;
mod service;

use failure::Error;
use tokio;
use service::Service;
use vpn::{VPNError, VPN};

#[tokio::main]
async fn main() {
    //start_vpn();
    if let Err(e) = start_service().await {
        eprintln!("Error during starting service: {}", e);
    }
}

async fn start_service() -> Result<(), Error> {
    let mut service = Service::new("http://127.0.0.1:40404/json_rpc", VPN::new()?).await?;
    service.run().await
}