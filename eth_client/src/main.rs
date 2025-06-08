pub mod eth_client;
use eth_client::eth_client::event_listener;

#[tokio::main]
async fn main() {
    if let Err(e) = event_listener().await {
        eprintln!("Error in event_listener: {:?}", e);
    }
}
