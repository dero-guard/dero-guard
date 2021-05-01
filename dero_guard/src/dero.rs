use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct BlockCountResponse {
    pub count: u64,
    pub status: String
}

#[derive(Deserialize, Debug)]
pub struct GetTransfersResponse {
    pub entries: Vec<Entry>
}

#[derive(Serialize)]
pub struct GetTransfersParams {
    //pub coinbase: bool,
    #[serde(rename(serialize = "in"))]
    pub _in: bool,
    /*pub out: bool,
    pub min_height: u64,
    pub max_height: u64,
    pub sender: String,
    pub receiver: String,
    pub dstport: u64,
    pub srcport: u64*/
}

#[derive(Deserialize, Debug)]
pub struct Entry {
    pub height: u64,
    pub topoheight: u64,
    pub blockhash: String,
    pub minerreward: u64,
    pub tpos: u64,
    pub pos: u64,
    pub coinbase: bool,
    pub incoming: bool,
    pub txid: String,
    pub destination: String,
    pub amount: u64,
    pub fees: u64,
    pub proof: String,
    pub status: u64,
    pub time: String, //TODO Time
    pub ewdata: String,
    pub data: String,
    pub payloadtype: u64,
    pub payload: String,
    pub payload_rpc: Vec<Argument>,
    pub sender: String,
    pub dstport: u64,
    pub srcport: u64,
}

#[derive(Deserialize)]
pub struct GetHeightResponse {
    pub height: u64
}

#[derive(Serialize)]
pub struct TransferSC {
    pub scid: String,
    pub amount: u64,
    pub payload_rpc: Vec<Argument>
}

#[derive(Serialize)]
pub struct Transfer {
    pub destination: String,
    pub amount: u64,
    pub payload_rpc: Vec<Argument>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Argument {
    pub name: String,
    pub datatype: String,
    pub value: serde_json::Value
}