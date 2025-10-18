use crate::domain::tries::Trie;
use crate::repository::{MongoRepository, RedisRepository};
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

/// 검색어 추천 서비스 - Trie, MongoDB, Redis 통합
#[derive(Clone)]
pub struct SuggestionService {
    /// 인메모리 Trie (빠른 검색)
    trie: Arc<RwLock<Trie>>,
    /// MongoDB (영구 저장)
    mongo: MongoRepository,
    /// Redis (캐싱)
    redis: RedisRepository,
    /// 캐시 TTL (초)
    cache_ttl: u64,
}

impl SuggestionService {
    /// 서비스 초기화
    pub async fn new(
        mongo: MongoRepository,
        redis: RedisRepository,
        cache_ttl: u64,
    ) -> Result<Self> {
        let mut trie = Trie::new();

        // MongoDB에서 모든 검색어를 불러와 Trie 초기화
        info!("Initializing Trie from MongoDB...");
        let terms = mongo.get_all_terms().await?;
        info!("Loaded {} terms from MongoDB", terms.len());

        let build_start = Instant::now();
        let mut total_frequency: u64 = 0;

        for (term, frequency) in terms {
            trie.insert_with_frequency(&term, frequency);
            total_frequency = total_frequency.saturating_add(frequency);
        }

        let build_elapsed = build_start.elapsed();
        info!(
            "Trie populated: {} nodes, total frequency {} ({} ms)",
            trie.node_count(),
            total_frequency,
            build_elapsed.as_millis()
        );

        Ok(Self {
            trie: Arc::new(RwLock::new(trie)),
            mongo,
            redis,
            cache_ttl,
        })
    }

    /// 검색어 추가 (사용자가 검색할 때 호출)
    pub async fn record_search(&self, term: &str) -> Result<()> {
        if term.is_empty() {
            return Ok(());
        }

        // 1. Trie에 추가
        {
            let mut trie = self.trie.write();
            trie.insert(term);
        }

        // 2. MongoDB에 저장/업데이트 (비동기)
        if let Err(e) = self.mongo.upsert_term(term).await {
            warn!("Failed to save term to MongoDB: {}", e);
        }

        // 3. Redis 실시간 인기 검색어 업데이트
        if let Err(e) = self.redis.increment_search_count(term).await {
            warn!("Failed to update Redis trending: {}", e);
        }

        // 4. 관련 캐시 무효화
        let cache_pattern = format!("suggest:{}*", &term[..term.len().min(3)]);
        if let Err(e) = self.redis.invalidate_pattern(&cache_pattern).await {
            warn!("Failed to invalidate cache: {}", e);
        }

        Ok(())
    }

    /// 검색어 추천 (prefix 기반)
    pub async fn suggest(&self, prefix: &str, limit: usize) -> Result<Vec<(String, u64)>> {
        if prefix.is_empty() {
            return Ok(vec![]);
        }

        let cache_key = format!("suggest:{}:{}", prefix, limit);

        // 1. Redis 캐시 확인
        if let Ok(Some(cached)) = self.redis.get_json::<Vec<(String, u64)>>(&cache_key).await {
            info!("Cache hit for prefix: {}", prefix);
            return Ok(cached);
        }

        // 2. Trie에서 검색
        let compute_start = Instant::now();
        let results = {
            let trie = self.trie.read();
            trie.suggest_top_k(prefix, limit)
        };
        let compute_elapsed = compute_start.elapsed();
        info!(
            "Trie suggestion computed: prefix='{}', limit={}, results={}, duration_ms={}",
            prefix,
            limit,
            results.len(),
            compute_elapsed.as_secs_f64() * 1000.0
        );

        // 3. 결과를 Redis에 캐싱
        if let Err(e) = self
            .redis
            .set_json(&cache_key, &results, self.cache_ttl)
            .await
        {
            warn!("Failed to cache results: {}", e);
        }

        Ok(results)
    }

    /// 특정 검색어 포함 여부 확인
    pub fn contains(&self, term: &str) -> bool {
        let trie = self.trie.read();
        trie.contains(term)
    }

    /// Prefix로 시작하는 검색어 존재 여부
    pub fn starts_with(&self, prefix: &str) -> bool {
        let trie = self.trie.read();
        trie.starts_with(prefix)
    }

    /// 실시간 인기 검색어 Top N (Redis 기반)
    pub async fn get_trending(&self, limit: isize) -> Result<Vec<(String, f64)>> {
        let cache_key = format!("trending:top:{}", limit);

        // 짧은 TTL로 캐싱 (예: 10초)
        if let Ok(Some(cached)) = self.redis.get_json::<Vec<(String, f64)>>(&cache_key).await {
            return Ok(cached);
        }

        let trending = self.redis.get_trending(limit).await?;

        if let Err(e) = self.redis.set_json(&cache_key, &trending, 10).await {
            warn!("Failed to cache trending: {}", e);
        }

        Ok(trending)
    }

    /// MongoDB 기반 인기 검색어 (전체 통계)
    pub async fn get_top_searches(&self, limit: i64) -> Result<Vec<(String, u64)>> {
        let cache_key = format!("top:searches:{}", limit);

        // 긴 TTL로 캐싱 (예: 5분)
        if let Ok(Some(cached)) = self.redis.get_json::<Vec<(String, u64)>>(&cache_key).await {
            return Ok(cached);
        }

        let top = self.mongo.get_top_searches(limit).await?;
        let results: Vec<(String, u64)> = top
            .into_iter()
            .map(|doc| {
                let frequency = doc.frequency.or(doc.approx_frequency).unwrap_or(0);
                (doc.word, frequency)
            })
            .collect();

        if let Err(e) = self.redis.set_json(&cache_key, &results, 300).await {
            warn!("Failed to cache top searches: {}", e);
        }

        Ok(results)
    }

    /// Trie 재초기화 (MongoDB 데이터 동기화)
    pub async fn rebuild_trie(&self) -> Result<()> {
        info!("Rebuilding Trie from MongoDB...");

        let terms = self.mongo.get_all_terms().await?;
        let mut new_trie = Trie::new();

        let build_start = Instant::now();
        let mut total_frequency: u64 = 0;

        for (term, frequency) in terms {
            new_trie.insert_with_frequency(&term, frequency);
            total_frequency = total_frequency.saturating_add(frequency);
        }

        {
            let mut trie = self.trie.write();
            *trie = new_trie;
        }

        // 모든 캐시 무효화
        self.redis.invalidate_pattern("suggest:*").await?;

        let build_elapsed = build_start.elapsed();
        info!(
            "Trie rebuilt: total frequency {} ({} ms)",
            total_frequency,
            build_elapsed.as_millis()
        );

        info!("Trie rebuilt successfully");
        Ok(())
    }
}
