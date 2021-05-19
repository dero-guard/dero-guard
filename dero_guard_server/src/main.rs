use tokio;

mod vpn;
mod service;

use failure::Error;
use vpn::{VPN, flush};
use service::Service;

#[tokio::main]
async fn main() {
    if std::env::args().len() < 2 {
        println!("Usage: dero_guard_server [--flush] <public_ip_address>");
        return;
    }

    if std::env::args().find(|a| a == "--flush").is_some() {
        if let Err(e) = flush() {
            eprintln!("Error while flushing devices: {}", e);
        }

        return;
    }

    if let Err(error) = start_service().await {
        eprintln!("Error during Service initialization: {}", error);
    }
}

async fn start_service() -> Result<(), Error> {
    let mut service = Service::new("http://127.0.0.1:40403/json_rpc", VPN::new()?).await?;
    service.run().await
}