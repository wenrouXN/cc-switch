use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::store::AppState;
use crate::web::WsState;

type Shared = (Arc<AppState>, Arc<WsState>);

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/quota", get(get_quota))
        .route("/oauth-quota", get(get_oauth_quota))
        .route("/coding-plan", post(get_coding_plan_quota))
        .route("/balance", post(get_balance))
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

fn err(msg: impl ToString) -> Json<serde_json::Value> {
    Json(json!({ "success": false, "error": msg.to_string() }))
}

#[derive(Deserialize)]
struct QuotaQuery {
    tool: String,
}

async fn get_quota(Query(query): Query<QuotaQuery>) -> Json<serde_json::Value> {
    match crate::services::subscription::get_subscription_quota(&query.tool).await {
        Ok(quota) => ok(quota),
        Err(error) => err(error),
    }
}

async fn get_oauth_quota(Query(query): Query<QuotaQuery>) -> Json<serde_json::Value> {
    ok(crate::services::subscription::SubscriptionQuota::not_found(
        &query.tool,
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodingPlanBody {
    base_url: String,
    api_key: String,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    coding_plan_provider: Option<String>,
    team_organization_id: Option<String>,
    team_project_id: Option<String>,
}

async fn get_coding_plan_quota(Json(body): Json<CodingPlanBody>) -> Json<serde_json::Value> {
    match crate::services::coding_plan::get_coding_plan_quota(
        &body.base_url,
        &body.api_key,
        body.access_key_id.as_deref(),
        body.secret_access_key.as_deref(),
        body.coding_plan_provider.as_deref(),
        body.team_organization_id.as_deref(),
        body.team_project_id.as_deref(),
    )
    .await
    {
        Ok(quota) => ok(quota),
        Err(error) => err(error),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BalanceBody {
    base_url: String,
    api_key: String,
}

async fn get_balance(Json(body): Json<BalanceBody>) -> Json<serde_json::Value> {
    match crate::services::balance::get_balance(&body.base_url, &body.api_key).await {
        Ok(balance) => ok(balance),
        Err(error) => err(error),
    }
}
