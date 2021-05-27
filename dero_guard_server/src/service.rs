use dero_guard::dero::*;
use dero_guard::json_rpc::{JsonRPCClient, JsonRPCError};
use dero_guard::service::CommonService;

use crate::vpn::*;
use failure::Error;

use serde_json::json;
use serde_json::Value;
use std::thread;
use std::time::Duration;

pub struct Service {
    parent: CommonService,
    vpn: VPN,
    block_height: u64,
}

impl Service {
    pub fn new(target: &str, vpn: VPN) -> Result<Service, JsonRPCError> {
        let client = JsonRPCClient::new(target);
        let mut service = Service {
            parent: CommonService::new(client)?,
            vpn,
            block_height: 0,
        };
        service.block_height = service.parent.get_height()?.height;

        Ok(service)
    }

    pub fn run(&mut self) -> Result<(), Error> {
        println!("Running Service!");
        loop {
            if let Err(err) = self.vpn.update() {
                eprintln!("Error during VPN update: {}", err);
            }

            let height = self.parent.get_height()?.height;
            if self.block_height < height {
                self.block_height = height;
                if let Ok(value) = self.parent.get_txs(GetTransfersParams { _in: true, min_height: height }) {
                    println!("TXs found: {}", value.entries.len());
                    for entry in value.entries {
                        if entry.payload_rpc.len() == 1 {
                            println!("Found TX for VPN!");
                            let opt_pkey = self.parent.get_payload_value("PK", &entry.payload_rpc);

                            if let Some(pk) = opt_pkey {
                                println!("Adding user to VPN server!");
                                let local_address = self.vpn.refill_client(pk, entry.amount)?;
                                let result = self.parent.send_tx(Transfer {
                                    destination: entry.sender,
                                    amount: 1,
                                    payload_rpc: vec![
                                        Argument {
                                            name: "PK".into(),
                                            datatype: "S".into(),
                                            value: self.vpn.get_public_key().into(),
                                        },
                                        Argument {
                                            name: "IP".into(),
                                            datatype: "S".into(),
                                            value: Value::String(self.vpn.get_address().into()),
                                        },
                                        Argument {
                                            name: "PO".into(),
                                            datatype: "U".into(),
                                            value: json!(self.vpn.get_port()),
                                        },
                                        Argument {
                                            name: "LO".into(),
                                            datatype: "S".into(),
                                            value: Value::String(local_address),
                                        },
                                    ]
                                });
    
                                if let Err(e) = result {
                                    println!("Error while sending TX reply to User: {}", e);
                                }
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_secs(1));
        }
    }
}
