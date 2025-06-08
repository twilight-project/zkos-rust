use crate::eth_client::zkos_contract::zkosContract;
use dotenv::dotenv;
use ethers::prelude::*;
use ethers::types::H256;
use std::env;
use std::sync::Arc;

pub async fn transfer_tx(
    tx_id: String,
    tx_byte_code: String,
    tx_fee: u64,
    eth_address: Address,
) -> Result<H256, Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();
    let rpc_url = env::var("RPC_URL")?;
    let private_key = env::var("PRIVATE_KEY")?;
    let chain_id: u64 = env::var("CHAIN_ID")?.parse()?;
    let contract_address: Address = env::var("CONTRACT_ADDRESS")?.parse()?;

    // Create a provider
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // Create a wallet from the private key
    let wallet: LocalWallet = private_key.parse()?;
    let wallet = wallet.with_chain_id(chain_id);

    // Create a client
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    // Create a contract instance
    let contract = zkosContract::new(contract_address, client.clone());

    // Call the transfer_tx function
    let call = contract.transfer_tx(tx_id, tx_byte_code, tx_fee, eth_address);
    let pending_tx = call.send().await?;
    let tx_hash = pending_tx.tx_hash();
    Ok(tx_hash)
}
#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::Address;
    use std::env;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_transfer_tx() {
        // Set up environment variables for testing
        env::set_var("RPC_URL", "http://localhost:8545");
        env::set_var("PRIVATE_KEY", "your_private_key_here");
        env::set_var("CHAIN_ID", "1");
        env::set_var(
            "CONTRACT_ADDRESS",
            "0x0000000000000000000000000000000000000000",
        );

        // Define test parameters
        let tx_id = "test_tx_id".to_string();
        let tx_byte_code = "test_byte_code".to_string();
        let tx_fee = 1000;
        let eth_address = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();

        // Call the transfer_tx function
        let result = transfer_tx(tx_id, tx_byte_code, tx_fee, eth_address).await;

        // Assert that the result is Ok and contains a valid transaction hash
        assert!(result.is_ok());
        let tx_hash = result.unwrap();
        assert_eq!(tx_hash.as_bytes().len(), 32); // H256 should be 32 bytes
    }
}
