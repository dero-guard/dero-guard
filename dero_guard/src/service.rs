use crate::json_rpc::{JsonRPCClient, JsonRPCError};
use crate::dero::*;
use failure::Error;
use serde_json::json;

pub struct CommonService {
    client: JsonRPCClient,
}

impl CommonService {

    pub async fn new(client: JsonRPCClient) -> Result<CommonService, JsonRPCError> {
        let service = CommonService {
            client
        };

        service.get_height().await?;

        Ok(service)
    }

    pub fn get_payload_value(&self, name: &str, elements: &Vec<Argument>) -> Option<String> {
        for el in elements {
            if el.name == name && el.datatype == "S" {
                match el.value.clone() {
                    serde_json::Value::String(s) => return Some(s),
                    _ => {}
                };
            }
        }

        None
    }

    pub fn get_payload_value_number(&self, name: &str, elements: &Vec<Argument>) -> Option<u16> {
        for el in elements {
            if el.name == name && el.datatype == "U" {
                match el.value.clone() {
                    serde_json::Value::Number(s) => return s.as_u64().map(|e| e as u16),
                    _ => {}
                };
            }
        }

        None
    }

    pub async fn get_height(&self) -> Result<GetHeightResponse, JsonRPCError> {
        let response: GetHeightResponse = self.client.call("getheight").await?;
        Ok(response)
    }

    pub async fn get_txs(&self, params: GetTransfersParams) -> Result<GetTransfersResponse, Error> {
        let response: GetTransfersResponse = self.client.call_with("get_transfers", &params).await?;
        Ok(response)
    }

    pub async fn send_tx(&self, transfer: Transfer) -> Result<(), Error> {
        self.client.notify_with("transfer", json!({
            "transfers": vec![transfer]
        })).await?;
        Ok(())
    }

    pub async fn send_tx_to_sc(&self, transfer: TransferSC) -> Result<(), Error> {
        self.client.notify_with("transfer", json!({
            "transfers": vec![transfer]
        })).await?;
        Ok(())
    }
}