use dero_guard::wg::{WireguardConfig, WireguardError, setup_interface, load_keys, WireguardPeer, apply_configuration, DEVICE_NAME};
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

        execute(vec!["ip", "route", "add", "", "via", &address])?;
        execute(vec!["ip", "route", "add", "0/1", "dev", DEVICE_NAME])?;
        execute(vec!["ip", "route", "add", "128/1", "dev", DEVICE_NAME])?;

        println!(" - Connected to '{}', local address is '{}'", address, local_address);

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), VPNError> {
        self.local_address = None;
        while self.config.peers.len() > 0 {
            self.config.peers.remove(0);
        }

        println!(" - Disconnected");

        Ok(())
    }
}