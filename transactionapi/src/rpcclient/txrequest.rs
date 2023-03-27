use super::id::Id;
use super::method::Method;
use jsonrpc_core::Error;
use jsonrpc_core::Version;
use serde_derive::{Deserialize, Serialize};
// use super::method::Method;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, CONTENT_TYPE, USER_AGENT};
use transaction::Transaction;
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

fn create_request(path: String, tx: Transaction) -> Response {
    let client = reqwest::blocking::Client::new();
    let clint_clone = client.clone();
    let res = clint_clone
            .post(path)
            .headers(construct_headers())
            .body("{\"jsonrpc\": \"2.0\", \"method\": \"CreateTraderOrder\", \"id\":123, \"params\": {\"account_id\":\"siddharth\",\"position_type\":\"LONG\",\"order_type\":\"MARKET\",\"leverage\":15.0,\"initial_margin\":2.0,\"available_margin\":2.0,\"order_status\":\"PENDING\",\"entryprice\":39000.01,\"execution_price\":44440.02} }")
            .send()
            .unwrap();
    return res;
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

    fn add_method(&self) -> &Method;

    fn into_json(self) -> String;

    fn send(self, url: String) -> Result<Response, reqwest::Error>;
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
        serde_json::to_string_pretty(&self).unwrap()
    }

    fn add_method(&self) -> &Method {
        &self.method
    }

    fn send(self, url: std::string::String) -> Result<Response, reqwest::Error> {
        let mut file = File::create("foo.txt").unwrap();
        file.write_all(&serde_json::to_vec(&self.clone()).unwrap())
            .unwrap();

        let client = reqwest::blocking::Client::new();
        let clint_clone = client.clone();
        let res = clint_clone
            .post(url)
            .headers(construct_headers())
            .body(self.into_json())
            .send();
        return res;
    }
}
use std::fs::File;
use std::io::prelude::*;
