use crate::eth_client::erc20_contract::ERC20Contract;
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

    //cargo test -p eth_client test_transfer_tx
    #[tokio::test]
    async fn test_transfer_tx() {
        // Set up environment variables for testing
        env::set_var("RPC_URL", "https://sepolia.base.org/");
        env::set_var("PRIVATE_KEY", "your_private_key_here");
        env::set_var("CHAIN_ID", "84532");
        env::set_var(
            "CONTRACT_ADDRESS",
            "0xE4AdB1819a91C88764cf52d2A9BA7e29BE2Fe087",
        );

        // Define test parameters - use a proper hex-encoded 32-byte transaction ID
        let tx_id = "7DE9F3368FDBA3E23CED4AB9F425475C848CFAD5E62B692AE9DAB70B374F087F".to_string();
        let tx_byte_code = "test_byte_code".to_string();
        let tx_fee = 1000;
        let eth_address = Address::from_str("0xE14d5eA54aa89c6c6E970167AA09FF262a51c8fD").unwrap();

        // Call the transfer_tx function
        let result = transfer_tx(tx_id, tx_byte_code, tx_fee, eth_address).await;
        match &result {
            Ok(tx_hash) => {
                println!("Transaction successful with hash: {:?}", tx_hash);
                assert_eq!(tx_hash.as_bytes().len(), 32); // H256 should be 32 bytes
            }
            Err(e) => {
                println!("Transaction failed with error: {:?}", e);
                panic!("Transaction failed: {}", e);
            }
        }
        // Assert that the result is Ok and contains a valid transaction hash
        // assert!(result.is_ok());
        // let tx_hash = result.unwrap();
        // assert_eq!(tx_hash.as_bytes().len(), 32); // H256 should be 32 bytes
    }

    //cargo test -p eth_client approve_function_with_zkos_contract_test
    #[tokio::test]
    async fn approve_function_with_zkos_contract_test() {
        // Set up environment variables for testing
        env::set_var("RPC_URL", "https://sepolia.base.org/");
        env::set_var("PRIVATE_KEY", "your_private_key_here");
        env::set_var("CHAIN_ID", "84532");
        env::set_var(
            "CONTRACT_ADDRESS",
            "0xE4AdB1819a91C88764cf52d2A9BA7e29BE2Fe087",
        );

        // Define test parameters
        let spender_address =
            Address::from_str("0xE4AdB1819a91C88764cf52d2A9BA7e29BE2Fe087").unwrap();
        let amount = U256::from(100000000000000000u64);

        // Create a wallet from the private key
        let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
        let wallet: LocalWallet = private_key.parse().expect("Invalid private key");
        let wallet = wallet.with_chain_id(84532u64);

        // Create a provider
        let provider =
            Provider::<Http>::try_from(env::var("RPC_URL").expect("RPC_URL must be set"))
                .expect("Invalid RPC URL");

        // Create a client
        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        // Create a zkos contract instance to get the USDC token address
        let contract_address =
            Address::from_str(&env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"))
                .expect("Invalid contract address");
        let zkos_contract = zkosContract::new(contract_address, client.clone());

        // Get the USDC token address from the zkos contract
        let usdc_token_address = zkos_contract
            .usdc_token()
            .call()
            .await
            .expect("Failed to get USDC token address");

        // Create an ERC20 contract instance for the USDC token
        let erc20_contract = ERC20Contract::new(usdc_token_address, client.clone());

        // Call the approve function on the ERC20 contract
        let call = erc20_contract.approve(spender_address, amount);
        let pending_tx = call.send().await.expect("Transaction failed");
        let tx_hash = pending_tx.tx_hash();

        // Assert that the transaction hash is valid
        assert_eq!(tx_hash.as_bytes().len(), 32); // H256 should be 32 bytes
    }

    //cargo test -p eth_client approve_function_with_erc20_contract_test
    #[tokio::test]
    async fn approve_function_with_erc20_contract_test() {
        // Set up environment variables for testing
        env::set_var("RPC_URL", "https://sepolia.base.org/");
        env::set_var("PRIVATE_KEY", "your_private_key_here");
        env::set_var("CHAIN_ID", "84532");
        env::set_var(
            "CONTRACT_ADDRESS",
            "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        );

        // contract address of zkos contract
        let spender_address =
            Address::from_str("0xE4AdB1819a91C88764cf52d2A9BA7e29BE2Fe087").unwrap();
        let amount = U256::from(100000000000000000u64);

        // Create a wallet from the private key
        let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
        let wallet: LocalWallet = private_key.parse().expect("Invalid private key");
        let wallet = wallet.with_chain_id(84532u64);

        // Create a provider
        let provider =
            Provider::<Http>::try_from(env::var("RPC_URL").expect("RPC_URL must be set"))
                .expect("Invalid RPC URL");

        // Create a client
        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        // Create a zkos contract instance to get the USDC token address
        let contract_address =
            Address::from_str(&env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"))
                .expect("Invalid contract address");
        let erc20_contract = ERC20Contract::new(contract_address, client.clone());

        // Call the approve function on the ERC20 contract
        let call = erc20_contract.approve(spender_address, amount);
        let pending_tx = call.send().await.expect("Transaction failed");
        let tx_hash = pending_tx.tx_hash();

        // Assert that the transaction hash is valid
        assert_eq!(tx_hash.as_bytes().len(), 32); // H256 should be 32 bytes
    }
}
