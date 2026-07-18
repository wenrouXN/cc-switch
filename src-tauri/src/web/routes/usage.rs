//! Usage statistics routes — delegate to Database methods.

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/summary", get(get_summary))
        .route("/summary-by-app", get(get_summary_by_app))
        .route("/trends", get(get_trends))
        .route("/provider-stats", get(get_provider_stats))
        .route("/model-stats", get(get_model_stats))
        .route("/request-logs", get(get_request_logs))
        .route("/request-detail", get(get_request_detail))
        .route("/model-pricing", get(get_model_pricing))
        .route("/model-pricing", post(update_model_pricing))
        .route("/model-pricing/delete", post(delete_model_pricing))
        .route("/provider-limits", get(check_provider_limits))
        .route("/sync", post(sync_session_usage))
        .route("/data-sources", get(get_data_sources))
}

#[derive(Deserialize)]
struct UsageQuery {
    #[serde(rename = "startDate")]
    start_date: Option<i64>,
    #[serde(rename = "endDate")]
    end_date: Option<i64>,
    #[serde(rename = "appType")]
    app_type: Option<String>,
    #[serde(rename = "providerName")]
    provider_name: Option<String>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct LogsQuery {
    #[serde(rename = "startDate")]
    start_date: Option<i64>,
    #[serde(rename = "endDate")]
    end_date: Option<i64>,
    #[serde(rename = "appType")]
    app_type: Option<String>,
    #[serde(rename = "providerName")]
    provider_name: Option<String>,
    model: Option<String>,
    #[serde(rename = "statusCode")]
    status_code: Option<u16>,
    #[serde(rename = "page", default)]
    page: u32,
    #[serde(rename = "pageSize", default = "default_page_size")]
    page_size: u32,
}

fn default_page_size() -> u32 {
    20
}

#[derive(Deserialize)]
struct RequestDetailQuery {
    #[serde(rename = "requestId")]
    request_id: String,
}

#[derive(Deserialize)]
struct ProviderLimitsQuery {
    #[serde(rename = "providerId")]
    provider_id: String,
    #[serde(rename = "appType")]
    app_type: String,
}

#[derive(Deserialize)]
struct DeletePricingBody {
    #[serde(rename = "modelId")]
    model_id: String,
}

#[derive(Deserialize)]
struct UpdatePricingBody {
    #[serde(rename = "modelId")]
    model_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "inputCost")]
    input_cost: String,
    #[serde(rename = "outputCost")]
    output_cost: String,
    #[serde(rename = "cacheReadCost")]
    cache_read_cost: String,
    #[serde(rename = "cacheCreationCost")]
    cache_creation_cost: String,
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({"success": true, "data": data}))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(serde_json::json!({"success": false, "error": msg.to_string()}))
}

async fn get_summary(
    State((state, _)): State<Shared>,
    Query(q): Query<UsageQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_usage_summary(
        q.start_date,
        q.end_date,
        q.app_type.as_deref(),
        q.provider_name.as_deref(),
        q.model.as_deref(),
    ) {
        Ok(s) => ok(s),
        Err(e) => err(e),
    }
}

async fn get_summary_by_app(
    State((state, _)): State<Shared>,
    Query(q): Query<UsageQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_usage_summary_by_app(
        q.start_date,
        q.end_date,
        q.provider_name.as_deref(),
        q.model.as_deref(),
    ) {
        Ok(s) => ok(s),
        Err(e) => err(e),
    }
}

async fn get_trends(
    State((state, _)): State<Shared>,
    Query(q): Query<UsageQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_daily_trends(
        q.start_date,
        q.end_date,
        q.app_type.as_deref(),
        q.provider_name.as_deref(),
        q.model.as_deref(),
    ) {
        Ok(t) => ok(t),
        Err(e) => err(e),
    }
}

async fn get_provider_stats(
    State((state, _)): State<Shared>,
    Query(q): Query<UsageQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_provider_stats(
        q.start_date,
        q.end_date,
        q.app_type.as_deref(),
        q.provider_name.as_deref(),
        q.model.as_deref(),
    ) {
        Ok(s) => ok(s),
        Err(e) => err(e),
    }
}

async fn get_model_stats(
    State((state, _)): State<Shared>,
    Query(q): Query<UsageQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_model_stats(
        q.start_date,
        q.end_date,
        q.app_type.as_deref(),
        q.provider_name.as_deref(),
        q.model.as_deref(),
    ) {
        Ok(s) => ok(s),
        Err(e) => err(e),
    }
}

async fn get_request_logs(
    State((state, _)): State<Shared>,
    Query(q): Query<LogsQuery>,
) -> Json<serde_json::Value> {
    let filters = crate::services::usage_stats::LogFilters {
        start_date: q.start_date,
        end_date: q.end_date,
        app_type: q.app_type,
        provider_name: q.provider_name,
        model: q.model,
        status_code: q.status_code,
    };
    match state.db.get_request_logs(&filters, q.page, q.page_size) {
        Ok(logs) => ok(logs),
        Err(e) => err(e),
    }
}

async fn get_request_detail(
    State((state, _)): State<Shared>,
    Query(q): Query<RequestDetailQuery>,
) -> Json<serde_json::Value> {
    match state.db.get_request_detail(&q.request_id) {
        Ok(detail) => ok(detail),
        Err(e) => err(e),
    }
}

