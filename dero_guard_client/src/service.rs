use dero_guard::service::CommonService;
use dero_guard::dero::*;
use dero_guard::json_rpc::{JsonRPCClient, JsonRPCError};

use failure::Error;
use crate::vpn::*;

use std::thread;
use std::time::Duration;

pub struct Service {
    parent: CommonService,
    vpn: VPN,
    block_height: u64,
}

impl Service {
    pub async fn new(target: &str, vpn: VPN) -> Result<Service, JsonRPCError> {
        let client = JsonRPCClient::new(target);
        let mut service = Service {
            parent: CommonService::new(client, false).await?,
            vpn,
            block_height: 0,
        };

        service.block_height = service.parent.get_height().await?.height;

        Ok(service)
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let param = Transfer {
            amount: 1,
            destination: std::env::args().collect::<Vec<String>>().remove(1).into(), //TODO config
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

        loop {
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
                                self.vpn.connect(opt_pkey.unwrap(), opt_ip.unwrap(), opt_port.unwrap(), opt_local.unwrap())?;
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_secs(1));
        }
    }
}