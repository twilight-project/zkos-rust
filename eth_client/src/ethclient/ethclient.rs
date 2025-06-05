use ethers::prelude::*;
use std::{fs, sync::Arc};

// Replace with your Ethereum RPC URL and contract address
const ETH_RPC_URL: &str = "https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID";
const CONTRACT_ADDR: &str = "0xYourContractAddress";

#[tokio::main]
async fn eventListener() -> Result<(), Box<dyn std::error::Error>> {
    // Read the ABI from the file
    let abi_path = "zkosContract.abi";
    let contract_abi = fs::read_to_string(abi_path).expect("Failed to read ABI file");

    // Connect to the Ethereum provider
    let provider = match Provider::<Http>::try_from(ETH_RPC_URL) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to connect to Ethereum provider: {:?}", e);
            return Err(Box::new(e));
        }
    };
    let provider = Arc::new(provider);

    // Parse the contract address
    let contract_address: Address = match CONTRACT_ADDR.parse() {
        Ok(address) => address,
        Err(e) => {
            eprintln!("Failed to parse contract address: {:?}", e);
            return Err(Box::new(e));
        }
    };

    // Create a contract instance
    let contract = match Contract::from_json(provider.clone(), contract_address, contract_abi.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create contract instance: {:?}", e);
            return Err(Box::new(e));
        }
    };

    // Define the event filters
    let burn_request_filter = match contract.event::<(bool, u64, String, String, Address)>("BurnRequest") {
        Ok(filter) => filter,
        Err(e) => {
            eprintln!("Failed to create BurnRequest event filter: {:?}", e);
            return Err(Box::new(e));
        }
    };

    let mint_or_burn_filter = match contract.event::<(bool, u64, String, String, Address)>("MintOrBurn") {
        Ok(filter) => filter,
        Err(e) => {
            eprintln!("Failed to create MintOrBurn event filter: {:?}", e);
            return Err(Box::new(e));
        }
    };

    let transfer_tx_filter = match contract.event::<(String, String, u64, Address)>("TransferTx") {
        Ok(filter) => filter,
        Err(e) => {
            eprintln!("Failed to create TransferTx event filter: {:?}", e);
            return Err(Box::new(e));
        }
    };

    // Stream the events
    let mut burn_request_stream = match burn_request_filter.stream().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create BurnRequest event stream: {:?}", e);
            return Err(Box::new(e));
        }
    };

    let mut mint_or_burn_stream = match mint_or_burn_filter.stream().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create MintOrBurn event stream: {:?}", e);
            return Err(Box::new(e));
        }
    };

    let mut transfer_tx_stream = match transfer_tx_filter.stream().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create TransferTx event stream: {:?}", e);
            return Err(Box::new(e));
        }
    };

    println!("Listening for BurnRequest, MintOrBurn, and TransferTx events...");

    tokio::select! {
use crate::pgsql::sql::{save_mint_or_burn_event, save_burn_request_event, save_transfer_tx_event};

async fn event_listener() -> Result<(), Box<dyn std::error::Error>> {
    // Assume the streams (burn_request_stream, mint_or_burn_stream, transfer_tx_stream) are already set up

    println!("Listening for BurnRequest, MintOrBurn, and TransferTx events...");

    tokio::select! {
        // Handle BurnRequest events
        _ = async {
            while let Some(event) = burn_request_stream.next().await {
                match event {
                    Ok(log) => {
                        let (mint_or_burn, usdc_value, qq_account, encrypt_scalar, eth_address) = log;

                        // Access metadata
                        let block_number = log.block_number.unwrap_or_default() as i64;
                        let tx_hash = log.transaction_hash.unwrap_or_default().to_string();

                        println!("BurnRequest event detected:");
                        // Save the event data to the database
                        save_burn_request_event(
                            mint_or_burn,
                            usdc_value as i64,
                            &qq_account,
                            &encrypt_scalar,
                            &eth_address.to_string(),
                            block_number,
                            &tx_hash,
                        )
                        .await
                        .expect("Failed to save BurnRequest event to database");
                    }
                    Err(e) => {
                        eprintln!("Error in BurnRequest event stream: {:?}", e);
                    }

                    // call blockprocessing func
                }
            }
        } => {},

        // Handle MintOrBurn events
        _ = async {
            while let Some(event) = mint_or_burn_stream.next().await {
                match event {
                    Ok(log) => {
                        let (mint_or_burn, usdc_value, qq_account, encrypt_scalar, eth_address) = log;

                        // Access metadata
                        let block_number = log.block_number.unwrap_or_default() as i64;
                        let tx_hash = log.transaction_hash.unwrap_or_default().to_string();

                        println!("MintOrBurn event detected:");
                        // Save the event data to the database
                        save_mint_or_burn_event(
                            mint_or_burn,
                            usdc_value as i64,
                            &qq_account,
                            &encrypt_scalar,
                            &eth_address.to_string(),
                            block_number,
                            &tx_hash,
                        )
                        .await
                        .expect("Failed to save MintOrBurn event to database");
                    }
                    Err(e) => {
                        eprintln!("Error in MintOrBurn event stream: {:?}", e);
                    }

                    if mint_or_burn{
                    // call blockprocessing func
                    }
                }
            }
        } => {},

        // Handle TransferTx events
        _ = async {
            while let Some(event) = transfer_tx_stream.next().await {
                match event {
                    Ok(log) => {
                        let (tx_id, tx_bytecode, tx_fee, eth_address) = log;

                        // Access metadata
                        let block_number = log.block_number.unwrap_or_default() as i64;
                        let tx_hash = log.transaction_hash.unwrap_or_default().to_string();

                        println!("TransferTx event detected:");
     
                        // Save the event data to the database
                        save_transfer_tx_event(
                            &tx_id,
                            &tx_bytecode,
                            tx_fee as i64,
                            &eth_address.to_string(),
                            block_number,
                            &tx_hash,
                        )
                        .await
                        .expect("Failed to save TransferTx event to database");
                    }
                    Err(e) => {
                        eprintln!("Error in TransferTx event stream: {:?}", e);
                    }

                    // call blockprocessing func
                }
            }
        } => {},
    }

    Ok(())
}

    Ok(())
}