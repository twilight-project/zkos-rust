use super::id::Id;
use super::method::Method;
// use curve25519_dalek::digest::Output;
use jsonrpc_core::response::Output;
use jsonrpc_core::Version;
use serde_derive::{Deserialize, Serialize};
use utxo_in_memory::pgsql::QueryUtxoFromDB;
// use super::method::Method;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, CONTENT_TYPE, USER_AGENT};
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
        let tx = serde_json::to_string(&Payload::new(self)).unwrap();
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
            Method::txCommit => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::TxQueue => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::TxStatus => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }

            // Method::allOutputs => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getUtxos => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::allUtxos => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::allMemoUtxos => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::allSateUtxos => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }

            // Method::getOutput => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getMemoOutput => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getStateOutput => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getUtxosFromDB => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            _ => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
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
    pub params: Vec<String>,
}
impl Payload {
    pub fn new(data: RpcBody<Transaction>) -> Payload {
        let tx_data = bincode::serialize(&data.params).unwrap();
        Payload {
            jsonrpc: data.jsonrpc,
            id: data.id,
            method: data.method,
            params: vec![hex::encode(&tx_data)],
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
            let rpc_response = match output {
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
            Ok(rpc_response)

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
            Method::txCommit => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::TxQueue => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::TxStatus => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            } // _ => {}
            // Method::allOutputs => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getUtxos => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::allUtxos => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::allMemoUtxos => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::allSateUtxos => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getOutput => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getMemoOutput => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getStateOutput => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            // Method::getUtxosFromDB => {
            //     let client = reqwest::blocking::Client::new();
            //     let clint_clone = client.clone();
            //     let res = clint_clone
            //         .post(url)
            //         .headers(construct_headers())
            //         .body(self.into_json())
            //         .send();

            //     return rpc_response(res);
            // }
            _ => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
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

impl RpcRequest<Vec<String>> for RpcBody<Vec<String>> {
    fn new(request: Vec<String>, method: Method) -> Self {
        Self::new_with_id(Id::uuid_v4(), request, method)
    }

    fn new_with_id(id: Id, request: Vec<String>, method: Method) -> Self {
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

    fn params(&self) -> &Vec<String> {
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
        match self.method {
            Method::txCommit => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::TxQueue => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::TxStatus => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::allOutputs => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::getUtxos => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::getStateUtxos => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::getMemoUtxos => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::allUtxos => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::allMemoUtxos => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::allSateUtxos => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::getOutput => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::getMemoOutput => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::getStateOutput => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            Method::getUtxosFromDB => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
        }
    }
}

impl RpcRequest<QueryUtxoFromDB> for RpcBody<QueryUtxoFromDB> {
    fn new(request: QueryUtxoFromDB, method: Method) -> Self {
        Self::new_with_id(Id::uuid_v4(), request, method)
    }

    fn new_with_id(id: Id, request: QueryUtxoFromDB, method: Method) -> Self {
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

    fn params(&self) -> &QueryUtxoFromDB {
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
        match self.method {
            Method::getUtxosFromDB => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
            _ => {
                let client = reqwest::blocking::Client::new();
                let clint_clone = client.clone();
                let res = clint_clone
                    .post(url)
                    .headers(construct_headers())
                    .body(self.into_json())
                    .send();

                rpc_response(res)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::rpcclient::method::*;
    use crate::rpcclient::txrequest::{Resp, RpcBody, RpcRequest};
    use std::fs::File;
    use std::io::prelude::*;
    use utxo_in_memory::pgsql::QueryUtxoFromDB;
    // cargo test -- --nocapture --test check_allOutputs_test --test-threads 5
    #[test]
    fn check_allOutputs_test() {
        let tx_send: RpcBody<Vec<String>> =
            RpcRequest::new(Vec::new(), crate::rpcclient::method::Method::allOutputs);
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        println!(
            "Response : {:#?}",
            AllOutputsResponse::get_response(res.unwrap())
        );
    }
    // cargo test -- --nocapture --test check_getUtxos_test --test-threads 5
    #[test]
    fn check_getUtxos_test() {
        let tx_send: RpcBody<Vec<String>> =
            RpcRequest::new(vec!["0cba90f5645c15f43b243dbca276d5a6f8e8308b89f6ce54a569ea52326ad736669242166e4b84335d9b59363bf98de48ba016f88cbff1eadcc30c78afda48353290251e90".to_string()], crate::rpcclient::method::Method::getUtxos);
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        let response = GetUtxosResponse::get_response(res.unwrap());
        println!("Utxo : {:#?}", response);
        let mut file = File::create("foo_response.txt").unwrap();
        file.write_all(&serde_json::to_vec_pretty(&response).unwrap())
            .unwrap();
    }
    // cargo test -- --nocapture --test check_getMemoUtxos_test --test-threads 5
    #[test]
    fn check_getMemoUtxos_test() {
        let tx_send: RpcBody<Vec<String>> =
            RpcRequest::new(vec!["0cba90f5645c15f43b243dbca276d5a6f8e8308b89f6ce54a569ea52326ad736669242166e4b84335d9b59363bf98de48ba016f88cbff1eadcc30c78afda48353290251e90".to_string()], crate::rpcclient::method::Method::getMemoUtxos);
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        let response = GetUtxosResponse::get_response(res.unwrap());
        println!("Utxo : {:#?}", response);
        let mut file = File::create("foo_response.txt").unwrap();
        file.write_all(&serde_json::to_vec_pretty(&response).unwrap())
            .unwrap();
    }
    // cargo test -- --nocapture --test check_getStateUtxos_test --test-threads 5
    #[test]
    fn check_getStateUtxos_test() {
        let tx_send: RpcBody<Vec<String>> =
            RpcRequest::new(vec!["0cba90f5645c15f43b243dbca276d5a6f8e8308b89f6ce54a569ea52326ad736669242166e4b84335d9b59363bf98de48ba016f88cbff1eadcc30c78afda48353290251e90".to_string()], crate::rpcclient::method::Method::getStateUtxos);
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        let response = GetUtxosResponse::get_response(res.unwrap());
        println!("Utxo : {:#?}", response);
        let mut file = File::create("foo_response.txt").unwrap();
        file.write_all(&serde_json::to_vec_pretty(&response).unwrap())
            .unwrap();
    }

    // cargo test -- --nocapture --test check_allUtxo_test --test-threads 5
    #[test]
    fn check_allUtxo_test() {
        let tx_send: RpcBody<Vec<String>> =
            RpcRequest::new(Vec::new(), crate::rpcclient::method::Method::allUtxos);
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        println!(
            "Response : {:#?}",
            AllUtxoResponse::get_response(res.unwrap())
        );
    }
    // cargo test -- --nocapture --test check_allMemoUtxos_test --test-threads 5
    #[test]
    fn check_allMemoUtxos_test() {
        let tx_send: RpcBody<Vec<String>> =
            RpcRequest::new(Vec::new(), crate::rpcclient::method::Method::allMemoUtxos);
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        println!(
            "Response : {:#?}",
            AllUtxoResponse::get_response(res.unwrap())
        );
    }
    // cargo test -- --nocapture --test check_allSateUtxos_test --test-threads 5
    #[test]
    fn check_allSateUtxos_test() {
        let tx_send: RpcBody<Vec<String>> =
            RpcRequest::new(Vec::new(), crate::rpcclient::method::Method::allSateUtxos);
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        println!(
            "Response : {:#?}",
            AllUtxoResponse::get_response(res.unwrap())
        );
    }
    // cargo test -- --nocapture --test check_getOutput_test --test-threads 5
    #[test]
    fn check_getOutput_test() {
        let tx_send: RpcBody<Vec<String>> = RpcRequest::new(
            vec!["3a8859ec23372a757d0bc0afc64f1642793ddbb941d4ef5bdc634161d2700fe402".to_string()],
            crate::rpcclient::method::Method::getOutput,
        );
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        println!(
            "Response : {:#?}",
            GetCoinOutputResponse::get_response(res.unwrap())
        );
    }
    // cargo test -- --nocapture --test check_getMemoOutput_test --test-threads 5
    #[test]
    fn check_getMemoOutput_test() {
        let tx_send: RpcBody<Vec<String>> = RpcRequest::new(
            vec!["83e7d0e449f640defd863d855c9e5d2f1419dc8fb3aef560f9bd823d6efc62c300".to_string()],
            crate::rpcclient::method::Method::getMemoOutput,
        );
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        println!(
            "Response : {:#?}",
            GetMemoOutputResponse::get_response(res.unwrap())
        );
    }
    // cargo test -- --nocapture --test check_getStateOutput_test --test-threads 5
    #[test]
    fn check_getStateOutput_test() {
        let tx_send: RpcBody<Vec<String>> = RpcRequest::new(
            vec!["b0ac2025a33a65b737adc1e5aa1c2b5670c864ea71b58f85afd3bbf50c1ba32d01".to_string()],
            crate::rpcclient::method::Method::getStateOutput,
        );
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        println!(
            "Response : {:#?}",
            GetStateOutputResponse::get_response(res.unwrap())
        );
    }
    // cargo test -- --nocapture --test check_getUtxosFromDB_test --test-threads 5
    #[test]
    fn check_getUtxosFromDB_test() {
        let tx_send: RpcBody<QueryUtxoFromDB> = RpcRequest::new(
            QueryUtxoFromDB {
                start_block: 0,
                end_block: 10000,
                limit: 5,
                pagination: 0,
                io_type: zkvm::zkos_types::IOType::Coin,
            },
            crate::rpcclient::method::Method::getUtxosFromDB,
        );
        let res: Result<
            crate::rpcclient::txrequest::RpcResponse<serde_json::Value>,
            reqwest::Error,
        > = tx_send.send("http://165.232.134.41:3030".to_string());

        println!(
            "Response : {:?}",
            GetUtxosFromDBResponse::get_response(res.unwrap())
        );
    }

    // cargo test -- --nocapture --test check_hexdata_test --test-threads 5
    #[test]
    fn check_hexdata_test() {
        let rpc_data:String="0a0000000000000021000000000000000bafb16883bc54b2ec81dd39ebdb4607889ccd779c703b7087a1e9db6a74042f00da000000000000000000000000000000ea44841062cbb162866734dbe58dbb0c34c510000576bf61f4bb5e6aa8088603ba4f574ca1de85d56b8073acfd107e244932be9ee14ab432ecbb9eb87151d6578a00000000000000306361306261303131643639343564373539373132343634643332656236353931373262366638663631343233646239623666323265346332356663626233623438336533636264646334613262306130306663316364353062303463643531646531316133616636363037626531386634346665313636373563363963636432303238336436616262130b000000000000210000000000000070cf8fba29fb9d782062213e5bc72db6643ffe9a3ba08d1d17af8e9abf0a5b6000da00000000000000000000000000000034ce921372068b0a488c9e3677514ea4b7b4159fb8608c55e277db00d41b8146f2902357f7cb05fc1b36d50f0b3b0605be7c216a554a0ebf42615d6c3b1c23078a00000000000000306339303838373935323833343866313032636339393432306632366532633637396635353733663166346333633363643934623937313061356334336539343333646333373938373061343839376639306332333338373463616136393037316463633837653633333834396134363030356162386364326436366561336330393133363765383530fd0b00000000000021000000000000001ba502e5ef1062756dfdf13931d8cb63693df3350a48b7cf5306f15997ebf48f00090100000000000001000000010000002a000000000000003138653036353936613032653739353866643036323361626630383663323531663539306339643162378a000000000000003063626139306635363435633135663433623234336462636132373664356136663865383330386238396636636535346135363965613532333236616437333636363932343231363665346238343333356439623539333633626639386465343862613031366638386362666631656164636333306337386166646134383335333239303235316539300100000000000000640000000000000000000000000000007f14af96f278a16cc0bcd14cf9e3053fbdc06ef9ec7e5cc6240fb1fb24f134030000000000643d00000000000021000000000000001ba502e5ef1062756dfdf13931d8cb63693df3350a48b7cf5306f15997ebf48f0151010000000000000200000002000000010000002a000000000000003138653036353936613032653739353866643036323361626630383663323531663539306339643162378a000000000000003063626139306635363435633135663433623234336462636132373664356136663865383330386238396636636535346135363965613532333236616437333636363932343231363665346238343333356439623539333633626639386465343862613031366638386362666631656164636333306337386166646134383335333239303235316539300100000000000000640000000000000000000000000000007f14af96f278a16cc0bcd14cf9e3053fbdc06ef9ec7e5cc6240fb1fb24f13403010100000000000000020000000100000000000000640000000000000000000000000000007f14af96f278a16cc0bcd14cf9e3053fbdc06ef9ec7e5cc6240fb1fb24f1340300000000643d00000000000021000000000000003cb13f7ab97d93017554fa3ad33e619d4977fc9562a8eeed276e9d599ab8894100090100000000000001000000010000002a000000000000003138653036353936613032653739353866643036323361626630383663323531663539306339643162378a000000000000003063333831616165383666363934383664316131386631386666306635623330613032346338393932396530346536646639396332636639313263656565656530346632393163313930333361646464616637353339643732353136323261653062343066346136386233363963333161336165663137633266303532646137333765623263383735610100000000000000640000000000000000000000000000003b527b214416126e32a59ab3b4b7cf6992bbef21b14fde8975f81afc7616ae020000000000294000000000000021000000000000003cb13f7ab97d93017554fa3ad33e619d4977fc9562a8eeed276e9d599ab889410151010000000000000200000002000000010000002a000000000000003138653036353936613032653739353866643036323361626630383663323531663539306339643162378a000000000000003063333831616165383666363934383664316131386631386666306635623330613032346338393932396530346536646639396332636639313263656565656530346632393163313930333361646464616637353339643732353136323261653062343066346136386233363963333161336165663137633266303532646137333765623263383735610100000000000000640000000000000000000000000000003b527b214416126e32a59ab3b4b7cf6992bbef21b14fde8975f81afc7616ae02010100000000000000020000000100000000000000640000000000000000000000000000003b527b214416126e32a59ab3b4b7cf6992bbef21b14fde8975f81afc7616ae0200000000294000000000000021000000000000008255432d2832f9c62fb7045798629da29f3c98ef05927196cf00de155721d66302da000000000000000000000000000000d0a5476e991acc667e106731759fc4bb936ea8d4d205ec66a8f2dc1da2b618752c2f830e72151a67922a7bda15f78265f2c14e7d92707eb131b9b2a0cd85d3218a00000000000000306331346538363033626134323939656333616461333033653135396461616362636266303763323035323664626336376533666436326433643536356437633631303633363632323163663939633733356164376663366336366565643762383865343138333234303465386636303239663136643465333931636533656532343962656437373338724000000000000021000000000000008255432d2832f9c62fb7045798629da29f3c98ef05927196cf00de155721d66303da000000000000000000000000000000a0bd5bbe2b82051053189351789fd9c11356196aa995c1b867b99a222de5b36398ec7b41948faddd0b8deb022f3db2d01423defededdaea6d9b46bf804952d798a00000000000000306366636335646363366433623339623866633532396366396431663830383539323637643239636664646638366564356236656362316564393733643833343232346333663034313864376536326337316139353233333265633730346663323135613534393532343236633561613463663963613032323338376238313737316333343664393937724000000000000021000000000000008255432d2832f9c62fb7045798629da29f3c98ef05927196cf00de155721d66300da00000000000000000000000000000094851c41660249b504b93be4155228c11fee4db421eb0f5ce6dc4558deed71039e0542bf4332a0df7791810c45543f7e34645b8e83614dd5f49a65381b16cd4e8a00000000000000306333616432333039613362366237646236656635313162656665616335393735623530393932346631633939663562646631356330316163356430653936383633636539373838313538343966383262386334626466303835333331666462323666333366666538333565343462313531646262303837666162616433343730313666323263363862724000000000000021000000000000008255432d2832f9c62fb7045798629da29f3c98ef05927196cf00de155721d66304da000000000000000000000000000000e2929e750c8c2ed240498de9338c2421cf76666b813ee1f6f36c59c4646bb201d8b3165f62a5d6216cd5ca7552325222458dcedd09953c2a9f83e6e741026b788a000000000000003063313038336431333131663432626334666130333133386161613831346163353265363166393934633062636238653861383861316133373064343333326532616438346563616265633436626137663365336434613330363361393861336237303533373261316539383035343939386165386230333930666437386365313265636238346436387240000000000000".to_string();

        let decode = hex::decode(rpc_data).unwrap();
        let bin_vec_output: Vec<zkvm::zkos_types::Utxo> = bincode::deserialize(&decode).unwrap();
        println!("data: {:?}", bin_vec_output);
    }
}
