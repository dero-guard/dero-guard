use tokio;
use failure::Error;

use serde_json::json;
use dero_guard::json_rpc::JsonRPCClient;
use dero_guard::dero::{BlockCountResponse, GetTransfersResponse, TransferParams, GetTransfersParams, Transfer, Argument, DataType};

#[tokio::main]
async fn main() {
    if let Err(error) = print_block_count().await {
        eprintln!("Error while querying block count: {}", error);
    }

    let get_transfers_params = GetTransfersParams {
        _in: true,
    };

    if let Err(error) = get_txs(get_transfers_params).await {
        eprintln!("Error while querying txs: {}", error);
    }

    let transfer_params: TransferParams = TransferParams {
        transfers: vec![
            Transfer {
                //scid: String,
                destination: String::from("deto1qxsq4rr3dwjce5h35lr424q3v523vrhmwjd2829tzm2ny2jfv7sjhzcfahu4n"),
                amount: 0, 
                burn: 0,
                payload_rpc: vec![
                    Argument {
                        name: String::from("MSG"),
                        datatype: DataType::String.to_string(),
                        value: json!("This is a test message from rust using json_rpc")
                    }
                ]
            }
        ]
    };
    if let Err(error) = send_tx(transfer_params).await {
        eprintln!("Error while sending tx with payload: {}", error);
    }
}

async fn print_block_count() -> Result<(), Error> { //daemon
    let client = JsonRPCClient::new("http://127.0.0.1:40402/json_rpc");
    let response: BlockCountResponse = client.call("getblockcount").await?;

    println!("Block count: {}", response.count);

    Ok(())
}

async fn get_txs(params: GetTransfersParams) -> Result<GetTransfersResponse, Error> {
    let client = JsonRPCClient::new("http://127.0.0.1:40403/json_rpc");
    let response: GetTransfersResponse = client.call_with("get_transfers", &params).await?;

    println!("response: {:?}", response);

    Ok(response)
}

async fn send_tx(params: TransferParams) -> Result<(), Error> {
    let client = JsonRPCClient::new("http://127.0.0.1:40403/json_rpc");
    client.notify_with("transfer", &params).await?;
    Ok(())
}