async fn get_model_pricing(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    if let Err(e) = state.db.ensure_model_pricing_seeded() {
        return err(e);
    }
    let db = state.db.clone();
    let conn = match db.conn.lock() {
        Ok(c) => c,
        Err(e) => return err(format!("Mutex lock failed: {e}")),
    };
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='model_pricing'",
            [],
            |row| row.get::<_, i64>(0).map(|count| count > 0),
        )
        .unwrap_or(false);
    if !table_exists {
        let empty: Vec<serde_json::Value> = Vec::new();
        return ok(empty);
    }
    let mut pricing = Vec::new();
    {
        let mut stmt = match conn.prepare(
            "SELECT model_id, display_name, input_cost_per_million, output_cost_per_million,
                    cache_read_cost_per_million, cache_creation_cost_per_million
             FROM model_pricing ORDER BY display_name",
        ) {
            Ok(s) => s,
            Err(e) => return err(e.to_string()),
        };
        let mut rows = match stmt.query([]) {
            Ok(r) => r,
            Err(e) => return err(e.to_string()),
        };
        while let Ok(Some(row)) = rows.next() {
            pricing.push(serde_json::json!({
                "modelId": row.get::<_, String>(0).unwrap_or_default(),
                "displayName": row.get::<_, String>(1).unwrap_or_default(),
                "inputCostPerMillion": row.get::<_, String>(2).unwrap_or_default(),
                "outputCostPerMillion": row.get::<_, String>(3).unwrap_or_default(),
                "cacheReadCostPerMillion": row.get::<_, String>(4).unwrap_or_default(),
                "cacheCreationCostPerMillion": row.get::<_, String>(5).unwrap_or_default(),
            }));
        }
    }
    ok(pricing)
}

async fn update_model_pricing(
    State((state, _)): State<Shared>,
    Json(body): Json<UpdatePricingBody>,
) -> Json<serde_json::Value> {
    let db = state.db.clone();
    let model_id = body.model_id.trim().to_string();
    let display_name = body.display_name.trim().to_string();
    if model_id.is_empty() {
        return err("model_id required");
    }
    if display_name.is_empty() {
        return err("display_name required");
    }

    let conn = match db.conn.lock() {
        Ok(c) => c,
        Err(e) => return err(format!("Mutex lock failed: {e}")),
    };
    match conn.execute(
        "INSERT OR REPLACE INTO model_pricing (
            model_id, display_name, input_cost_per_million, output_cost_per_million,
            cache_read_cost_per_million, cache_creation_cost_per_million
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            model_id,
            display_name,
            body.input_cost.trim(),
            body.output_cost.trim(),
            body.cache_read_cost.trim(),
            body.cache_creation_cost.trim()
        ],
    ) {
        Ok(_) => {
            let _ = db.backfill_missing_usage_costs_for_model(&model_id);
            ok(true)
        }
        Err(e) => err(e.to_string()),
    }
}

async fn delete_model_pricing(
    State((state, _)): State<Shared>,
    Json(body): Json<DeletePricingBody>,
) -> Json<serde_json::Value> {
    let db = state.db.clone();
    let conn = match db.conn.lock() {
        Ok(c) => c,
        Err(e) => return err(format!("Mutex lock failed: {e}")),
    };
    match conn.execute(
        "DELETE FROM model_pricing WHERE model_id = ?1",
        rusqlite::params![body.model_id],
    ) {
        Ok(_) => ok(true),
        Err(e) => err(e.to_string()),
    }
}

async fn check_provider_limits(
    State((state, _)): State<Shared>,
    Query(q): Query<ProviderLimitsQuery>,
) -> Json<serde_json::Value> {
    match state.db.check_provider_limits(&q.provider_id, &q.app_type) {
        Ok(s) => ok(s),
        Err(e) => err(e),
    }
}

async fn sync_session_usage(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    let mut result = crate::services::session_usage::sync_claude_session_logs(&state.db)
        .unwrap_or_else(|e| crate::services::session_usage::SessionSyncResult {
            imported: 0,
            skipped: 0,
            files_scanned: 0,
            errors: vec![e.to_string()],
        });

    match crate::services::session_usage_codex::sync_codex_usage(&state.db) {
        Ok(r) => {
            result.imported += r.imported;
            result.skipped += r.skipped;
            result.files_scanned += r.files_scanned;
            result.errors.extend(r.errors);
        }
        Err(e) => {
            result.errors.push(format!("Codex: {e}"));
        }
    }
    match crate::services::session_usage_gemini::sync_gemini_usage(&state.db) {
        Ok(r) => {
            result.imported += r.imported;
            result.skipped += r.skipped;
            result.files_scanned += r.files_scanned;
            result.errors.extend(r.errors);
        }
        Err(e) => {
            result.errors.push(format!("Gemini: {e}"));
        }
    }
    match crate::services::session_usage_opencode::sync_opencode_usage(&state.db) {
        Ok(r) => {
            result.imported += r.imported;
            result.skipped += r.skipped;
            result.files_scanned += r.files_scanned;
            result.errors.extend(r.errors);
        }
        Err(e) => {
            result.errors.push(format!("OpenCode: {e}"));
        }
    }

    ok(result)
}

async fn get_data_sources(State((state, _)): State<Shared>) -> Json<serde_json::Value> {
    match crate::services::session_usage::get_data_source_breakdown(&state.db) {
        Ok(s) => ok(s),
        Err(e) => err(e),
    }
}
