mod api;
mod domain;
mod repository;
mod service;

use anyhow::Result;
use repository::{MongoRepository, RedisRepository};
use service::SuggestionService;
use dotenvy::dotenv;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // 로깅 초기화
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tries_suggest=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // .env 파일 로드 (없으면 조용히 무시)
    dotenv().ok();

    // 환경 변수에서 설정 읽기 (또는 기본값 사용)
    let mongo_uri =
        std::env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mongo_db = std::env::var("MONGO_DB").unwrap_or_else(|_| "search_db".to_string());
    let mongo_collection =
        std::env::var("MONGO_COLLECTION").unwrap_or_else(|_| "search_terms".to_string());

    let redis_uri =
        std::env::var("REDIS_URI").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_ttl: u64 = std::env::var("CACHE_TTL")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    info!("Connecting to MongoDB at {}...", mongo_uri);
    let mongo = MongoRepository::new(&mongo_uri, &mongo_db, &mongo_collection).await?;

    info!("Connecting to Redis at {}...", redis_uri);
    let redis = RedisRepository::new(&redis_uri).await?;

    info!("Initializing SuggestionService...");
    let service = SuggestionService::new(mongo, redis, cache_ttl).await?;

    // CORS 설정
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API 라우터 생성
    let app = api::create_router(service).layer(cors);

    // 서버 시작
    let addr = format!("0.0.0.0:{}", port);
    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("🚀 Server is running on http://{}", addr);
    info!("📋 API Endpoints:");
    info!("  GET  /health              - Health check");
    info!("  GET  /suggest?q=han&limit=5 - Get suggestions");
    info!("  POST /search              - Record search (body: {{\"term\": \"...\"}})");
    info!("  GET  /trending?limit=10   - Get real-time trending searches");
    info!("  GET  /top?limit=10        - Get all-time top searches");
    info!("  GET  /contains?q=term     - Check if term exists");
    info!("  POST /rebuild             - Rebuild trie from DB");

    axum::serve(listener, app).await?;

    Ok(())
}
