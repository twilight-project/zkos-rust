use ethers::{
    core::k256::elliptic_curve::rand_core::block, prelude::{abigen, Abigen}, providers::{Http, Middleware, Provider, Ws}, types::Address
};

use ethers::prelude::*;
use utxo_in_memory::blockoperations::blockprocessing;

use std::fs;
use hex;
use std::io::{self, Cursor, Read};
use std::{sync::Arc, time::Duration};
use tokio::join;
use tokio_stream::StreamExt;

use crate::{eth_client::zkos_contract::{
    zkosContract, BurnRequestFilter, MintOrBurnFilter, TransferTxFilter,
}};

use crate::eth_client::event_db::{save_burn_request_event, save_mint_or_burn_event, save_transfer_tx_event};


const ETH_RPC_URL: &str = "https://omniscient-holy-aura.base-sepolia.quiknode.pro/59f87730ae3774d9a9ff2a5e0514e89941bdc620/";
const ETH_WSS_URL: &str = "wss://omniscient-holy-aura.base-sepolia.quiknode.pro/59f87730ae3774d9a9ff2a5e0514e89941bdc620/";
// const ETH_RPC_URL: &str = "https://rpctest.twilight.rest/";
// const ETH_WSS_URL: &str = "wss://rpctest.twilight.rest/";
const CONTRACT_ADDR: &str = "0x3c17f344b3C4Fc1Fa8668E625d6b076380d659ca";
const PRIVATE_KEY: &str = "0x4c0883a69102937d6231471b5dbb62f6e2f8c7e8d9f8c7e8d9f8c7e8d9f8c7e8";

pub async fn event_listener() -> Result<(), Box<dyn std::error::Error>> {
    let ws_provider = Provider::<Ws>::connect(ETH_WSS_URL).await?;
    let client = Arc::new(ws_provider);
    let contract_address: Address = CONTRACT_ADDR.parse()?;
    let contract = zkosContract::new(contract_address, client);

    let rpc_provider = Provider::<Http>::try_from(ETH_RPC_URL)?;
    let block_number: U64 = rpc_provider.get_block_number().await?;
    println!("Current block number: {}", block_number);

    let (mint_burn_result, transfer_tx_result, burn_req_result) = join!(
        listen_mintBurn_events(&contract, block_number),
        listen_transfertx_events(&contract, block_number),
        listen_burn_req_events(&contract, block_number),
    );

    // Handle results
    if let Err(e) = mint_burn_result {
        eprintln!("Error in listen_mintBurn_events: {:?}", e);
    }
    if let Err(e) = transfer_tx_result {
        eprintln!("Error in listen_transfertx_events: {:?}", e);
    }
    if let Err(e) = burn_req_result {
        eprintln!("Error in listen_burn_req_events: {:?}", e);
    }

    Ok(())
}

async fn listen_mintBurn_events(
    contract: &zkosContract<Provider<Ws>>,
    block_number: U64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Listening for MintOrBurnFilter events...");

    let client = get_client().await;
    let client = Arc::new(client);

    let events = contract
        .event::<MintOrBurnFilter>()
        .from_block(block_number);
    let mut stream = events.subscribe_with_meta().await?.take(2);

    // Note that `log` has type AnswerUpdatedFilter
    while let Some(Ok((event, meta))) = stream.next().await {
        println!("Mint or burn event event: {:#?}", event);
        println!("Meta: {:#?}", meta);
        let qq_account = event.qq_account.to_string();
        let encrypt_scalar = event.encrypt_scalar.to_string();
        let eth_address = event.eth_address.to_string();
        let usdc_value = event.usdc_value as i64;
        let tx_id = hex::encode(meta.transaction_hash.0);
        let event_block_number = meta.block_number.as_u64() as i64;
        save_mint_or_burn_event(event.mint_or_burn, usdc_value, &qq_account, &encrypt_scalar, &eth_address, event_block_number, &tx_id).await?;
        blockprocessing::process_trade_mint(tx_id, qq_account.clone(), event.mint_or_burn, event.usdc_value , meta.block_number.as_u64());
    }

    Ok(())
}

async fn listen_transfertx_events(
    contract: &zkosContract<Provider<Ws>>,
    block_number: U64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Listening for transfer tx events...");

    let client = get_client().await;
    let client = Arc::new(client);

    let events = contract
        .event::<TransferTxFilter>()
        .from_block(block_number);
    let mut stream = events.subscribe_with_meta().await?.take(2);

    // Note that `log` has type AnswerUpdatedFilter
    while let Some(Ok((event, meta))) = stream.next().await {
        println!("transfer tx event: {:#?}", event);
        println!("Meta: {:#?}", meta);
        let transfer_tx_bytes = event.tx_byte_code.to_string();
        let transfer_tx_id = event.tx_id.to_string();
        let eth_address = event.eth_address.to_string();
        let fee_value = event.tx_fee as i64;
        let tx_id = hex::encode(meta.transaction_hash.0);
        let event_block_number = meta.block_number.as_u64() as i64;
        save_transfer_tx_event(&transfer_tx_id, &transfer_tx_bytes, fee_value, &eth_address, event_block_number, &tx_id).await?;
        blockprocessing::process_transfer(transfer_tx_id, transfer_tx_bytes, meta.block_number.as_u64());
    }

    Ok(())

}

async fn listen_burn_req_events(
    contract: &zkosContract<Provider<Ws>>,
    block_number: U64,
) -> Result<(), Box<dyn std::error::Error>> {


    println!("Listening for burn request events...");

    let client = get_client().await;
    let client = Arc::new(client);

    let events = contract
        .event::<BurnRequestFilter>()
        .from_block(block_number);
    let mut stream = events.subscribe_with_meta().await?.take(2);

    // Note that `log` has type AnswerUpdatedFilter
    while let Some(Ok((event, meta))) = stream.next().await {
        println!("Mint or burn event event: {:#?}", event);
        println!("Meta: {:#?}", meta);
        let qq_account = event.qq_account.to_string();
        let encrypt_scalar = event.encrypt_scalar.to_string();
        let eth_address = event.eth_address.to_string();
        let usdc_value = event.usdc_value as i64;
        let tx_id = hex::encode(meta.transaction_hash.0);
        let event_block_number = meta.block_number.as_u64() as i64;
        save_burn_request_event(event.mint_or_burn, usdc_value, &qq_account, &encrypt_scalar, &eth_address, event_block_number, &tx_id).await?;
        // blockprocessing::process_trade_mint(&tx_id, qq_account.clone(), event.mint_or_burn, event.usdc_value , meta.block_number.as_u64());
    }

    Ok(())
}


async fn confirmBurn(qq_account: String, encrypt_scalar: String, usdc_value: u64, eth_address: Address) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the Ethereum provider
    let contract_address: Address = CONTRACT_ADDR.parse()?;
    let provider = Provider::<Http>::try_from(ETH_RPC_URL)?;
    let wallet = PRIVATE_KEY.parse::<LocalWallet>()?;
    let client = Arc::new(SignerMiddleware::new(provider, wallet));
    let contract = zkosContract::new(contract_address, client.clone());

    let tx = contract.burn(false, usdc_value, qq_account, encrypt_scalar, eth_address).send().await?;
    println!("Transaction sent! Tx hash: {:?}", tx);
    Ok(())
}


async fn get_client() -> Provider<Ws> {
    Provider::<Ws>::connect(ETH_WSS_URL)
        .await
        .unwrap()
}