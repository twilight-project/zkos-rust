use ethers::{
    prelude::{abigen, Abigen},
    providers::{Http, Middleware, Provider, Ws},
    types::Address,
};

use ethers::prelude::*;

use tokio::join;
use std::{sync::Arc, time::Duration};
use tokio_stream::StreamExt;
use std::io::{self, Read, Cursor};
use std::fs;

use crate::eth_client::zkos_contract::{zkosContract, BurnRequestFilter, MintOrBurnFilter, TransferTxFilter};

const ETH_RPC_URL: &str = "https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID";
const ETH_WSS_URL: &str = "wss://mainnet.infura.io/ws/v3/YOUR_INFURA_PROJECT_ID";
const CONTRACT_ADDR: &str = "0xYourContractAddress";
const ABI_PATH: &str = "zkosContract.abi";

async fn event_listener() -> Result<(), Box<dyn std::error::Error>> {
    let ws_provider = Provider::<Ws>::connect(ETH_WSS_URL).await?;
    let client = Arc::new(ws_provider);
    let contract_address: Address = CONTRACT_ADDR.parse()?;
    let contract = zkosContract::new(
        contract_address,
        client,
    );

    let rpc_provider = Provider::<Http>::try_from(ETH_RPC_URL)?;
    let block_number: U64 = rpc_provider.get_block_number().await?;

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


async fn listen_mintBurn_events(contract: &zkosContract<Provider<Ws>>, block_number: U64) -> Result<(), Box<dyn std::error::Error>> {
    let events = contract.event::<MintOrBurnFilter>().from_block(block_number);
    let mut stream = events.stream().await?.take(1);

    while let Some(Ok(f)) = stream.next().await {
        println!("ApprovalFilter event: {f:?}");
    }

    Ok(())
}

async fn listen_transfertx_events(contract: &zkosContract<Provider<Ws>>, block_number: U64) -> Result<(), Box<dyn std::error::Error>> {
    let events = contract.event::<TransferTxFilter>().from_block(block_number);
    let mut stream = events.stream().await?.take(1);

    while let Some(Ok(f)) = stream.next().await {
        println!("ApprovalFilter event: {f:?}");
    }

    Ok(())
}

async fn listen_burn_req_events(contract: &zkosContract<Provider<Ws>>, block_number: U64) -> Result<(), Box<dyn std::error::Error>> {
    let events = contract.event::<BurnRequestFilter>().from_block(block_number);
    let mut stream = events.stream().await?.take(1);

    while let Some(Ok(f)) = stream.next().await {
        println!("ApprovalFilter event: {f:?}");
    }

    Ok(())
}