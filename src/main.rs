//! Passage Watch HTTP service. One origin serves the product API; the React
//! bundle is served separately by Vite in dev and by any static host in prod.

use std::sync::Arc;

use chrono::{Datelike, Timelike};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use passage_watch::cmc::{CmcClient, ENDPOINTS};
use passage_watch::config;
use passage_watch::date::SimpleDate;
use passage_watch::engine::{self, rule_version};
use passage_watch::model::{Destination, Instrument, MarketSession, Onboarding, PlanContext};
use passage_watch::route::RouteResult;
use passage_watch::rpc::SolanaRpc;
use passage_watch::sell_quote::{sell_quote_usdc, SellInput};
use passage_watch::unit::active_multiplier;

use std::time::Duration;

#[derive(Clone)]
struct AppState {
    cmc: Arc<CmcClient>,
    rpc: Arc<SolanaRpc>,
    http: reqwest::Client,
    cache: Arc<passage_watch::cache::Cache>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = config::load().expect("configuration error");
    let port = config.port;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("http client");
    let state = AppState {
        cmc: Arc::new(CmcClient::new(&config)),
        rpc: Arc::new(SolanaRpc::default()),
        http,
        cache: Arc::new(passage_watch::cache::Cache::new(Duration::from_secs(45))),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/status", get(status))
        .route("/api/v1/catalog", get(catalog))
        .route("/api/v1/instruments/:id", get(instrument))
        .route("/api/v1/check", get(check))
        .route("/api/v1/method", get(method))
        .route("/api/v1/observations/:id", get(observations))
        .fallback(static_files)
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(%addr, "Passage Watch listening");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

/// Serves the built frontend from `dist/` (produced by `pnpm build`). In dev,
/// Vite serves the frontend instead and proxies `/api` to this service.
async fn static_files(req: axum::extract::Request) -> axum::response::Response {
    use axum::body::Body;
    use axum::response::IntoResponse;

    let path = req.uri().path();
    let clean = path.trim_start_matches('/');
    let candidate = if clean.is_empty() {
        "index.html"
    } else {
        clean
    };
    let candidate = format!("dist/{}", candidate);

    match std::fs::read(&candidate) {
        Ok(bytes) => {
            let content_type = if candidate.ends_with(".html") {
                "text/html"
            } else if candidate.ends_with(".js") {
                "application/javascript"
            } else if candidate.ends_with(".css") {
                "text/css"
            } else if candidate.ends_with(".svg") {
                "image/svg+xml"
            } else if candidate.ends_with(".json") {
                "application/json"
            } else {
                "application/octet-stream"
            };
            (
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, content_type)],
                Body::from(bytes),
            )
                .into_response()
        }
        Err(_) => {
            let index = std::fs::read("dist/index.html").unwrap_or_default();
            (
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "text/html")],
                Body::from(index),
            )
                .into_response()
        }
    }
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

    // Resolve the Solana contract (cached). If unavailable, unit stays unknown
    // and sell-side quote amounts are suppressed — we never guess.
    let contract_key = format!("contract:{}", q.crypto_id);
    if instrument.contract.is_empty() {
        let got = passage_watch::cache::get_or_load(&s.cache, &contract_key, || {
            let client = s.cmc.clone();
            async move {
                client
                    .contract_for(q.crypto_id)
                    .await
                    .map_err(|e| e.to_string())
                    .map(|c| c.unwrap_or_default())
            }
        })
        .await;
        if let Ok(c) = got {
            instrument.contract = c;
        }
    }

    // Resolve the active multiplier from the parsed mint via cached RPC.
    let now_secs = chrono::Utc::now().timestamp();
    let mut active_unit: Option<passage_watch::unit::ScaledUiConfig> = None;
    if !instrument.contract.is_empty() {
        let key = format!("mint:{}", instrument.contract);
        let res = passage_watch::cache::get_or_load(&s.cache, &key, || {
            let rpc = s.rpc.clone();
            let mint = instrument.contract.clone();
            async move {
                rpc.scaled_ui_config(&mint)
                    .await
                    .map(|c| serde_json::to_string(&c).expect("serialize config"))
                    .map_err(|e| e.to_string())
            }
        })
        .await;
        if let Ok(json) = res {
            if let Ok(cfg) = serde_json::from_str::<passage_watch::unit::ScaledUiConfig>(&json) {
                if active_multiplier(&cfg, now_secs).is_some() {
                    instrument.unit_basis = passage_watch::model::UnitBasis::Verified;
                    active_unit = Some(cfg);
                }
            }
        }
    }

    // Primary liquidity signal: Sell N tokens -> USDC. Direction and semantics
    // were corrected: input = your holding, output = USDC, not a share quote.
    let mut quote: Option<passage_watch::sell_quote::SellQuote> = None;
    if let Some(unit) = active_unit.as_ref() {
        let sell = sell_quote_usdc(
            &s.http,
            SellInput {
                token_symbol: &instrument.token_symbol,
                token_mint: &instrument.contract,
                quantity_displayed: q.quantity,
                unit: *unit,
                cmc_observed_price_per_share: instrument.observed_price_usd,
            },
            &now_iso(),
        )
        .await;
        quote = Some(sell);
    }

    let now = chrono_like_now();
    let session =
        MarketSession::from_utc_datetime(now.year, now.month, now.day, now.weekday(), now.hour);
    let today = SimpleDate::new(now.year, now.month, now.day);

    let observed = instrument.observed_price_usd.map(|p| p * q.quantity);

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
            .map(|x| x.usdc_amount.is_some())
            .unwrap_or(false),
        quote_output_amount: quote.as_ref().and_then(|x| x.usdc_amount),
        quote_expired: false,
        evidence_reviewed_at: "2026-11-18".into(),
    };

    let routes: Vec<RouteResult> = engine::evaluate(&ctx, today);

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

// Minimal UTC timestamp handling.
struct Now {
    iso: String,
    year: i32,
    month: u32,
    day: u32,
    weekday: u32, // chrono convention: 0 = Sunday, 1 = Monday, ... 6 = Saturday
    hour: u32,
}

impl Now {
    fn weekday(&self) -> u32 {
        self.weekday
    }
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
        weekday: now.weekday().num_days_from_sunday(),
        hour: now.hour(),
    }
}
