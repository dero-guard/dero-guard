use dero_guard::wg::{WireguardConfig, setup_interface, load_keys, WireguardError, apply_configuration, WireguardPeer};
use dero_guard::command::execute;

const BASE_ADDRESS: &str = "10.0.0";
const LOCAL_ADDRESS: &str = "10.0.0.1/24";
const SOURCE_ADDRESS: &str = "10.0.0.0/24";
const ADDRESS_MASK: &str = "24";
const OUTPUT_INTERFACE: &str = "eth0";
const PORT: u16 = 22350;

pub struct VPN {
    config: WireguardConfig,
    next_ip: u8
}

pub type VPNError = WireguardError;

impl VPN {
    pub fn new() -> Result<VPN, VPNError> {
        setup_interface(LOCAL_ADDRESS)?;
        execute(vec![
            "iptables",
            "-t", "nat",
            "-A", "POSTROUTING",
            "-s", SOURCE_ADDRESS,
            "-o", OUTPUT_INTERFACE,
            "-j", "MASQUERADE"
        ])?;

        let config = WireguardConfig {
            keys: load_keys()?,
            listen_port: PORT,
            peers: Vec::new()
        };
        apply_configuration(&config)?;

        Ok(VPN { config, next_ip: 2 })
    }

    pub fn get_public_key(&self) -> &str {
        &self.config.keys.public_key
    }

    pub fn get_port(&self) -> u16 {
        self.config.listen_port
    }

    pub fn add_client(&mut self, public_key: String) -> Result<String, VPNError> {
        let address = format!("{}.{}/{}", BASE_ADDRESS, self.next_ip, ADDRESS_MASK);

        self.next_ip += 1;
        self.config.peers.push(WireguardPeer {
            public_key: public_key.clone(),
            allowed_ips: address.clone(),
            endpoint: None
        });

        apply_configuration(&self.config)?;

        println!(" - Client '{}' ('{}') ready for connection", public_key, address);

        Ok(address)
    }

    pub fn remove_client(&mut self, public_key: String) -> Result<bool, VPNError> {
        let peers = &mut self.config.peers;
        let index = peers.iter().enumerate().find(|(_, p)| p.public_key == public_key);

        if let Some((index, _)) = index {
            let removed = peers.remove(index);
            apply_configuration(&self.config)?;

            println!(" - Disconnected client '{}' ('{}')", removed.public_key, removed.allowed_ips);

            Ok(true)
        } else {
            Ok(false)
        }
    }
}