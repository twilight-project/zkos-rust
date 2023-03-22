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

/// JSON-RPC request wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize)]
pub struct Wrapper<R> {
    /// JSON-RPC version
    jsonrpc: Version,

    /// Identifier included in request
    id: Id,

    /// Request method
    method: Method,

    /// Request parameters (i.e. request object)
    params: R,
}

impl<R> Wrapper<R>
where
    R: RequestMessage,
{
    /// Create a new request wrapper from the given request.
    ///
    /// The ID of the request is set to a random [UUIDv4] value.
    ///
    /// [UUIDv4]: https://en.wikipedia.org/wiki/Universally_unique_identifier#Version_4_(random)
    pub fn new(request: R) -> Self {
        Self::new_with_id(Id::uuid_v4(), request)
    }

    pub(crate) fn new_with_id(id: Id, request: R) -> Self {
        Self {
            jsonrpc: Version::current(),
            id,
            method: request.method(),
            params: request,
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn params(&self) -> &R {
        &self.params
    }

    pub fn into_json(self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}
