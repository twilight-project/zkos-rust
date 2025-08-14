# Chain Oracle

A Rust library for connecting to a Cosmos-based blockchain, retrieving, parsing, and analyzing blocks and transactions—designed for integration with zkVM and related systems.

## Features

- **Block Subscription:** Continuously fetches and processes new blocks from a Cosmos chain using a configurable RPC endpoint.
- **Block & Transaction Parsing:** Decodes block data, extracts transactions, and provides structured access to block and transaction fields.
- **Threaded Processing:** Utilizes a thread pool for efficient block serialization and processing.
- **Configurable:** Uses environment variables (e.g., `NYKS_BLOCK_SUBSCRIBER_URL`) for endpoint configuration.
- **Extensible Types:** Provides rich Rust structs for blocks, transactions, and related data, ready for further analysis or integration.

## Usage

Add to your workspace and import as a library:

```rust
use chain_oracle::{subscribe_block, Block, BlockRaw};
```

### Example: Subscribing to Blocks

```rust
use chain_oracle::pubsub_chain::subscribe_block;

fn main() {
    let (receiver, _handle) = subscribe_block(false);
    // Now you can receive parsed Block structs from the channel
}
```

### Example: Fetching Latest Block Height

```rust
use chain_oracle::BlockRaw;

let latest_height = BlockRaw::get_latest_block_height().unwrap();
```

## Environment Variables

- `NYKS_BLOCK_SUBSCRIBER_URL`  
  The base URL for the Cosmos chain RPC endpoint (default: `http://localhost:1317/`).

## Project Structure

- `block_types.rs` – Block and transaction data structures and parsing logic.
- `pubsub_chain.rs` – Block subscription and threaded processing.
- `transaction_types.rs` – Transaction message types and helpers.
- `threadpool.rs` – Simple thread pool implementation for concurrent processing.
- `example_main.rs` – Example usage (if present).

## Minimum Supported Rust Version

Rust **1.70** or newer.


## License

Licensed under [`Apache-2.0`](../../LICENSE). 
