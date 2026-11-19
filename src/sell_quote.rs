//! Read-only sell-side Jupiter quote: how much USDC does selling N tokens give.
//!
//! Direction is *correct*: input = the token you hold, output = USDC.
//! Amount = user quantity (displayed share-equivalents) converted to raw base
//! units with the active Token-2022 multiplier. The quoted USDC is an
//! **estimate, not a guaranteed fill.**
//!
//! Security: Jupiter's `/order` without `taker` returns a quote, no transaction.
//! We never pass a taker or build/execute anything here.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::unit::{active_multiplier, displayed_to_base, ScaledUiConfig};

pub const JUPITER_V2_ORDER: &str = "https://api.jup.ag/swap/v2/order";
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[derive(Debug, Clone, Serialize)]
pub struct SellQuote {
    pub token_symbol: String,
    pub quantity_displayed: f64,
    pub raw_token_in: u128,
    pub usdc_amount: Option<f64>,
    pub output_label: &'static str,
    pub spread_vs_cmc_bps: Option<f64>,
    pub usdc_per_share_estimate: Option<f64>,
    pub retrieved_at: String,
    pub error: Option<String>,
    pub is_ready_to_confirm: bool,
}

#[derive(Debug, Deserialize)]
struct JupiterOrder {
    #[serde(default, rename = "outAmount")]
    out_amount: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

pub struct SellInput<'a> {
    pub token_symbol: &'a str,
    pub token_mint: &'a str,
    pub quantity_displayed: f64,
    pub unit: ScaledUiConfig,
    pub cmc_observed_price_per_share: Option<f64>,
}

fn quote_with_err(
    token_symbol: &str,
    quantity_displayed: f64,
    raw_token_in: u128,
    error: Option<String>,
    now_iso: &str,
) -> SellQuote {
    SellQuote {
        token_symbol: token_symbol.to_string(),
        quantity_displayed,
        raw_token_in,
        usdc_amount: None,
        output_label: if error.is_some() {
            "no quote from this provider"
        } else {
            "usdc dollars"
        },
        spread_vs_cmc_bps: None,
        usdc_per_share_estimate: None,
        retrieved_at: now_iso.to_string(),
        error,
        is_ready_to_confirm: false,
    }
}

pub async fn sell_quote_usdc(client: &Client, input: SellInput<'_>, now_iso: &str) -> SellQuote {
    let SellInput {
        token_symbol,
        token_mint,
        quantity_displayed,
        unit,
        cmc_observed_price_per_share,
    } = input;

    let active = active_multiplier(&unit, 0);
    let Some(m) = active else {
        return quote_with_err(
            token_symbol,
            quantity_displayed,
            0,
            Some("unit basis is not verified; sell amount suppressed".into()),
            now_iso,
        );
    };

    let raw_token_in = match displayed_to_base(quantity_displayed, unit.decimals, m) {
        Some(v) if v > 0 => v,
        _ => {
            return quote_with_err(
                token_symbol,
                quantity_displayed,
                0,
                Some("invalid quantity".into()),
                now_iso,
            )
        }
    };

    let url = format!(
        "{}?inputMint={}&outputMint={}&amount={}&slippageBps=50",
        JUPITER_V2_ORDER, token_mint, USDC_MINT, raw_token_in
    );

    let res = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await;
    let json: Result<JupiterOrder, String> = match res {
        Ok(r) if r.status().is_success() => r.json().await.map_err(|e| e.to_string()),
        Ok(r) => Err(format!("HTTP {}", r.status())),
        Err(e) => Err(format!("request failed: {e}")),
    };

    match json {
        Err(e) => quote_with_err(
            token_symbol,
            quantity_displayed,
            raw_token_in,
            Some(e),
            now_iso,
        ),
        Ok(order) => match order.error {
            Some(e) => quote_with_err(
                token_symbol,
                quantity_displayed,
                raw_token_in,
                Some(e),
                now_iso,
            ),
            None => match order.out_amount {
                Some(out) if out != "0" => {
                    let raw_usdc: u128 = out.parse().unwrap_or(0);
                    let usdc = raw_usdc as f64 / 1e6;
                    let spread_vs_cmc_bps = cmc_observed_price_per_share.and_then(|obs| {
                        if quantity_displayed <= 0.0 || obs <= 0.0 {
                            return None;
                        }
                        Some((usdc / (obs * quantity_displayed) - 1.0) * 10_000.0)
                    });
                    let usdc_per_share_estimate = if quantity_displayed > 0.0 {
                        Some(usdc / quantity_displayed)
                    } else {
                        None
                    };
                    SellQuote {
                        token_symbol: token_symbol.to_string(),
                        quantity_displayed,
                        raw_token_in,
                        usdc_amount: Some(usdc),
                        output_label: "usdc dollars",
                        spread_vs_cmc_bps,
                        usdc_per_share_estimate,
                        retrieved_at: now_iso.to_string(),
                        error: None,
                        // Never auto-guarantee the fill; a cached/present quote is
                        // an estimate until a confirmation flow is added.
                        is_ready_to_confirm: false,
                    }
                }
                _ => quote_with_err(
                    token_symbol,
                    quantity_displayed,
                    raw_token_in,
                    Some("empty quote response".into()),
                    now_iso,
                ),
            },
        },
    }
}
