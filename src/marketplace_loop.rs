use crate::infrai_client::{InfraiClient, InfraiError};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SellerAsset { pub asset_id: String, pub title: String, pub price_cents: u32 }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuyerUpdate { pub order_id: String, pub buyer: String, pub message: String }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderHandoff { pub order_id: String, pub asset_id: String, pub buyer: String }

pub fn valid_handoff(asset: &SellerAsset, update: &BuyerUpdate) -> bool {
    asset.price_cents > 0 && !update.message.trim().is_empty()
}

pub async fn handoff_order(client: &InfraiClient, asset: SellerAsset, update: BuyerUpdate) -> Result<OrderHandoff, InfraiError> {
    if !valid_handoff(&asset, &update) {
        let payload = json!({
            "title": "marketplace handoff rejected",
            "message": "asset price and buyer message are required",
            "level": "warning",
            "fingerprint": ["marketplace", "handoff-validation"],
            "exception": {"type": "ValidationError", "message": "invalid order handoff"},
            "context": {"order_id": update.order_id, "asset_id": asset.asset_id, "buyer": update.buyer}
        });
        client.capture(&payload).await?;
        return Err(InfraiError::Api("order handoff rejected".into()));
    }
    Ok(OrderHandoff { order_id: update.order_id, asset_id: asset.asset_id, buyer: update.buyer })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn rejects_empty_buyer_update_before_handoff() {
        let asset = SellerAsset { asset_id: "a1".into(), title: "Guide".into(), price_cents: 1200 };
        let update = BuyerUpdate { order_id: "o1".into(), buyer: "b1".into(), message: " ".into() };
        assert!(!valid_handoff(&asset, &update));
    }
}
