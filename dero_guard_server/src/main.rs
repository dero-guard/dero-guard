use tokio;

mod vpn;
mod service;

use failure::Error;
use vpn::VPN;
use service::Service;

#[tokio::main]
async fn main() {
    if let Err(error) = start_service().await {
        eprintln!("Error during Service initialization: {}", error);
    }
}

async fn start_service() -> Result<(), Error> {
    let mut vpn = VPN::new()?;
    let mut service = Service::new("http://127.0.0.1:40403/json_rpc", vpn).await?;
    service.run().await?
}