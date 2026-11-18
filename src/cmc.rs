//! Thin, allow-listed CoinMarketCap client. Fixed host, key only from config,
//! no user-supplied upstream URLs, no key logging.

use std::time::Duration;

use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::model::{Instrument, UnitBasis};

pub const CMC_HOST: &str = "https://pro-api.coinmarketcap.com";

pub const ENDPOINTS: &[&str] = &[
    "/v1/key/info",
    "/v5/real-world-assets/map",
    "/v5/real-world-assets/info",
    "/v5/real-world-assets/quotes/latest",
    "/v5/real-world-assets/issuers/list",
    "/v5/real-world-assets/issuers",
    "/v2/cryptocurrency/info",
    "/v3/cryptocurrency/quotes/latest",
    "/v3/cryptocurrency/quotes/historical",
];

#[derive(Debug, Clone, Deserialize)]
pub struct CmcStatus {
    #[serde(default)]
    pub error_code: serde_json::Value,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub credit_count: Option<i64>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    status: CmcStatus,
    data: Option<T>,
}

#[derive(Debug, thiserror::Error)]
pub enum CmcError {
    #[error("endpoint not allow-listed")]
    EndpointNotAllowed,
    #[error("CMC returned an error: {0:?} {1:?}")]
    Api(serde_json::Value, Option<String>),
    #[error("request failed: {0}")]
    Request(String),
    #[error("missing data in CMC response")]
    MissingData,
}

#[derive(Clone)]
pub struct CmcClient {
    http: Client,
    key: String,
}

impl CmcClient {
    pub fn new(config: &Config) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(12))
            .build()
            .expect("build http client");
        Self {
            http,
            key: config.cmc_api_key.clone(),
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<(CmcStatus, T), CmcError> {
        if !ENDPOINTS.contains(&path) {
            return Err(CmcError::EndpointNotAllowed);
        }
        let url = format!("{}{}", CMC_HOST, path);
        let res = self
            .http
            .get(url)
            .header("X-CMC_PRO_API_KEY", &self.key)
            .header("Accept", "application/json")
            .query(params)
            .send()
            .await
            .map_err(|e| CmcError::Request(e.to_string()))?;

        let status_code = res.status();
        let body = res
            .text()
            .await
            .map_err(|e| CmcError::Request(e.to_string()))?;
        let env: Envelope<T> =
            serde_json::from_str(&body).map_err(|e| CmcError::Request(format!("bad JSON: {e}")))?;

        let is_error = match &env.status.error_code {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
            serde_json::Value::String(s) => s != "0" && !s.is_empty(),
            _ => false,
        } || !status_code.is_success();

        if is_error {
            return Err(CmcError::Api(
                env.status.error_code.clone(),
                env.status.error_message.clone(),
            ));
        }
        match env.data {
            Some(data) => Ok((env.status, data)),
            None => Err(CmcError::MissingData),
        }
    }

    pub async fn key_info(&self) -> Result<serde_json::Value, CmcError> {
        let (_, data) = self.get::<serde_json::Value>("/v1/key/info", &[]).await?;
        Ok(data)
    }

    pub async fn catalog(&self) -> Result<Vec<Instrument>, CmcError> {
        // Underlying symbols that back our four supported xStocks wrappers.
        let symbols = ["AAPL", "NVDA", "TSLA", "SPY"].join(",");
        let (_, map): (_, RwaMap) = self
            .get(
                "/v5/real-world-assets/map",
                &[("symbol", symbols), ("limit", "20".into())],
            )
            .await?;

        let rwa_ids = map
            .rwa_assets
            .iter()
            .filter(|a| a.has_tokens)
            .map(|a| a.rwa_id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let (_, quotes): (_, RwaQuotes) = self
            .get(
                "/v5/real-world-assets/quotes/latest",
                &[("rwa_id", rwa_ids)],
            )
            .await?;

        let mut instruments = Vec::new();
        for asset in quotes.rwa_assets {
            for token in asset.tokens {
                if !is_supported_wrapper(&token.symbol) {
                    continue;
                }
                let price = token.price;
                instruments.push(Instrument {
                    underlying: asset.symbol.clone(),
                    underlying_name: asset.name.clone(),
                    rwa_id: asset.rwa_id,
                    token_symbol: token.symbol.clone(),
                    crypto_id: token.crypto_id,
                    issuer: token.issuer_name.clone().unwrap_or_default(),
                    network: "Solana".into(),
                    contract: String::new(),
                    observed_price_usd: price,
                    unit_basis: UnitBasis::Unknown,
                });
            }
        }
        // Stable, predictable ordering for judge reproducibility.
        instruments.sort_by(|a, b| a.token_symbol.cmp(&b.token_symbol));
        Ok(instruments)
    }

    pub async fn contract_for(&self, crypto_id: u64) -> Result<Option<String>, CmcError> {
        let (_, data): (_, serde_json::Value) = self
            .get("/v2/cryptocurrency/info", &[("id", crypto_id.to_string())])
            .await?;
        let entry = data
            .get(&crypto_id.to_string())
            .or_else(|| data.as_object().and_then(|m| m.values().next()));
        if let Some(list) = entry
            .and_then(|e| e.get("contract_address"))
            .and_then(|v| v.as_array())
        {
            for item in list {
                let platform = item
                    .get("platform")
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if platform.eq_ignore_ascii_case("solana") {
                    return Ok(item
                        .get("contract_address")
                        .and_then(|v| v.as_str())
                        .map(String::from));
                }
            }
        }
        Ok(None)
    }

    pub async fn historical_prices(
        &self,
        crypto_id: u64,
    ) -> Result<Vec<serde_json::Value>, CmcError> {
        let (_, data): (_, serde_json::Value) = self
            .get(
                "/v3/cryptocurrency/quotes/historical",
                &[
                    ("id", crypto_id.to_string()),
                    ("interval", "1d".into()),
                    ("count", "10".into()),
                ],
            )
            .await?;
        let key = crypto_id.to_string();
        let quotes = data
            .get(&key)
            .and_then(|v| v.get("quotes"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(quotes)
    }
}

fn is_supported_wrapper(symbol: &str) -> bool {
    matches!(symbol, "AAPLX" | "NVDAX" | "TSLAX" | "SPYX")
}

#[derive(Debug, Deserialize)]
struct RwaMap {
    #[serde(default)]
    rwa_assets: Vec<RwaAssetLite>,
}

#[derive(Debug, Deserialize)]
struct RwaAssetLite {
    rwa_id: u64,
    #[serde(default)]
    has_tokens: bool,
}

#[derive(Debug, Deserialize)]
struct RwaQuotes {
    #[serde(default)]
    rwa_assets: Vec<RwaAssetQuote>,
}

#[derive(Debug, Deserialize)]
struct RwaAssetQuote {
    rwa_id: u64,
    symbol: String,
    name: String,
    #[serde(default)]
    tokens: Vec<RwaToken>,
}

#[derive(Debug, Deserialize)]
struct RwaToken {
    symbol: String,
    price: Option<f64>,
    crypto_id: u64,
    #[serde(default)]
    issuer_name: Option<String>,
}
