use super::id::Id;
use super::method::Method;
// use curve25519_dalek::digest::Output;
use jsonrpc_core::response::{Failure, Output, Success};
use jsonrpc_core::Response as JsonRPCResponse;
use jsonrpc_core::Version;
use serde_derive::{Deserialize, Serialize};
// use super::method::Method;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, CONTENT_TYPE, USER_AGENT};
use serde_json::Error;
use transaction::Transaction;
// pub type TransactionStatusId = String;
use crate::TransactionStatusId;
fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcBody<T> {
    /// JSON-RPC version
    pub jsonrpc: Version,

    /// Identifier included in request
    pub id: Id,

    /// Request method
    pub method: Method,

    /// Request parameters (i.e. request object)
    pub params: T,
}

pub trait RpcRequest<T> {
    // fn remove(&mut self, order: T, cmd: RpcCommand) -> Result<T, std::io::Error>;
    fn new(request: T, method: Method) -> Self;

    fn new_with_id(id: Id, request: T, method: Method) -> Self;

    fn id(&self) -> &Id;

    fn params(&self) -> &T;

    fn get_method(&self) -> &Method;

    fn into_json(self) -> String;

    // fn send(self, url: String) -> Result<Response, reqwest::Error>;
    fn send(self, url: String) -> Result<RpcResponse<serde_json::Value>, reqwest::Error>;
    // fn response(resp: Result<Response, reqwest::Error>);
    // // -> Result<jsonrpc_core::Response, jsonrpc_core::Error>;
}

impl RpcRequest<Transaction> for RpcBody<Transaction> {
    fn new(request: Transaction, method: Method) -> Self {
        Self::new_with_id(Id::uuid_v4(), request, method)
    }

    fn new_with_id(id: Id, request: Transaction, method: Method) -> Self {
        Self {
            jsonrpc: Version::V2,
            id,
            method: method,
            params: request,
        }
    }

    fn id(&self) -> &Id {
        &self.id
    }

    fn params(&self) -> &Transaction {
        &self.params
    }
    fn into_json(self) -> String {
        let tx = serde_json::to_value(&Payload::new(self)).unwrap();
        let mut file = File::create("foo.txt").unwrap();
        file.write_all(&serde_json::to_vec_pretty(&tx.clone()).unwrap())
            .unwrap();
        tx
    }

    fn get_method(&self) -> &Method {
        &self.method
    }

    fn send(
        self,
        url: std::string::String,
    ) -> Result<RpcResponse<serde_json::Value>, reqwest::Error> {
        match self.method {
            Method::TxCommit => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":1,"id":1}"#)
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[],"id":1}"#)
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[1],"id":1}"#)
                    .send();

                return rpc_response(res);
            }
            Method::txCommit => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":1,"id":1}"#)
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[],"id":1}"#)
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[1],"id":1}"#)
                    .send();

                return rpc_response(res);
            }
            Method::TxQueue => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                return rpc_response(res);
            }
            Method::TxStatus => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                return rpc_response(res);
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Payload {
    pub jsonrpc: Version,

    /// Identifier included in request
    pub id: Id,

    /// Request method
    pub method: Method,

    /// Request parameters (i.e. request object)
    pub params: String,
}
impl Payload {
    pub fn new(data: RpcBody<Transaction>) -> Payload {
        let tx_data = bincode::serialize(&data.params).unwrap();
        Payload {
            jsonrpc: data.jsonrpc,
            id: data.id,
            method: data.method,
            params: hex::encode(&tx_data),
        }
    }
}
use std::fs::File;
use std::io::prelude::*;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: Version,

    /// Identifier included in request
    pub id: jsonrpc_core::Id,
    pub result: Result<T, jsonrpc_core::Error>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resp {
    /// Protocol version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<Version>,
    /// Result
    pub result: String,
    /// Correlation id
    pub id: Id,
}

pub fn rpc_response(
    resp: Result<Response, reqwest::Error>,
) -> Result<RpcResponse<serde_json::Value>, reqwest::Error> {
    match resp {
        Ok(response) => {
            // if response.status().is_success() {
            let output: Output = serde_json::from_slice(&response.bytes().unwrap()).unwrap();
            let kk = match output {
                Output::Success(s) => RpcResponse {
                    jsonrpc: s.jsonrpc.unwrap(),
                    id: s.id,
                    result: Ok(s.result),
                },
                Output::Failure(f) => RpcResponse {
                    jsonrpc: f.jsonrpc.unwrap(),
                    id: f.id,
                    result: Err(f.error),
                },
            };
            return Ok(kk);

            // } else { };
        }
        Err(arg) => Err(arg),
    }
}

impl RpcRequest<TransactionStatusId> for RpcBody<TransactionStatusId> {
    fn new(request: TransactionStatusId, method: Method) -> Self {
        Self::new_with_id(Id::uuid_v4(), request, method)
    }

    fn new_with_id(id: Id, request: TransactionStatusId, method: Method) -> Self {
        Self {
            jsonrpc: Version::V2,
            id,
            method: method,
            params: request,
        }
    }

    fn id(&self) -> &Id {
        &self.id
    }

    fn params(&self) -> &TransactionStatusId {
        &self.params
    }
    fn into_json(self) -> String {
        let tx = serde_json::to_string(&self).unwrap();
        let mut file = File::create("foo.txt").unwrap();
        file.write_all(&serde_json::to_vec_pretty(&tx.clone()).unwrap())
            .unwrap();
        tx
    }

    fn get_method(&self) -> &Method {
        &self.method
    }

    fn send(
        self,
        url: std::string::String,
    ) -> Result<RpcResponse<serde_json::Value>, reqwest::Error> {
        // let res: Result<reqwest::blocking::Response, reqwest::Error>;
        match self.method {
            Method::TxCommit => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":1,"id":1}"#)
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[],"id":1}"#)
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[1],"id":1}"#)
                    .send();

                return rpc_response(res);
            }
            Method::txCommit => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":1,"id":1}"#)
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[],"id":1}"#)
                    // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[1],"id":1}"#)
                    .send();

                return rpc_response(res);
            }
            Method::TxQueue => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                return rpc_response(res);
            }
            Method::TxStatus => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                return rpc_response(res);
            }
        }

        // let client = reqwest::blocking::Client::new();
        // let clint_clone = client.clone();
        // let res = clint_clone
        //     .post(url)
        //     .headers(construct_headers())
        //     .body(self.into_json())
        //     // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":1,"id":1}"#)
        //     // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[],"id":1}"#)
        //     // .body(r#"{"jsonrpc":"2.0","method":"TxQueue","params":[1],"id":1}"#)
        //     .send();

        // return rpc_response(res);
    }
}
