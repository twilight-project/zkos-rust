use super::*;
use jsonrpsee::{core::error::Error, server::logger::Params};


pub(super) fn get_utxo_id(
    params: Params<'_>,
) -> Result<serde_json::Value, Error> {
    
    let args: UtxoArgs = params.parse()?;

    address = match address::Standard::from_hex_with_error(&hex_str) { 
            Ok(addr) => addr,
            Err(e) => {
                let err = JsonRpcError::invalid_params(e.to_string());
                return Err(err);
            }
    };

        let utxos = search_coin_type_utxo_by_address(address);
        if utxos.len() > 0 {
            let response_body = serde_json::to_value(&utxos).expect("Failed to serialize to JSON");
            Ok(response_body)
        } else {
            let result = format!("{{ Error: Utxo not available for provided address}}");
            let response_body = serde_json::to_value(result).expect("Failed to serialize to JSON");
            Ok(response_body)
        }
}
