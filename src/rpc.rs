//! Read-only Solana RPC. We use `getAccountInfo` with `jsonParsed`, which
//! returns clean token-2022 extension state (including scaled UI multiplier)
//! without any hand-rolled binary parsing. No transactions are ever built.

use std::time::Duration;

use reqwest::Client;
use serde_json::{json, Value};

use crate::unit::ScaledUiConfig;

pub const SOLANA_RPC: &str = "https://api.mainnet-beta.solana.com";

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("rpc request failed: {0}")]
    Request(String),
    #[error("unexpected rpc response")]
    Unexpected,
}

#[derive(Debug, Clone)]
pub struct SolanaRpc {
    http: Client,
}

impl Default for SolanaRpc {
    fn default() -> Self {
        Self {
            http: Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .expect("http client"),
        }
    }
}

impl SolanaRpc {
    pub async fn scaled_ui_config(&self, mint: &str) -> Result<ScaledUiConfig, RpcError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [mint, { "encoding": "jsonParsed" }]
        });
        let res = self
            .http
            .post(SOLANA_RPC)
            .json(&body)
            .send()
            .await
            .map_err(|e| RpcError::Request(e.to_string()))?;
        let value: Value = res
            .json()
            .await
            .map_err(|e| RpcError::Request(e.to_string()))?;

        let extensions = value
            .get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.get("data"))
            .and_then(|d| d.get("parsed"))
            .and_then(|p| p.get("info"))
            .and_then(|i| i.get("extensions"))
            .and_then(|e| e.as_array())
            .ok_or(RpcError::Unexpected)?;

        for ext in extensions {
            if ext.get("extension").and_then(|x| x.as_str()) == Some("scaledUiAmountConfig") {
                let state = ext.get("state").ok_or(RpcError::Unexpected)?;
                let multiplier = state
                    .get("multiplier")
                    .and_then(|x| x.as_str())
                    .and_then(|s| s.parse().ok());
                let new_multiplier = state
                    .get("newMultiplier")
                    .and_then(|x| x.as_str())
                    .and_then(|s| s.parse().ok());
                let eff = state
                    .get("newMultiplierEffectiveTimestamp")
                    .and_then(|x| x.as_i64())
                    .unwrap_or(0);
                if let (Some(m), Some(nm)) = (multiplier, new_multiplier) {
                    return Ok(ScaledUiConfig {
                        multiplier: m,
                        new_multiplier: nm,
                        new_multiplier_effective_timestamp: eff,
                    });
                }
            }
        }
        // No scaled UI config: a plain token with an implicit multiplier of 1.
        Ok(ScaledUiConfig {
            multiplier: 1.0,
            new_multiplier: 1.0,
            new_multiplier_effective_timestamp: 0,
        })
    }

    pub async fn block_time(&self) -> Result<i64, RpcError> {
        let slot_body = json!({"jsonrpc":"2.0","id":1,"method":"getSlot","params":[]});
        let slot: Value = self
            .http
            .post(SOLANA_RPC)
            .json(&slot_body)
            .send()
            .await
            .map_err(|e| RpcError::Request(e.to_string()))?
            .json()
            .await
            .map_err(|e| RpcError::Request(e.to_string()))?;
        let slot_num = slot
            .get("result")
            .and_then(|x| x.as_u64())
            .ok_or(RpcError::Unexpected)?;
        let body = json!({"jsonrpc":"2.0","id":2,"method":"getBlockTime","params":[slot_num]});
        let time: Value = self
            .http
            .post(SOLANA_RPC)
            .json(&body)
            .send()
            .await
            .map_err(|e| RpcError::Request(e.to_string()))?
            .json()
            .await
            .map_err(|e| RpcError::Request(e.to_string()))?;
        time.get("result")
            .and_then(|x| x.as_i64())
            .ok_or(RpcError::Unexpected)
    }
}
