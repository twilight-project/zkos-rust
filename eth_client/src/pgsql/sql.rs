use tokio_postgres::{NoTls, Error};

/// Function to save MintOrBurn event data into the PostgreSQL database
async fn save_mint_or_burn_event(
    mint_or_burn: bool,
    usdc_value: i64,
    qq_account: &str,
    encrypt_scalar: &str,
    eth_address: &str,
    block_number: i64,
    transaction_hash: &str,
) -> Result<(), Error> {
    // Connect to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=your_user dbname=your_db password=your_password",
        NoTls,
    )
    .await?;

    // Spawn the connection in a separate task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Insert the event data into the database
    client
        .execute(
            "INSERT INTO mint_or_burn_events (mint_or_burn, usdc_value, qq_account, encrypt_scalar, eth_address, block_number, transaction_hash)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &mint_or_burn,
                &usdc_value,
                &qq_account,
                &encrypt_scalar,
                &eth_address,
                &block_number,
                &transaction_hash,
            ],
        )
        .await?;

    println!("MintOrBurn event saved to database.");
    Ok(())
}

/// Function to save BurnRequest event data into the PostgreSQL database
async fn save_burn_request_event(
    mint_or_burn: bool,
    usdc_value: i64,
    qq_account: &str,
    encrypt_scalar: &str,
    eth_address: &str,
    block_number: i64,
    transaction_hash: &str,
) -> Result<(), Error> {
    // Connect to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=your_user dbname=your_db password=your_password",
        NoTls,
    )
    .await?;

    // Spawn the connection in a separate task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Insert the event data into the database
    client
        .execute(
            "INSERT INTO burn_request_events (mint_or_burn, usdc_value, qq_account, encrypt_scalar, eth_address, block_number, transaction_hash)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &mint_or_burn,
                &usdc_value,
                &qq_account,
                &encrypt_scalar,
                &eth_address,
                &block_number,
                &transaction_hash,
            ],
        )
        .await?;

    println!("BurnRequest event saved to database.");
    Ok(())
}

/// Function to save TransferTx event data into the PostgreSQL database
async fn save_transfer_tx_event(
    tx_id: &str,
    tx_bytecode: &str,
    tx_fee: i64,
    eth_address: &str,
    block_number: i64,
    transaction_hash: &str,
) -> Result<(), Error> {
    // Connect to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=your_user dbname=your_db password=your_password",
        NoTls,
    )
    .await?;

    // Spawn the connection in a separate task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Insert the event data into the database
    client
        .execute(
            "INSERT INTO transfer_tx_events (tx_id, tx_bytecode, tx_fee, eth_address, block_number, transaction_hash)
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &tx_id,
                &tx_bytecode,
                &tx_fee,
                &eth_address,
                &block_number,
                &transaction_hash,
            ],
        )
        .await?;

    println!("TransferTx event saved to database.");
    Ok(())
}