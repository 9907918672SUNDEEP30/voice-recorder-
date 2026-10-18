//! # Phase 5: Flash Alpha Alerts & WebSocket Broadcasting Hub (`alert_hub.rs`)
//!
//! Sub-50ms distribution of 6-vector synthesized Alpha Cards to frontend clients,
//! mobile dashboards, and WebSocket subscribers.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// Six-Vector Institutional Mosaic Synthesis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SixVectorSynthesis {
    /// 1. Government Policy & Fiscal Subsidies (GST cuts, PLI, land concessions)
    pub policy_vector: String,
    /// 2. Technological & Product Moats (patents, proprietary specs)
    pub tech_moat_vector: String,
    /// 3. S-Curve Market Share Velocity (Inflection and market share accretion)
    pub s_curve_market_share_vector: String,
    /// 4. Management & Promoter DNA (Clean governance, 0.0% pledge, proven CEO)
    pub management_pedigree_vector: String,
    /// 5. Ground Customer Adoption & NPS (Organic pull without expensive PR)
    pub ground_adoption_vector: String,
    /// 6. Valuation Mispricing / Institutional Blind Spot (Depressed PE vs High ROCE)
    pub institutional_blind_spot_vector: String,
}

/// Real-time Institutional Flash Alpha Card
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlashAlphaCard {
    pub alert_id: String,
    pub timestamp_epoch_ms: u64,
    pub symbol: String,
    pub company_name: String,
    pub sector: String,
    pub current_price: f64,
    pub base_target_price: f64,
    pub supercycle_target_price: f64,
    pub potential_upside_pct: f64,
    pub catalyst_headline: String,
    pub synthesis: SixVectorSynthesis,
    pub conviction_score: f64,
    pub statutory_disclaimer: String,
}

/// Central Real-Time Alert Distribution Hub
pub struct AlertDistributionHub {
    tx: broadcast::Sender<FlashAlphaCard>,
    recent_cards_cache: Arc<RwLock<Vec<FlashAlphaCard>>>,
    max_cached_cards: usize,
}

impl Default for AlertDistributionHub {
    fn default() -> Self {
        Self::new(1000, 100)
    }
}

impl AlertDistributionHub {
    pub fn new(channel_capacity: usize, max_cached_cards: usize) -> Self {
        let (tx, _) = broadcast::channel(channel_capacity);
        Self {
            tx,
            recent_cards_cache: Arc::new(RwLock::new(Vec::with_capacity(max_cached_cards))),
            max_cached_cards,
        }
    }

    /// Subscribes to the live stream of Flash Alpha Cards
    pub fn subscribe(&self) -> broadcast::Receiver<FlashAlphaCard> {
        self.tx.subscribe()
    }

    /// Broadcasts an Alpha Card to all connected listeners in < 1 millisecond
    pub fn broadcast(&self, card: FlashAlphaCard) -> Result<usize, broadcast::error::SendError<FlashAlphaCard>> {
        // Update circular cache for new client reconnections
        if let Ok(mut cache) = self.recent_cards_cache.write() {
            if cache.len() >= self.max_cached_cards {
                cache.remove(0);
            }
            cache.push(card.clone());
        }

        self.tx.send(card)
    }

    /// Fetches all recently broadcasted cards (for instant dashboard load)
    pub fn get_recent_cards(&self) -> Vec<FlashAlphaCard> {
        self.recent_cards_cache
            .read()
            .map(|cache| cache.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_hub_broadcast_and_cache() {
        let hub = AlertDistributionHub::new(100, 10);
        let mut rx = hub.subscribe();

        let card = FlashAlphaCard {
            alert_id: "ALPHA_20261018_01".to_string(),
            timestamp_epoch_ms: 1729245000000,
            symbol: "KAYNES".to_string(),
            company_name: "Kaynes Technology India Ltd".to_string(),
            sector: "EMS / Semiconductors".to_string(),
            current_price: 5200.0,
            base_target_price: 7800.0,
            supercycle_target_price: 11500.0,
            potential_upside_pct: 121.1,
            catalyst_headline: "Cabinet clears ₹2,850 Cr Semiconductor OSAT Facility + ₹1,450 Cr Fresh Tier-1 Order Win".to_string(),
            synthesis: SixVectorSynthesis {
                policy_vector: "25% SPECS 2.0 Capex Subsidy approved by MeitY".to_string(),
                tech_moat_vector: "Proprietary high-density interconnect substrates with sub-10nm accuracy".to_string(),
                s_curve_market_share_vector: "Domestic EMS market share growing from 3.2% towards 7.5%".to_string(),
                management_pedigree_vector: "Founder-led tier-1 pedigree with 0.0% promoter pledge".to_string(),
                ground_adoption_vector: "Zero field returns across automotive and aerospace customer deployments".to_string(),
                institutional_blind_spot_vector: "Forward EPS expanding at 48% CAGR while valued at historical manufacturing PE".to_string(),
            },
            conviction_score: 94.5,
            statutory_disclaimer: "RESEARCH & QUANTITATIVE SCENARIO SIMULATION ONLY: Educational simulation under SEBI Research Analysts Regulations.".to_string(),
        };

        let listeners_count = hub.broadcast(card.clone()).unwrap();
        assert_eq!(listeners_count, 1);

        let received = rx.try_recv().expect("Expected broadcasted card");
        assert_eq!(received.symbol, "KAYNES");
        assert_eq!(received.conviction_score, 94.5);

        let recent = hub.get_recent_cards();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].alert_id, "ALPHA_20261018_01");
    }
}
