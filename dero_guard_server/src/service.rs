use dero_guard::service::CommonService;
use dero_guard::dero::*;
use dero_guard::json_rpc::{JsonRPCClient, JsonRPCError};

use failure::Error;
use crate::vpn::*;

use std::thread;
use std::time::Duration;
use serde_json::Value;
use serde_json::json;

pub struct Service {
    parent: CommonService,
    vpn: VPN,
}

impl Service {
    pub async fn new(target: &str, vpn: VPN) -> Result<Service, JsonRPCError> {
        let client = JsonRPCClient::new(target);
        let service = Service {
            parent: CommonService::new(client, false).await?,
            vpn
        };

        Ok(service)
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            thread::sleep(Duration::from_secs(1));
            if let Ok(value) = self.parent.get_txs(GetTransfersParams { _in: true }).await {
                for entry in value.entries {
                    if entry.payload_rpc.len() == 1 {
                        println!("Found TX for VPN!");
                        let opt_pkey = self.parent.get_payload_value("PK", &entry.payload_rpc);

                        if let Some(pk) = opt_pkey {
                            println!("Adding user to VPN server!");
                            let local_address = self.vpn.add_client(pk)?;
                            let result = self.parent.send_tx(Transfer {
                                destination: entry.sender,
                                amount: 0,
                                payload_rpc: vec![
                                    Argument {
                                        name: "PK".into(),
                                        datatype: "S".into(),
                                        value: self.vpn.get_public_key().into(),
                                    },
                                    Argument {
                                        name: "IP".into(),
                                        datatype: "S".into(),
                                        value: Value::String(self.vpn.get_address()),
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
                            }).await;

                            if let Err(e) = result {
                                println!("Error while sending TX reply to User: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}