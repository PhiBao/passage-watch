//! Passage Watch HTTP service. One origin serves the product API; the React
//! bundle is served separately by Vite in dev and by any static host in prod.

use std::sync::Arc;

use chrono::{Datelike, Timelike};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use passage_watch::cmc::{CmcClient, ENDPOINTS};
use passage_watch::config;
use passage_watch::date::SimpleDate;
use passage_watch::engine::{self, rule_version};
use passage_watch::model::{Destination, Instrument, MarketSession, Onboarding, PlanContext};
use passage_watch::route::RouteResult;
use passage_watch::rpc::SolanaRpc;
use passage_watch::unit::active_multiplier;

#[derive(Clone)]
struct AppState {
    cmc: Arc<CmcClient>,
    rpc: Arc<SolanaRpc>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = config::load().expect("configuration error");
    let port = config.port;
    let state = AppState {
        cmc: Arc::new(CmcClient::new(&config)),
        rpc: Arc::new(SolanaRpc::default()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/status", get(status))
        .route("/api/v1/catalog", get(catalog))
        .route("/api/v1/instruments/:id", get(instrument))
        .route("/api/v1/check", get(check))
        .route("/api/v1/method", get(method))
        .route("/api/v1/observations/:id", get(observations))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(%addr, "Passage Watch listening");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

async fn health() -> &'static str {
    "ok"
}

async fn status(State(s): State<AppState>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match s.cmc.key_info().await {
        Ok(info) => Ok(Json(
            json!({ "cmc": "connected", "plan": info.get("data").and_then(|d| d.get("plan")) }),
        )),
        Err(e) => Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

async fn catalog(State(s): State<AppState>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match s.cmc.catalog().await {
        Ok(instruments) => Ok(Json(json!({ "instruments": instruments }))),
        Err(e) => Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

async fn instrument(
    State(s): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Instrument>, (StatusCode, Json<Value>)> {
    let catalog = s.cmc.catalog().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": e.to_string() })),
        )
    })?;
    catalog
        .into_iter()
        .find(|i| i.crypto_id == id)
        .map(Json)
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "instrument not found" })),
        ))
}

#[derive(Debug, Deserialize)]
struct CheckQuery {
    crypto_id: u64,
    quantity: f64,
    destination: Destination,
    #[serde(default)]
    deadline: String,
    #[serde(default)]
    issuer_onboarding: Option<Onboarding>,
    #[serde(default)]
    broker_onboarding: Option<Onboarding>,
}

async fn check(
    State(s): State<AppState>,
    Query(q): Query<CheckQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut catalog = s.cmc.catalog().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": e.to_string() })),
        )
    })?;
    let instrument = catalog
        .iter_mut()
        .find(|i| i.crypto_id == q.crypto_id)
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "instrument not found" })),
        ))?;

    // Resolve the Solana contract for identity display; unknown if unavailable.
    if let Ok(Some(contract)) = s.cmc.contract_for(q.crypto_id).await {
        instrument.contract = contract;
    }

    // Resolve the active multiplier from the parsed mint. If it cannot be
    // resolved, unit_basis stays UNKNOWN and we do not present share quantities.
    let now_secs = chrono::Utc::now().timestamp();
    let mut active_unit: Option<passage_watch::unit::ScaledUiConfig> = None;
    if !instrument.contract.is_empty() {
        if let Ok(cfg) = s.rpc.scaled_ui_config(&instrument.contract).await {
            if active_multiplier(&cfg, now_secs).is_some() {
                instrument.unit_basis = passage_watch::model::UnitBasis::Verified;
                active_unit = Some(cfg);
            }
        }
    }

    // Read-only secondary quote (Jupiter V2, no taker -> quote only, keyless).
    let mut quote = fetch_secondary_quote(&instrument.token_symbol, q.quantity).await;
    // Normalize raw base units into displayed share-equivalents when the unit
    // basis is verified: raw / 10^decimals / active multiplier.
    if let (Some(q), Some(unit)) = (quote.as_mut(), active_unit.as_ref()) {
        if let Some(raw) = q.output_amount {
            if let Some(m) = active_multiplier(unit, now_secs) {
                // Solar Scaled UI Amount rule: displayed = raw × multiplier / 10^decimals.
                let displayed = raw * m / 10_f64.powi(unit.decimals as i32);
                q.output_amount = Some(displayed);
                q.output_label = "share-equivalents".into();
            }
        }
    }

    let now = chrono_like_now();
    let session = MarketSession::from_utc_hour(now.hour);
    let today = SimpleDate::new(now.year, now.month, now.day);

    let ctx = PlanContext {
        instrument: instrument.clone(),
        quantity: q.quantity,
        destination: q.destination,
        deadline: q.deadline.clone(),
        issuer_onboarding: q.issuer_onboarding.unwrap_or_default(),
        broker_onboarding: q.broker_onboarding.unwrap_or_default(),
        market_session: session,
        has_secondary_quote: quote
            .as_ref()
            .map(|x| x.output_amount.is_some())
            .unwrap_or(false),
        quote_output_amount: quote.as_ref().and_then(|x| x.output_amount),
        quote_expired: false,
        evidence_reviewed_at: "2026-11-18".into(),
    };

    let routes: Vec<RouteResult> = engine::evaluate(&ctx, today);
    let observed = instrument.observed_price_usd.map(|p| p * q.quantity);

    Ok(Json(json!({
        "instrument": instrument,
        "destination": q.destination,
        "quantity": q.quantity,
        "observed_value_usd": observed,
        "quote": quote,
        "routes": routes,
        "checked_at": now.iso,
        "rule_version": rule_version(),
    })))
}

