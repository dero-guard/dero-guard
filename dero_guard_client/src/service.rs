use dero_guard::dero::*;
use dero_guard::json_rpc::{JsonRPCClient, JsonRPCError};
use dero_guard::service::CommonService;
use dero_guard::wg::{get_bandwidth, BandwidthUsage};

use crate::vpn::*;
use failure::Error;

use std::thread;
use std::time::Duration;

pub struct Service {
    parent: CommonService,
    daemon_rpc: JsonRPCClient,
    vpn: VPN,
    block_height: u64,

    connected: Option<(String, String)>
}

impl Service {
    pub fn new(target: &str, vpn: VPN) -> Result<Service, JsonRPCError> {
        let client = JsonRPCClient::new(target);
        let mut service = Service {
            parent: CommonService::new(client)?,
            vpn,
            daemon_rpc: JsonRPCClient::new("http://127.0.0.1:40402/json_rpc"), //TODO Config
            block_height: 0,

            connected: None
        };

        service.block_height = service.parent.get_height()?.height;

        Ok(service)
    }

    pub fn connect(&mut self, dero_address: String, amount: u64) -> Result<(String, String), Error> {
        let param = Transfer {
            amount,
            destination: dero_address,
            payload_rpc: vec![
                Argument {
                    name: "PK".into(),
                    datatype: "S".into(),
                    value: self.vpn.get_public_key().into()
                },
            ],
        };

        println!("sending TX for registration");
        self.parent.send_tx(param)?;

        if let Some(remote) = &self.connected {
            return Ok(remote.clone());
        }

        while self.connected.is_none() {
            let height = self.parent.get_height()?.height;
            if self.block_height < height {
                self.block_height = height;

                if let Ok(value) = self.parent.get_txs(GetTransfersParams { _in: true, min_height: height, }) {
                    for entry in value.entries {
                        if entry.payload_rpc.len() == 4 {
                            println!("Found entry with 4 payload!");
                            let opt_pkey = self.parent.get_payload_value("PK", &entry.payload_rpc);
                            let opt_ip = self.parent.get_payload_value("IP", &entry.payload_rpc);
                            let opt_port = self
                                .parent
                                .get_payload_value_number("PO", &entry.payload_rpc);
                            let opt_local = self.parent.get_payload_value("LO", &entry.payload_rpc);

                            if opt_pkey.is_some()
                                && opt_ip.is_some()
                                && opt_port.is_some()
                                && opt_local.is_some()
                            {
                                println!("Connecting to VPN...");
                                let address = opt_ip.unwrap();
                                let public_key = opt_pkey.unwrap();
                                self.vpn.connect(public_key.clone(), address.clone(), opt_port.unwrap(), opt_local.unwrap())?;

                                self.connected = Some((address, public_key));
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
        Ok(if let Some((address, _)) = &self.connected {
            self.vpn.disconnect(address)?
        } else {
            ()
        })
    }

    pub fn get_bandwidth(&self, public_key: String) -> Result<BandwidthUsage, Error> {
        Ok(get_bandwidth(&public_key)?)
    }

    fn get_provider(&self, id: u64) -> Result<Provider, Error> {
        let mut res: GetSCResponse = self
            .daemon_rpc
            .call_with(
                "getsc",
                GetSCParams {
                    code: false,
                    scid: String::from(
                        "94064cf9838a354e4afd8cd09a63ccfcffabdc879b331a71cbe8228ca7adfa7d",
                    ),
                    keysstring: vec![
                        format!("provider_{}_price", id),
                        format!("provider_{}_name", id),
                        format!("provider_{}_country", id),
                        format!("provider_{}_address", id),
                    ],
                },
            )?;

        let rate = (res.valuesstring.remove(0).parse::<u64>()? / 10 ^ 5) as f64; //TODO
        let name = res.valuesstring.remove(0);
        let location = res.valuesstring.remove(0);
        let dero_address = res.valuesstring.remove(0);

        Ok(Provider {
            location,
            rate,
            name,
            dero_address
        })
    }

    pub fn get_providers(&self) -> Vec<Provider> {
        //             dero_address: "deto1qxqp9lquvzejrxlct5jjlntwe26kuk53646gz8ege7k0ah9kzma7yts6fg3va".into()
        //TODO delete async
        let mut res: GetSCResponse = match self
            .daemon_rpc
            .call_with(
                "getsc",
                GetSCParams {
                    code: false,
                    scid: String::from(
                        "94064cf9838a354e4afd8cd09a63ccfcffabdc879b331a71cbe8228ca7adfa7d",
                    ), //TODO Config
                    keysstring: vec![String::from("total")],
                },
            )
        {
            Ok(res) => res,
            Err(err) => panic!("{}", err),
        };

        let total = match res.valuesstring.remove(0).parse::<u64>() {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
        let mut providers = vec![];
        let mut i = 0;
        while i < total {
            providers.push(match self.get_provider(i) {
                Ok(v) => v,
                Err(err) => panic!("{}", err),
            });
            i = i + 1;
        }

        providers
    }
}
