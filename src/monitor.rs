//! Exit-readiness monitoring.
//!
//! A background worker snapshots each live instrument: its CMC observed price
//! and a standard-size Jupiter sell quote. Changes are exposed to the UI so a
//! saved plan can be diffed against real collected history ("change since
//! saved"). Everything here is read-only.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;

use crate::cache::{get_or_load, Cache};
use crate::cmc::CmcClient;
use crate::model::UnitBasis;
use crate::rpc::SolanaRpc;
use crate::sell_quote::{sell_quote_usdc, SellInput};

/// The standard size we snapshot for monitoring (display share-equivalents).
pub const MONITOR_SIZE: f64 = 50.0;

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub at_iso: String,
    pub price_usd: Option<f64>,
    pub usdc_at_monitor_size: Option<f64>,
    pub quote_error: Option<String>,
}

pub struct MonitorStore {
    /// Recent ring of snapshots for change-since-saved diffs.
    pub logs: Mutex<Vec<(String, Snapshot)>>, // (instrument token, snapshot)
}

impl MonitorStore {
    pub fn new() -> Self {
        Self { logs: Mutex::new(Vec::new()) }
    }

    pub fn push(&self, token_symbol: &str, snap: Snapshot) {
        let mut ring = self.logs.lock().unwrap();
        ring.push((token_symbol.to_string(), snap));
        if ring.len() > 300 {
            let excess = ring.len() - 300;
            ring.drain(0..excess);
        }
    }

    /// Returns snapshots for `token_symbol` after `since_iso` (all if empty), newest first.
    pub fn changes_for(&self, token_symbol: &str, since_iso: Option<&str>) -> Vec<Snapshot> {
        let snaps = self.logs.lock().unwrap();
        let since = since_iso.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());
        snaps
            .iter()
            .rev()
            .filter(|(t, s)| t == token_symbol && since.map_or(true, |st| s.at_iso > st.to_rfc3339()))
            .map(|(_, s)| s.clone())
            .collect()
    }
}

pub async fn snapshot_instrument(
    cmc: &CmcClient,
    http: &reqwest::Client,
    rpc: &SolanaRpc,
    cache: &Arc<Cache>,
    instrument: &crate::model::Instrument,
) -> Snapshot {
    let now_iso = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut instrument = instrument.clone();
    if instrument.contract.is_empty() {
        let key = format!("contract:{}", instrument.crypto_id);
        let c = get_or_load(cache, &key, || {
            let cmc = cmc.clone();
            let id = instrument.crypto_id;
            async move { cmc.contract_for(id).await.map_err(|e| e.to_string()).map(|c| c.unwrap_or_default()) }
        })
        .await;
        instrument.contract = c.unwrap_or_default();
    }

    let mut usdc = None;
    if !instrument.contract.is_empty() {
        let mint_key = format!("mint:{}", instrument.contract);
        let cfg = get_or_load(cache, &mint_key, || {
            let rpc = rpc.clone();
            let m = instrument.contract.clone();
            async move {
                rpc.scaled_ui_config(&m)
                    .await
                    .map(|c| serde_json::to_string(&c).expect("serialize"))
                    .map_err(|e| e.to_string())
            }
        })
        .await;
        if let Ok(json) = cfg {
            if let Ok(unit) = serde_json::from_str::<crate::unit::ScaledUiConfig>(&json) {
                instrument.unit_basis = UnitBasis::Verified;
                let sq = sell_quote_usdc(
                    http,
                    SellInput {
                        token_symbol: &instrument.token_symbol,
                        token_mint: &instrument.contract,
                        quantity_displayed: MONITOR_SIZE,
                        unit,
                        cmc_observed_price_per_share: instrument.observed_price_usd,
                    },
                    &now_iso,
                )
                .await;
                usdc = sq.usdc_amount;
            }
        }
    }

    Snapshot {
        at_iso: now_iso,
        price_usd: instrument.observed_price_usd,
        usdc_at_monitor_size: usdc,
        quote_error: None,
    }
}

pub async fn monitor_worker(
    cmc: Arc<CmcClient>,
    http: reqwest::Client,
    rpc: Arc<SolanaRpc>,
    cache: Arc<Cache>,
    store: Arc<MonitorStore>,
) {
    const SNAPSHOT_LIMIT: usize = 30; // Representative subset; keep upstream call volume sane.
    loop {
        if let Ok(cat) = cmc.catalog().await {
            // Prefer instruments whose price we already observe, plus the first few.
            for instr in cat.iter().take(SNAPSHOT_LIMIT) {
                let snap = snapshot_instrument(&cmc, &http, &rpc, &cache, instr).await;
                store.push(&instr.token_symbol, snap);
            }
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
