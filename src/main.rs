mod infrai_client;
mod marketplace_loop;

use infrai_client::InfraiClient;
use marketplace_loop::{handoff_order, BuyerUpdate, SellerAsset};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reject = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [flag] if flag == "--reject" => true,
        _ => return Err("usage: marketplace-agent-errors [--reject]".into()),
    };
    let client = InfraiClient::from_env()?;
    let handoff = handoff_order(&client,
        SellerAsset { asset_id: "asset-42".into(), title: "Rust observability kit".into(), price_cents: 2400 },
        BuyerUpdate { order_id: "order-7".into(), buyer: "buyer-9".into(), message: if reject { "" } else { "Please include the async example" }.into() }).await?;
    println!("handoff ready: {} -> {}", handoff.order_id, handoff.asset_id);
    Ok(())
}
