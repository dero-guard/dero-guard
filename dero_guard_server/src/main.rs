use tokio;
use dero_guard::json_rpc::{JsonRPCClient, JsonRPCError};
use dero_guard::service::Service;

mod vpn;

use vpn::{VPN, VPNError};

#[tokio::main]
async fn main() {
    if std::env::args().len() < 2 {
        println!("Usage: dero_guard_server <public_ip_address>");
        return;
    }

    if let Err(error) = start_service().await {
        eprintln!("Error during Service initialization: {}", error);
    }

    /*if let Err(error) = start_vpn() {
        eprintln!("Error during VPN initialization: {}", error);
    }*/
}

async fn start_service() -> Result<(), JsonRPCError> {
    let client = JsonRPCClient::new("http://127.0.0.1:40403");
    let service = Service::new(client).await?;
    
    Ok(())
}

fn start_vpn() -> Result<(), VPNError> {
    let mut vpn = VPN::new()?;
    vpn.add_client("CLIENT PUBLIC KEY".into())?;

    Ok(())
}