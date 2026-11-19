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

        let info = value
            .get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.get("data"))
            .and_then(|d| d.get("parsed"))
            .and_then(|p| p.get("info"))
            .ok_or(RpcError::Unexpected)?;

        let decimals = info
            .get("decimals")
            .and_then(|d| d.as_u64())
            .and_then(|d| u8::try_from(d).ok())
            .unwrap_or(0);

        let extensions = info
            .get("extensions")
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
                        decimals,
                    });
                }
            }
        }
        // No scaled UI config: a plain token with an implicit multiplier of 1.
        Ok(ScaledUiConfig {
            multiplier: 1.0,
            new_multiplier: 1.0,
            new_multiplier_effective_timestamp: 0,
            decimals,
        })
    }

    /// Returns { token-mint -> uiAmount } that `wallet` holds. Read-only.
    /// Uses the Token-2022 program (all Backed tokens live on it) in a single
    /// `getTokenAccountsByOwner` call, then a parsed `getAccountInfo` per account.
    pub async fn wallet_holdings(&self, wallet: &str) -> Result<std::collections::HashMap<String, f64>, RpcError> {
        const TOKEN_PROGRAM_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

        let res = self
            .http
            .post(SOLANA_RPC)
            .json(&json!({"jsonrpc":"2.0","id":1,"method":"getTokenAccountsByOwner","params":[wallet, [TOKEN_PROGRAM_2022]]}))
            .send()
            .await
            .map_err(|e| RpcError::Request(e.to_string()))?;
        let v: serde_json::Value = res.json().await.map_err(|e| RpcError::Request(e.to_string()))?;
        let accounts = v.get("result").and_then(|r| r.get("value")).and_then(|x| x.as_array()).cloned().unwrap_or_default();

        let mut out: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for acct in accounts {
            let pubkey = acct.get(0).and_then(|x| x.as_str()).unwrap_or_default().to_string();
            if pubkey.is_empty() {
                continue;
            }
            let res2 = self
                .http
                .post(SOLANA_RPC)
                .json(&json!({"jsonrpc":"2.0","id":2,"method":"getAccountInfo","params":[pubkey,{"encoding":"jsonParsed"}]}))
                .send()
                .await
                .map_err(|e| RpcError::Request(e.to_string()))?;
            let v2: serde_json::Value = res2.json().await.map_err(|e| RpcError::Request(e.to_string()))?;
            let info = v2
                .get("result").and_then(|r| r.get("value")).and_then(|x| x.get("data")).and_then(|d| d.get("parsed")).and_then(|p| p.get("info")).cloned();
            let Some(info) = info else { continue };
            let mint = info.get("mint").and_then(|m| m.as_str()).unwrap_or_default().to_string();
            let amount = info
                .get("tokenAmount")
                .and_then(|t| t.get("amount"))
                .and_then(|s| s.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let decimals = info.get("tokenAmount").and_then(|t| t.get("decimals")).and_then(|d| d.as_u64()).unwrap_or(0);
            let ui = info
                .get("tokenAmount")
                .and_then(|t| t.get("uiAmountString"))
                .and_then(|s| s.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| info.get("tokenAmount").and_then(|t| t.get("uiAmount")).and_then(|s| s.as_f64()))
                .unwrap_or(amount / 10_f64.powi(decimals as i32));
            if !mint.is_empty() && ui > 0.0 {
                out.insert(mint, ui);
            }
        }
        Ok(out)
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
