use std::fs::OpenOptions;
use std::io::Write;

use dero_guard::command::execute;
use dero_guard::wg::*;
use dero_guard::log;

const PORT: u16 = 23500;

pub struct VPN {
    config: WireguardConfig,
}

pub type VPNError = WireguardError;

impl VPN {
    pub fn new() -> Result<VPN, VPNError> {
        Ok(VPN {
            config: WireguardConfig {
                keys: load_keys()?,
                listen_port: PORT,
                peers: Vec::new(),
            },
        })
    }

    pub fn get_public_key(&self) -> &str {
        &self.config.keys.public_key
    }

    pub fn connect(
        &mut self,
        public_key: String,
        address: String,
        port: u16,
        local_address: String,
    ) -> Result<(), VPNError> {
        let keys = load_keys()?;
        /*setup_interface(&local_address)?;

        self.config.peers.push(WireguardPeer {
            public_key,
            allowed_ips: "0.0.0.0/0".into(),
            endpoint: Some(format!("{}:{}", address, port)),
        });

        apply_configuration(&self.config)?;
        edit_route(&address, "add")?;*/

        let result = format!("\
[Interface]
PrivateKey = {}
Address = {}
DNS = 1.1.1.1, 1.0.0.1

[Peer]
PublicKey = {}
Endpoint = {}:{}
AllowedIPs = 0.0.0.0/0", keys.private_key, local_address, public_key, address, port);

        let file = get_folder()?.join("generated.conf");
        let mut config = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file)?;

        config.write_all(result.as_bytes())?;

        log::info!("Generated wireguard configuration at '{:?}'", file);

        // execute(vec!["wg", "setconf", DEVICE_NAME, &format!("{}", file.display())])?;
        execute(vec!["wg-quick", "up", file.to_str().unwrap()])?;

        log::info!(
            "Connected to '{}:{}', local address is '{}'",
            address, port, local_address
        );

        Ok(())
    }

    pub fn disconnect(&mut self, _: &str) -> Result<(), VPNError> {
        while self.config.peers.len() > 0 {
            self.config.peers.remove(0);
        }

        // edit_route(address, "del")?;
        // remove_interface()?;

        let file = get_folder()?.join("generated.conf");
        execute(vec!["wg-quick", "down", file.to_str().unwrap()])?;

        log::info!("Disconnected");

        Ok(())
    }
}

fn edit_route(address: &str, action: &str) -> Result<(), VPNError> {
    let route = execute(vec!["ip", "route", "get", &address])?;
    let route = route.split(" ").collect::<Vec<&str>>();

    execute(vec!["ip", "route", action, route[0], route[1], route[2]])?;
    execute(vec!["ip", "route", action, "0/1", "dev", DEVICE_NAME])?;
    execute(vec!["ip", "route", action, "128/1", "dev", DEVICE_NAME])?;

    Ok(())
}
