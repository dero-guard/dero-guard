use std::cell::Cell;

use reqwest::{Client as HttpClient};

use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use failure::Fail;

const JSON_RPC_VERSION: &str = "2.0";
const PARSE_ERROR_CODE: i16 = -32700;
const INVALID_REQUEST_CODE: i16 = -32600;
const METHOD_NOT_FOUND_CODE: i16 = -32601;
const INVALID_PARAMS_CODE: i16 = -32602;
const INTERNAL_ERROR_CODE: i16 = -32603;

type JsonRPCResult<T> = Result<T, JsonRPCError>;

pub struct JsonRPCClient {
    http: HttpClient,
    target: String,
    count: Cell<usize>
}

impl JsonRPCClient {
    pub fn new(target: &str) -> Self {
        JsonRPCClient {
            http: HttpClient::new(),
            target: target.into(),
            count: Cell::new(0)
        }
    }

    pub async fn call<R: DeserializeOwned>(&self, method: &str) -> JsonRPCResult<R> {
        self.count.set(self.count.get() + 1);

        self.send(json!({
            "jsonrpc": JSON_RPC_VERSION,
            "method": method,
            "id": self.count.get()
        })).await
    }

    pub async fn call_with<P, R>(&self, method: &str, params: P) -> JsonRPCResult<R>
        where P: Serialize + Sized, R: DeserializeOwned
    {
        self.count.set(self.count.get() + 1);

        self.send(json!({
            "jsonrpc": JSON_RPC_VERSION,
            "method": method,
            "id": self.count.get(),
            "params": &params
        })).await
    }

    pub async fn notify(&self, method: &str) -> JsonRPCResult<()> {
        self.http.post(&self.target)
            .json(&json!({
                "jsonrpc": JSON_RPC_VERSION,
                "method": method
            }))
            .send()
            .await?;

        Ok(())
    }

    pub async fn notify_with<P>(&self, method: &str, params: P) -> JsonRPCResult<()>
        where P: Serialize + Sized
    {
        self.http.post(&self.target)
            .json(&json!({
                "jsonrpc": JSON_RPC_VERSION,
                "method": method,
                "params": &params
            }))
            .send()
            .await?;

        Ok(())
    }

    async fn send<R: DeserializeOwned>(&self, value: Value) -> JsonRPCResult<R> {
        println!("Request: {}", serde_json::to_string_pretty(&value)?);
        let mut response: Value = self.http.post(&self.target)
            .json(&value)
            .send()
            .await?
            .json()
            .await?;

        println!("Response: {}", serde_json::to_string_pretty(&response)?);
        if let Some(error) = response.get_mut("error") {
            let error: JsonRPCErrorResponse = serde_json::from_value(error.take())?;
            let data = match error.data {
                Some(content) => Some(serde_json::to_string_pretty(&content)?),
                None => None
            };

            return Err(match error.code {
                PARSE_ERROR_CODE => JsonRPCError::ParseError,
                INVALID_REQUEST_CODE => JsonRPCError::InvalidRequest,
                METHOD_NOT_FOUND_CODE => JsonRPCError::MethodNotFound,
                INVALID_PARAMS_CODE => JsonRPCError::InvalidParams,
                INTERNAL_ERROR_CODE => JsonRPCError::InternalError {
                    message: error.message.clone(),
                    data
                },
                code => JsonRPCError::ServerError {
                    code,
                    message: error.message.clone(),
                    data
                }
            });
        }

        Ok(serde_json::from_value(
            response.get_mut("result").ok_or(JsonRPCError::MissingResult)?.take()
        )?)
    }
}

#[derive(Deserialize)]
struct JsonRPCErrorResponse {
    code: i16,
    message: String,
    #[serde(default)]
    data: Option<Value>
}

#[derive(Debug, Fail)]
pub enum JsonRPCError {
    #[fail(display = "Server failed to parse request JSON data")]
    ParseError,

    #[fail(display = "Server received invalid JSON-RPC request")]
    InvalidRequest,

    #[fail(display = "Unknown method requested to the server")]
    MethodNotFound,

    #[fail(display = "Invalid parameters were provided")]
    InvalidParams,

    #[fail(display = "Server internal JSON-RPC error: {}", message)]
    InternalError {
        message: String,
        data: Option<String>
    },

    #[fail(display = "Server returned error: [{}] {}", code, message)]
    ServerError {
        code: i16,
        message: String,
        data: Option<String>
    },


    #[fail(display = "Server returned a response without result")]
    MissingResult,

    #[fail(display = "Error while (de)serializing JSON data: {}", inner)]
    SerializationError {
        inner: serde_json::Error
    },

    #[fail(display = "HTTP error during JSON-RPC communication: {}", inner)]
    HttpError {
        inner: reqwest::Error
    }
}

impl From<reqwest::Error> for JsonRPCError {
    fn from(err: reqwest::Error) -> Self {
        JsonRPCError::HttpError {
            inner: err
        }
    }
}

impl From<serde_json::Error> for JsonRPCError {
    fn from(err: serde_json::Error) -> Self {
        JsonRPCError::SerializationError {
            inner: err
        }
    }
}