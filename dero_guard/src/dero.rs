use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct BlockCountResponse {
    pub count: u64,
    pub status: String
}

#[derive(Deserialize, Debug)]
pub struct GetTransfersResponse {
    entries: Vec<Entry>
}

#[derive(Serialize)]
pub struct TransferParams {
    pub transfers: Vec<Transfer>
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

#[derive(Serialize)]
pub struct Transfer {
    //pub scid: String, TODO fix it
    pub destination: String,
    pub amount: u64,
    pub burn: u64,
    pub payload_rpc: Vec<Argument>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Argument {
    pub name: String,
    pub datatype: String,
    pub value: serde_json::Value
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DataType {
    String,
    Int64,
    Uint64,
    Float64,
    Hash, //256 bit hash
    Address,
    Time
}

impl DataType {
    pub fn to_string(&self) -> String {
        use DataType::*;
        match self {
            String => "S",
            Int64 => "I",
            Uint64 => "U",
            Float64 => "F",
            Hash => "H",
            Address => "A",
            Time => "T",
        }.to_string()
    }
}