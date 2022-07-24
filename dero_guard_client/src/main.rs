use std::io::stdin;

use failure::Error;

mod service;
mod vpn;

use dero_guard::{setup_logger, log, clap, clap::Parser};
use service::Service;
use vpn::VPN;

const DEFAULT_DAEMON_ADDRESS: &str = "127.0.0.1:40402";
const DEFAULT_WALLET_ADDRESS: &str = "127.0.0.1:40403";

#[derive(Parser)]
pub struct Config {
    /// Enable the debug
    #[clap(short, long)]
    debug: bool,
    /// Disable the log file
    #[clap(short = 'f', long)]
    disable_file_logging: bool,
    /// Disconnect from the server address VPN
    #[clap(short = 'x', long)]
    disconnect: Option<String>,
    /// DERO daemon address
    #[clap(short = 'a', long, default_value_t = String::from(DEFAULT_DAEMON_ADDRESS))]
    daemon_address: String,
    /// DERO wallet address
    #[clap(short = 'w', long, default_value_t = String::from(DEFAULT_WALLET_ADDRESS))]
    wallet_address: String
}

fn main() {
    let config: Config = Config::parse();
    if let Err(e) = setup_logger(config.debug, config.disable_file_logging) {
        eprintln!("Error while initializing logger: {}", e);
        return;
    }

    if let Err(e) = start_service(config) {
        log::error!("Error during starting service: {}", e);
    }
}

fn start_service(config: Config) -> Result<(), Error> {
    let mut vpn = VPN::new()?;
    if let Some(address) = config.disconnect {
        log::info!("Trying to disconnect from '{}'", address);
        vpn.disconnect(&address)?;
        return Ok(())
    }

    let mut service = Service::new(&format!("http://{}/json_rpc", config.wallet_address), &format!("http://{}/json_rpc", config.daemon_address), vpn)?;
    let mut providers = service.get_providers();
    log::info!("Please select one of next providers ({}):", providers.len());
    log::info!("{0: <10} | {1: <10} | {2: <10} | {3: <10} | {4: <10}", "Id", "Name", "Location", "Price", "Address");
    for (i, provider) in providers.iter().enumerate() {
        log::info!(
            "{0: <10} | {1: <10} | {2: <10} | {3: <10} | {4: <10}",
            i, provider.name, provider.location, provider.rate, provider.dero_address
        );
    }

    log::info!("Provider selected: ");
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let id = input.trim().parse::<usize>()?;
    let selected = providers.remove(id);
    service.connect(selected.dero_address, selected.rate)?;

    Ok(())
}
