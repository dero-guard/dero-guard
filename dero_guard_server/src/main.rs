use std::str::FromStr;

use tokio;
use failure::Error;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

mod vpn;
mod service;

use vpn::{VPN, flush};
use service::Service;

#[tokio::main]
async fn main() {
    if std::env::args().find(|a| a == "--flush").is_some() {
        if let Err(e) = flush() {
            eprintln!("Error while flushing devices: {}", e);
        }

        return;
    }

    if std::env::args().len() < 3 {
        println!("Usage: dero_guard_server (<public_ip_address> <rate> | --flush)");
        return;
    }

    let args = std::env::args().collect::<Vec<String>>();
    let address = args.get(1).unwrap();
    let rate = args.get(2).unwrap();

    if let Ok(rate) = Decimal::from_str(rate.as_str()) {
        if rate < dec!(0.00000001) {
            eprintln!("'{}' is too small", rate);
            return;
        }

        println!("Public I.P. address: {}", address);
        println!("Rate: 1GB = {} $DERO\n", rate);

        if let Err(error) = start_service(address, rate).await {
            eprintln!("Error during Service initialization: {}", error);
        }
    } else {
        eprintln!("'{}' is not a valid floating point number", rate);
    }
}

async fn start_service(address: &str, rate: Decimal) -> Result<(), Error> {
    let vpn = VPN::new(address, rate)?;
    let mut service = Service::new("http://127.0.0.1:40403/json_rpc", vpn).await?;

    service.run().await
}