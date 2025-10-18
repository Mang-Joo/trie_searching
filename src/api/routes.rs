use crate::service::SuggestionService;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

/// API 응답 구조
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// 검색어 추천 요청
#[derive(Debug, Deserialize)]
pub struct SuggestQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

/// 검색어 기록 요청
#[derive(Debug, Deserialize)]
pub struct RecordRequest {
    pub term: String,
}

/// 인기 검색어 요청
#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    #[serde(default = "default_trending_limit")]
    pub limit: i64,
}

fn default_trending_limit() -> i64 {
    10
}

/// Router 생성
pub fn create_router(service: SuggestionService) -> Router {
    let state = Arc::new(service);

    Router::new()
        .route("/health", get(health_check))
        .route("/suggest", get(suggest_handler))
        .route("/search", post(record_search_handler))
        .route("/trending", get(trending_handler))
        .route("/top", get(top_searches_handler))
        .route("/contains", get(contains_handler))
        .route("/rebuild", post(rebuild_handler))
        .with_state(state)
}

/// Health check
async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::success("OK".to_string()))
}

/// 검색어 추천 API
/// GET /suggest?q=han&limit=5
async fn suggest_handler(
    State(service): State<Arc<SuggestionService>>,
    Query(query): Query<SuggestQuery>,
) -> impl IntoResponse {
    match service.suggest(&query.q, query.limit).await {
        Ok(results) => (StatusCode::OK, Json(ApiResponse::success(results))),
        Err(e) => {
            error!("Suggest error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<(String, u64)>>::error(e.to_string())),
            )
        }
    }
}

/// 검색어 기록 API
/// POST /search
/// Body: { "term": "hankook" }
async fn record_search_handler(
    State(service): State<Arc<SuggestionService>>,
    Json(payload): Json<RecordRequest>,
) -> impl IntoResponse {
    match service.record_search(&payload.term).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success("Recorded".to_string())),
        ),
        Err(e) => {
            error!("Record search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(e.to_string())),
            )
        }
    }
}

/// 실시간 인기 검색어 API
/// GET /trending?limit=10
async fn trending_handler(
    State(service): State<Arc<SuggestionService>>,
    Query(query): Query<TrendingQuery>,
) -> impl IntoResponse {
    match service.get_trending(query.limit as isize).await {
        Ok(results) => (StatusCode::OK, Json(ApiResponse::success(results))),
        Err(e) => {
            error!("Trending error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<(String, f64)>>::error(e.to_string())),
            )
        }
    }
}

/// 전체 인기 검색어 API (MongoDB 기반)
/// GET /top?limit=10
async fn top_searches_handler(
    State(service): State<Arc<SuggestionService>>,
    Query(query): Query<TrendingQuery>,
) -> impl IntoResponse {
    match service.get_top_searches(query.limit).await {
        Ok(results) => (StatusCode::OK, Json(ApiResponse::success(results))),
        Err(e) => {
            error!("Top searches error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<(String, u64)>>::error(e.to_string())),
            )
        }
    }
}

/// 검색어 포함 여부 확인 API
/// GET /contains?q=hankook
async fn contains_handler(
    State(service): State<Arc<SuggestionService>>,
    Query(query): Query<SuggestQuery>,
) -> impl IntoResponse {
    let exists = service.contains(&query.q);
    (StatusCode::OK, Json(ApiResponse::success(exists)))
}

/// Trie 재구성 API (관리자용)
/// POST /rebuild
async fn rebuild_handler(State(service): State<Arc<SuggestionService>>) -> impl IntoResponse {
    match service.rebuild_trie().await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Trie rebuilt successfully".to_string(),
            )),
        ),
        Err(e) => {
            error!("Rebuild error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(e.to_string())),
            )
        }
    }
}
