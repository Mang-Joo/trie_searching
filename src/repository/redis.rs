use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Instant;
use tracing::info;

/// Redis Repository - 캐싱 레이어
#[derive(Clone)]
pub struct RedisRepository {
    conn: ConnectionManager,
}

impl RedisRepository {
    /// Redis 연결
    pub async fn new(uri: &str) -> Result<Self> {
        let client = redis::Client::open(uri)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    /// 캐시에 JSON 데이터 저장 (TTL 설정)
    pub async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<()> {
        let json = serde_json::to_string(value)?;
        let mut conn = self.conn.clone();
        let start = Instant::now();
        conn.set_ex::<_, _, ()>(key, json, ttl_seconds).await?;
        let elapsed = start.elapsed();
        info!(
            target: "tries_suggest::redis",
            key = key,
            ttl_seconds,
            duration_ms = elapsed.as_secs_f64() * 1000.0,
            "redis_set_json"
        );
        Ok(())
    }

    /// 캐시에서 JSON 데이터 가져오기
    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.conn.clone();
        let start = Instant::now();
        let result: Option<String> = conn.get(key).await?;
        let elapsed = start.elapsed();
        info!(
            target: "tries_suggest::redis",
            key = key,
            hit = result.is_some(),
            duration_ms = elapsed.as_secs_f64() * 1000.0,
            "redis_get_json"
        );

        match result {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    /// 캐시 삭제
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    /// 패턴으로 캐시 무효화
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        let keys: Vec<String> = conn.keys(pattern).await?;

        if !keys.is_empty() {
            conn.del::<_, ()>(keys).await?;
        }

        Ok(())
    }

    /// Sorted Set에 검색어 추가 (실시간 인기 검색어용)
    pub async fn increment_search_count(&self, term: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        conn.zincr::<_, _, _, ()>("trending:searches", term, 1.0)
            .await?;
        Ok(())
    }

    /// 실시간 인기 검색어 Top N
    pub async fn get_trending(&self, limit: isize) -> Result<Vec<(String, f64)>> {
        let mut conn = self.conn.clone();
        let results: Vec<(String, f64)> = conn
            .zrevrange_withscores("trending:searches", 0, limit - 1)
            .await?;
        Ok(results)
    }

    /// 캐시 키 존재 여부 확인
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.conn.clone();
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }
}
