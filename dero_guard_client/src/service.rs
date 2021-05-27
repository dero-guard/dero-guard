use dero_guard::service::CommonService;
use dero_guard::dero::*;
use dero_guard::json_rpc::{JsonRPCClient, JsonRPCError};
use dero_guard::wg::{BandwidthUsage, get_bandwidth};

use failure::Error;
use crate::vpn::*;

use std::thread;
use std::time::Duration;

pub struct Service {
    parent: CommonService,
    vpn: VPN,
    block_height: u64,

    connected: Option<String>
}

impl Service {
    pub async fn new(target: &str, vpn: VPN) -> Result<Service, JsonRPCError> {
        let client = JsonRPCClient::new(target);
        let mut service = Service {
            parent: CommonService::new(client).await?,
            vpn,
            block_height: 0,

            connected: None
        };

        service.block_height = service.parent.get_height().await?.height;

        Ok(service)
    }

    pub async fn connect(&mut self, public_key: String, amount: u64) -> Result<String, Error> {
        let param = Transfer {
            amount,
            destination: public_key,
            payload_rpc: vec![
                Argument {
                    name: "PK".into(),
                    datatype: "S".into(),
                    value: self.vpn.get_public_key().into()
                },
            ],
        };

        println!("sending TX for registration");
        self.parent.send_tx(param).await?;

        if let Some(address) = &self.connected {
            return Ok(address.clone());
        }

        while self.connected.is_none() {
            let height = self.parent.get_height().await?.height;
            if self.block_height < height {
                self.block_height = height;

                if let Ok(value) = self.parent.get_txs(GetTransfersParams { _in: true, min_height: height, }).await {
                    for entry in value.entries {
                        if entry.payload_rpc.len() == 4 {
                            println!("Found entry with 4 payload!");
                            let opt_pkey = self.parent.get_payload_value("PK", &entry.payload_rpc);
                            let opt_ip = self.parent.get_payload_value("IP", &entry.payload_rpc);
                            let opt_port = self.parent.get_payload_value_number("PO", &entry.payload_rpc);
                            let opt_local = self.parent.get_payload_value("LO", &entry.payload_rpc);
    
                            if opt_pkey.is_some() && opt_ip.is_some() && opt_port.is_some() && opt_local.is_some() {
                                println!("Connecting to VPN...");
                                let address = opt_ip.unwrap();
                                self.vpn.connect(opt_pkey.unwrap(), address.clone(), opt_port.unwrap(), opt_local.unwrap())?;

                                self.connected = Some(address);
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_secs(1));
        }

        Ok(self.connected.as_ref().unwrap().clone())
    }

    pub fn disconnect(&mut self) -> Result<(), Error> {
        Ok(if let Some(address) = &self.connected {
            self.vpn.disconnect(address)?
        } else {
            ()
        })
    }

    pub fn get_bandwidth(&self, public_key: String) -> Result<BandwidthUsage, Error> {
        Ok(get_bandwidth(&public_key)?)
    }

    pub fn get_providers(&self) -> Vec<Provider> {
        vec![Provider {
            location: "fr".into(),
            name: "Litarvan's test VPN".into(),
            rate: 0.01,

            public_key: "".into()
        }]
    }
}