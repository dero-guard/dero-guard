use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use dero_guard::command::execute;
use dero_guard::wg::*;
use dero_guard::log;

const BASE_ADDRESS: &str = "10.0.0";
const LOCAL_ADDRESS: &str = "10.0.0.1/24";
const SOURCE_ADDRESS: &str = "10.0.0.0/24";
const ADDRESS_MASK: &str = "24";
const OUTPUT_INTERFACE: &str = "eth0";
const PORT: u16 = 50000; //22350;

pub struct VPN {
    config: WireguardConfig,
    address: String,
    rate: Decimal,

    clients: Vec<Client>,
    next_ip: u8,
}

pub struct Client {
    public_key: String,
    balance: u64,
    last_download: u64,
    last_upload: u64,
}

pub type VPNError = WireguardError;

impl VPN {
    pub fn new(address: &str, rate: Decimal) -> Result<VPN, VPNError> {
        setup_interface(LOCAL_ADDRESS)?;
        apply_nat("-A")?;

        let config = WireguardConfig {
            keys: load_keys()?,
            listen_port: PORT,
            peers: Vec::new(),
        };
        apply_configuration(&config)?;

        Ok(VPN {
            config,
            address: address.into(),
            rate,

            clients: Vec::new(),
            next_ip: 2,
        })
    }

    pub fn get_public_key(&self) -> &str {
        &self.config.keys.public_key
    }

    pub fn get_address(&self) -> &str {
        &self.address
    }

    pub fn get_rate(&self) -> Decimal {
        self.rate
    }

    pub fn get_port(&self) -> u16 {
        self.config.listen_port
    }

    pub fn refill_client(&mut self, public_key: String, paid: u64) -> Result<String, VPNError> {
        let rate = self.rate;
        let client = if let Some(client) = self.find_client(&public_key) {
            client
        } else {
            log::debug!("Registering new client '{}'", public_key);

            self.clients.push(Client {
                public_key: public_key.clone(),
                balance: 0,
                last_download: 0,
                last_upload: 0,
            });
            self.find_client(&public_key).unwrap()
        };

        let paid = Decimal::from(paid) / dec!(100000);
        let amount = paid / rate;
        log::info!(
            "Client '{}' paid {} $DERO to add {} GB to its balance",
            public_key, paid, amount
        );

        client.balance += (amount * dec!(1000000000)).to_u64().unwrap();

        let peer = if let Some(peer) = self.find_peer(&public_key) {
            peer
        } else {
            log::debug!("Adding client '{}' to peers", public_key);
            return self.register_peer(public_key);
        };

        Ok(peer.allowed_ips.clone())
    }

    fn register_peer(&mut self, public_key: String) -> Result<String, VPNError> {
        let address = format!("{}.{}/{}", BASE_ADDRESS, self.next_ip, ADDRESS_MASK);

        self.next_ip += 1;
        self.config.peers.push(WireguardPeer {
            public_key: public_key.clone(),
            allowed_ips: address.clone(),
            endpoint: None,
        });

        apply_configuration(&self.config)?;

        log::info!(
            "Client '{}' ('{}') ready for connection",
            public_key, address
        );

        Ok(address)
    }

    fn find_client(&mut self, public_key: &String) -> Option<&mut Client> {
        self.clients
            .iter_mut()
            .find(|c| c.public_key == *public_key)
    }

    fn find_peer(&self, public_key: &String) -> Option<&WireguardPeer> {
        self.config
            .peers
            .iter()
            .find(|p| p.public_key == *public_key)
    }

    fn remove_peer(&mut self, public_key: &String) -> Result<bool, VPNError> {
        let peers = &mut self.config.peers;
        let index = peers
            .iter()
            .enumerate()
            .find(|(_, p)| p.public_key == *public_key);

        if let Some((index, _)) = index {
            let removed = peers.remove(index);
            apply_configuration(&self.config)?;

            log::info!(
                "Disconnected client '{}' ('{}')",
                removed.public_key, removed.allowed_ips
            );

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn update(&mut self) -> Result<(), VPNError> {
        let mut to_remove = Vec::<String>::new();

        for client in &mut self.clients {
            let bandwidth = get_bandwidth(&client.public_key)?;
            let diff = (bandwidth.download - client.last_download)
                + (bandwidth.upload - client.last_upload);

            log::debug!("Client '{}' balance -= {}", client.public_key, diff);
            log::debug!("Balance is now '{}'", client.balance);

            if client.balance <= diff {
                client.balance = 0;
                client.last_download = 0;
                client.last_upload = 0;

                to_remove.push(client.public_key.clone());

                log::debug!("Client '{}' balance is now empty", client.public_key);
            } else {
                client.balance -= diff;
                client.last_download = bandwidth.download;
                client.last_upload = bandwidth.upload;
            }
        }

        for key in to_remove {
            self.remove_peer(&key)?;
        }

        Ok(())
    }
}

pub fn flush() -> Result<(), VPNError> {
    apply_nat("-D")?;
    remove_interface()?;

    log::info!("Flushed");

    Ok(())
}

fn apply_nat(method: &str) -> Result<(), VPNError> {
    execute(vec![
        "iptables",
        "-t",
        "nat",
        method,
        "POSTROUTING",
        "-s",
        SOURCE_ADDRESS,
        "-o",
        OUTPUT_INTERFACE,
        "-j",
        "MASQUERADE",
    ])?;

    Ok(())
}