async fn observations(
    State(s): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let quotes = s.cmc.historical_prices(id).await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": e.to_string() })),
        )
    })?;
    let points: Vec<Value> = quotes
        .iter()
        .map(|q| {
            let ts = q.get("timestamp").cloned().unwrap_or(Value::Null);
            let price = q
                .get("quote")
                .and_then(|x| x.get("USD"))
                .and_then(|x| x.get("price"))
                .cloned()
                .unwrap_or(Value::Null);
            json!({ "timestamp": ts, "price_usd": price })
        })
        .collect();
    Ok(Json(json!({ "crypto_id": id, "points": points })))
}

async fn method() -> Json<Value> {
    Json(json!({
        "rule_version": rule_version(),
        "endpoints": ENDPOINTS,
        "note": "CMC supplies identity and observed prices. Route terms are separately sourced and versioned."
    }))
}

// ---- secondary quote adapter ------------------------------------------------

#[derive(Debug, Serialize)]
struct QuoteObservation {
    provider: &'static str,
    input_amount_usd: f64,
    output_amount: Option<f64>,
    output_label: String,
    retrieved_at: String,
    expires_at: Option<String>,
    is_final: bool,
    error: Option<String>,
}

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

async fn fetch_secondary_quote(token_symbol: &str, quantity: f64) -> Option<QuoteObservation> {
    let mint = match token_symbol {
        "AAPLX" => "XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp",
        "NVDAX" => "Xsc9qvGR1efVDFGLrVsmkzv3qi45LTBjeUKSPmx9qEh",
        "TSLAX" => "XsDoVfqeBukxuZHWhdvWHBhgEHjGNst4MLodqsJHzoB",
        "SPYX" => "XsoCS1TfEyfFhfvj8EtZ528L3CaKBDBRqRapnBbDF2W",
        _ => return None,
    };

    // USDC has 6 decimals; quantity is displayed share-equivalents, so a
    // notional approximation uses a reference price. This is deliberately a
    // display estimate; the authoritative quote is what Jupiter returns.
    let notional_usd = match token_symbol {
        "AAPLX" => 350.0,
        "NVDAX" => 160.0,
        "TSLAX" => 384.0,
        "SPYX" => 739.0,
        _ => 1.0,
    } * quantity;
    let amount = ((notional_usd * 1_000_000.0).floor() as u64).to_string();

    let url = format!(
        "https://api.jup.ag/swap/v2/order?inputMint={}&outputMint={}&amount={}&slippageBps=50",
        USDC_MINT, mint, amount
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .ok()?;
    let res = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await;
    let retrieved_at = now_iso();

    let observation = match res {
        Ok(r) if r.status().is_success() => match r.json::<Value>().await {
            Ok(v) => {
                let out = v
                    .get("outAmount")
                    .and_then(|x| x.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let err = v.get("error").and_then(|x| x.as_str()).map(String::from);
                QuoteObservation {
                    provider: "Jupiter Swap V2 (read-only)",
                    input_amount_usd: notional_usd,
                    output_amount: out,
                    output_label: "raw token units".into(),
                    retrieved_at,
                    expires_at: v.get("expireAt").and_then(|x| x.as_str()).map(String::from),
                    is_final: false,
                    error: err,
                }
            }
            Err(e) => QuoteObservation {
                provider: "Jupiter Swap V2 (read-only)",
                input_amount_usd: notional_usd,
                output_amount: None,
                output_label: "raw token units".into(),
                retrieved_at,
                expires_at: None,
                is_final: false,
                error: Some(format!("invalid response: {e}")),
            },
        },
        Ok(r) => QuoteObservation {
            provider: "Jupiter Swap V2 (read-only)",
            input_amount_usd: notional_usd,
            output_amount: None,
            output_label: "raw token units".into(),
            retrieved_at,
            expires_at: None,
            is_final: false,
            error: Some(format!("HTTP {}", r.status())),
        },
        Err(e) => QuoteObservation {
            provider: "Jupiter Swap V2 (read-only)",
            input_amount_usd: notional_usd,
            output_amount: None,
            output_label: "raw token units".into(),
            retrieved_at,
            expires_at: None,
            is_final: false,
            error: Some(e.to_string()),
        },
    };
    Some(observation)
}

// Minimal UTC timestamp without pulling in a date-time crate for formatting.
struct Now {
    iso: String,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
}

fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn chrono_like_now() -> Now {
    let now = chrono::Utc::now();
    Now {
        iso: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        year: now.year(),
        month: now.month(),
        day: now.day(),
        hour: now.hour(),
    }
}
