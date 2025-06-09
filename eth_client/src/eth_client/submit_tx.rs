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
    // dotenv().ok();
    let rpc_url = env::var("ETH_RPC_URL")?;
    let private_key = env::var("PRIVATE_KEY")?;
    let chain_id: u64 = env::var("CHAIN_ID")?.parse()?;
    let contract_address: Address = env::var("ZKOS_CONTRACT_ADDRESS")?.parse()?;

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

// cargo test -p eth_client -- --test-threads=1
#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::Address;
    use std::env;
    use std::str::FromStr;
    use tokio::time::{sleep, Duration};

    //cargo test -p eth_client test_transfer_tx
    #[tokio::test]
    async fn test_transfer_tx() {
        // Set up environment variables for testing
        dotenv::from_filename("eth_client/.env.test").ok();

        // Define test parameters - use a proper hex-encoded 32-byte transaction ID
        let tx_id = "7DE9F3368FDBA3E23CED4AB9F425475C848CFAD5E62B692AE9DAB70B374F087F".to_string();
        let tx_byte_code = "test_byte_code".to_string();
        let tx_fee = 1000;
        let eth_address =
            Address::from_str(&env::var("SENDER_ADDRESS").expect("SENDER_ADDRESS must be set"))
                .unwrap();

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

        // Add delay to prevent nonce conflicts
        sleep(Duration::from_secs(3)).await;
    }

    //cargo test -p eth_client approve_function_with_zkos_contract_test
    #[tokio::test]
    async fn approve_function_with_zkos_contract_test() {
        // Set up environment variables for testing
        dotenv::from_filename("eth_client/.env.test").ok();

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
            Provider::<Http>::try_from(env::var("ETH_RPC_URL").expect("ETH_RPC_URL must be set"))
                .expect("Invalid RPC URL");

        // Create a client
        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        // Create a zkos contract instance to get the USDC token address
        let contract_address = Address::from_str(
            &env::var("ZKOS_CONTRACT_ADDRESS").expect("ZKOS_CONTRACT_ADDRESS must be set"),
        )
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

        // Add delay to prevent nonce conflicts
        sleep(Duration::from_secs(3)).await;
    }

    //cargo test -p eth_client approve_function_with_erc20_contract_test
    #[tokio::test]
    async fn approve_function_with_erc20_contract_test() {
        // Set up environment variables for testing
        dotenv::from_filename("eth_client/.env.test").ok();

        // contract address of zkos contract
        let spender_address = Address::from_str(
            &env::var("ZKOS_CONTRACT_ADDRESS").expect("ZKOS_CONTRACT_ADDRESS must be set"),
        )
        .expect("Invalid contract address");
        let amount = U256::from(100000000000000000u64);

        // Create a wallet from the private key
        let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
        let wallet: LocalWallet = private_key.parse().expect("Invalid private key");
        let wallet = wallet.with_chain_id(
            env::var("CHAIN_ID")
                .expect("CHAIN_ID must be set")
                .parse::<u64>()
                .expect("Invalid chain id"),
        );

        // Create a provider
        let provider =
            Provider::<Http>::try_from(env::var("ETH_RPC_URL").expect("ETH_RPC_URL must be set"))
                .expect("Invalid RPC URL");

        // Create a client
        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        // Create a zkos contract instance to get the USDC token address
        let contract_address = Address::from_str(
            &env::var("USDC_TOKEN_ADDRESS").expect("USDC_TOKEN_ADDRESS must be set"),
        )
        .expect("Invalid contract address");
        let erc20_contract = ERC20Contract::new(contract_address, client.clone());

        // Call the approve function on the ERC20 contract
        let call = erc20_contract.approve(spender_address, amount);
        let pending_tx = call.send().await.expect("Transaction failed");
        let tx_hash = pending_tx.tx_hash();

        // Assert that the transaction hash is valid
        assert_eq!(tx_hash.as_bytes().len(), 32); // H256 should be 32 bytes

        // Add delay to prevent nonce conflicts
        sleep(Duration::from_secs(3)).await;
    }

    //cargo test -p eth_client mint_or_burn_function_test
    #[tokio::test]
    async fn mint_or_burn_function_test() {
        // Load environment variables from a custom .env file
        dotenv::from_filename("eth_client/.env.test").ok();

        // Set up environment variables for testing
        let rpc_url = env::var("ETH_RPC_URL").expect("ETH_RPC_URL must be set");
        let private_key = env::var("SENDER_PRIVATE_KEY").expect("SENDER_PRIVATE_KEY must be set");
        let sender_address = env::var("SENDER_ADDRESS").expect("SENDER_ADDRESS must be set");
        let contract_address = Address::from_str(
            &env::var("ZKOS_CONTRACT_ADDRESS").expect("ZKOS_CONTRACT_ADDRESS must be set"),
        )
        .expect("Invalid contract address");
        // Define the parameters for the mint_or_burn function
        let mint_or_burn = true;
        let usdt_value = 1000u64;
        let qq_account = "0cb213bda02291b578b8a201ba78c2c7b621002361966b2ad898f12af561c30713d072cffc9f6fd339d1580adf81a0312a65d7addc802de15c769e73c70476ce6b32d";
        let encrypt_scalar = "731218f83d7bec90eb0a30f3cd037b7e";
        let eth_address = Address::from_str(&sender_address).expect("Invalid sender address");

        // Create a wallet from the private key
        let wallet: LocalWallet = private_key.parse().expect("Invalid private key");
        let wallet = wallet.with_chain_id(84532u64);

        // Create a provider
        let provider = Provider::<Http>::try_from(rpc_url).expect("Invalid RPC URL");

        // Create a client
        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        let contract = zkosContract::new(contract_address, client.clone());

        // Call the mint function on the contract
        let call = contract.mint(
            mint_or_burn,
            usdt_value,
            qq_account.to_string(),
            encrypt_scalar.to_string(),
            eth_address,
        );
        let pending_tx = call.send().await.expect("Transaction failed");
        let tx_hash = pending_tx.tx_hash();

        // Print the transaction hash
        println!("Transaction hash: {:?}", tx_hash);

        // Assert that the transaction hash is valid
        assert_eq!(tx_hash.as_bytes().len(), 32); // H256 should be 32 bytes

        // Add delay to prevent nonce conflicts
        sleep(Duration::from_secs(3)).await;
    }
}
