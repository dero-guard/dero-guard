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
        let param = Transfer {
            amount: 1,
            destination: "deto1qxqqcczpsnr3plderemw8gyzuek0tqjwjyg5ufxkerh4cmeh2jgaaqc5658tl".into(), //TODO config
            payload_rpc: vec![
                Argument {
                    name: "PK".into(),
                    datatype: "S".into(),
                    value: self.vpn.get_public_key().into()
                },
            ],
        };

        self.parent.send_tx(param).await?;

        loop {
            thread::sleep(Duration::from_secs(1));
            if let Ok(value) = self.parent.get_txs(GetTransfersParams { _in: true }).await {
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
    }
}