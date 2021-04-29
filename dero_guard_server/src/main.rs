use serde::Deserialize;
use tokio;
use failure::Error;

use dero_guard::json_rpc::JsonRPCClient;

#[tokio::main]
async fn main() {
    if let Err(error) = print_block_count().await {
        eprintln!("Error while querying block count: {}", error);
    }
}

async fn print_block_count() -> Result<(), Error> {
    let client = JsonRPCClient::new("http://127.0.0.1:20206/json_rpc");
    let response: BlockCountResponse = client.call("getblockcount").await?;

    println!("Block count: {}", response.count);

    Ok(())
}

#[derive(Deserialize)]
struct BlockCountResponse {
    count: u64,
    status: String
}