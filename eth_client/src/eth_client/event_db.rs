use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use r2d2::Pool;
use tokio_postgres::Error;

const POSTGRESQL_URL: &str = "postgresql://user:password@localhost/dbname"; // Replace with your actual PostgreSQL URL

lazy_static::lazy_static! {
    static ref POSTGRESQL_POOL_CONNECTION: Pool<PostgresConnectionManager<NoTls>> = {
        let manager = PostgresConnectionManager::new(POSTGRESQL_URL.parse().unwrap(), NoTls);
        Pool::new(manager).expect("Failed to create PostgreSQL connection pool")
    };
}

pub fn init_psql() {
    if let Err(e) = create_transfer_tx_event_table() {
        eprintln!("Error creating transfer_tx_events table: {:#?}", e);
    } else {
        println!("transfer_tx_events table created successfully");
    }

    if let Err(e) = create_mint_burn_event_table() {
        eprintln!("Error creating mint_or_burn_events table: {:#?}", e);
    } else {
        println!("mint_or_burn_events table created successfully");
    }

    if let Err(e) = create_burn_req_event_table() {
        eprintln!("Error creating burn_req_events table: {:#?}", e);
    } else {
        println!("burn_req_events table created successfully");
    }
}

fn create_transfer_tx_event_table() -> Result<(), Box<dyn std::error::Error>> {
    let query = "
        CREATE TABLE IF NOT EXISTS transfer_tx_events (
            tx_id TEXT PRIMARY KEY,
            tx_bytecode TEXT NOT NULL,
            tx_fee BIGINT NOT NULL, 
            eth_address TEXT NOT NULL,
            block_number BIGINT NOT NULL, 
            zkos_tx_id TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT NOW()
        );
    ";

    let mut client = POSTGRESQL_POOL_CONNECTION.get()?;
    client.execute(query, &[])?;
    Ok(())
}

fn create_mint_burn_event_table() -> Result<(), Box<dyn std::error::Error>> {
    let query = "
        CREATE TABLE IF NOT EXISTS mint_or_burn_events (
            mint_or_burn BOOLEAN NOT NULL,
            usdc_value BIGINT NOT NULL,
            qq_account TEXT NOT NULL,
            encrypt_scalar TEXT NOT NULL,
            eth_address TEXT NOT NULL,
            block_number BIGINT NOT NULL,
            tx_id TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT NOW()
        );
    ";

    let mut client = POSTGRESQL_POOL_CONNECTION.get()?;
    client.execute(query, &[])?;
    Ok(())
}

fn create_burn_req_event_table() -> Result<(), Box<dyn std::error::Error>> {
    let query = "
        CREATE TABLE IF NOT EXISTS burn_req_events (
            mint_or_burn BOOLEAN NOT NULL,
            usdc_value BIGINT NOT NULL,
            qq_account TEXT NOT NULL,
            encrypt_scalar TEXT NOT NULL,
            eth_address TEXT NOT NULL,
            block_number BIGINT NOT NULL,
            tx_id TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT NOW()
        );
    ";

    let mut client = POSTGRESQL_POOL_CONNECTION.get()?;
    client.execute(query, &[])?;
    Ok(())
}


pub async fn save_mint_or_burn_event(
    mint_or_burn: bool,
    usdc_value: i64,
    qq_account: &str,
    encrypt_scalar: &str,
    eth_address: &str,
    block_number: i64,
    transaction_hash: &str,
) -> Result<(), Error> {
    // Connect to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(POSTGRESQL_URL,
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
pub async fn save_burn_request_event(
    mint_or_burn: bool,
    usdc_value: i64,
    qq_account: &str,
    encrypt_scalar: &str,
    eth_address: &str,
    block_number: i64,
    transaction_hash: &str,
) -> Result<(), Error> {
    // Connect to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(POSTGRESQL_URL,
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
pub async fn save_transfer_tx_event(
    tx_id: &str,
    tx_bytecode: &str,
    tx_fee: i64,
    eth_address: &str,
    block_number: i64,
    transaction_hash: &str,
) -> Result<(), Error> {
    // Connect to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(POSTGRESQL_URL,
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
            "INSERT INTO transfer_tx_events (tx_id, tx_bytecode, tx_fee, eth_address, block_number, zkos_tx_id)
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