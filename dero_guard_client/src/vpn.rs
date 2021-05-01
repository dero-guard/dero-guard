use dero_guard::wg::*;
use dero_guard::command::execute;

const PORT: u16 = 23500;

pub struct VPN {
    config: WireguardConfig,
    local_address: Option<String>
}

pub type VPNError = WireguardError;

impl VPN {
    pub fn new() -> Result<VPN, VPNError> {
        Ok(VPN {
            config: WireguardConfig {
                keys: load_keys()?,
                listen_port: PORT,
                peers: Vec::new()
            },
            local_address: None
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
        local_address: String
    ) -> Result<(), VPNError> {
        setup_interface(&local_address)?;

        self.local_address = Some(local_address.clone());
        self.config.peers.push(WireguardPeer {
            public_key,
            allowed_ips: "0.0.0.0/0".into(),
            endpoint: Some(format!("{}:{}", address, port))
        });

        apply_configuration(&self.config)?;
        edit_route(&address, "add")?;

        println!(" - Connected to '{}', local address is '{}'", address, local_address);

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), VPNError> {
        while self.config.peers.len() > 0 {
            self.config.peers.remove(0);
        }

        if let Some(address) = &self.local_address {
            edit_route(address, "del")?;
        }

        self.local_address = None;
        println!(" - Disconnected");

        Ok(())
    }
}

fn edit_route(address: &str, action: &str) -> Result<(), VPNError> {
    let route = execute(vec!["ip", "route", "get", &address])?;
    let route = route
        .split(" ")
        .collect::<Vec<&str>>();

    execute(vec!["ip", "route", action, route[0], route[1], route[2]])?;
    execute(vec!["ip", "route", action, "0/1", "dev", DEVICE_NAME])?;
    execute(vec!["ip", "route", action, "128/1", "dev", DEVICE_NAME])?;

    Ok(())
}