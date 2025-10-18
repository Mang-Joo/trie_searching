use anyhow::Result;
use chrono::{DateTime, Utc};
use mongodb::{
    bson::{doc, DateTime as BsonDateTime},
    options::ClientOptions,
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};

/// MongoDB에 저장되는 검색어 문서
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTermDocument {
    #[serde(deserialize_with = "deserialize_word")]
    pub word: String,
    #[serde(default)]
    pub frequency: Option<u64>,
    #[serde(default, rename = "approx_frequency")]
    pub approx_frequency: Option<u64>,
    #[serde(default)]
    pub last_searched: Option<DateTime<Utc>>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

fn deserialize_word<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum WordValue {
        Str(String),
        Int(i64),
        Float(f64),
        Bool(bool),
        Null,
    }

    match WordValue::deserialize(deserializer)? {
        WordValue::Str(s) => Ok(s),
        WordValue::Int(n) => Ok(n.to_string()),
        WordValue::Float(f) => Ok(f.to_string()),
        WordValue::Bool(b) => Ok(b.to_string()),
        WordValue::Null => Err(de::Error::custom("word field is null")),
    }
}

/// MongoDB Repository - 검색어 영구 저장
#[derive(Clone)]
pub struct MongoRepository {
    collection: Collection<SearchTermDocument>,
}

impl MongoRepository {
    /// MongoDB 연결 및 초기화
    pub async fn new(uri: &str, database: &str, collection: &str) -> Result<Self> {
        let client_options = ClientOptions::parse(uri).await?;
        let client = Client::with_options(client_options)?;

        let db = client.database(database);
        let collection = db.collection::<SearchTermDocument>(collection);

        // 인덱스 생성 (term 필드에 유니크 인덱스)
        let index = mongodb::IndexModel::builder()
            .keys(doc! { "word": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .partial_filter_expression(doc! { "word": { "$type": "string" } })
                    .build(),
            )
            .build();

        collection.create_index(index).await?;

        Ok(Self { collection })
    }

    /// 검색어 저장 또는 빈도수 증가
    pub async fn upsert_term(&self, term: &str) -> Result<()> {
        let now = Utc::now();
        let bson_now = BsonDateTime::from_millis(now.timestamp_millis());

        let filter = doc! { "word": term };
        let update = doc! {
            "$inc": { "frequency": 1 },
            "$set": { "last_searched": bson_now },
            "$setOnInsert": {
                "word": term,
                "created_at": bson_now
            }
        };

        self.collection
            .update_one(filter, update)
            .upsert(true)
            .await?;

        Ok(())
    }

    /// 특정 prefix로 시작하는 검색어들 가져오기
    pub async fn find_by_prefix(
        &self,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<SearchTermDocument>> {
        // MongoDB regex에서 특수문자 이스케이프
        let escaped = prefix
            .replace("\\", "\\\\")
            .replace(".", "\\.")
            .replace("*", "\\*")
            .replace("+", "\\+")
            .replace("?", "\\?")
            .replace("^", "\\^")
            .replace("$", "\\$")
            .replace("[", "\\[")
            .replace("]", "\\]")
            .replace("(", "\\(")
            .replace(")", "\\)")
            .replace("{", "\\{")
            .replace("}", "\\}")
            .replace("|", "\\|");

        let filter = doc! {
            "word": {
                "$regex": format!("^{}", escaped),
                "$options": "i"
            }
        };

        let mut cursor = self
            .collection
            .find(filter)
            .sort(doc! { "frequency": -1, "approx_frequency": -1, "word": 1 })
            .limit(limit)
            .await?;
        let mut results = Vec::new();

        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }

    /// 모든 검색어 가져오기 (Trie 초기화용)
    pub async fn get_all_terms(&self) -> Result<Vec<(String, u64)>> {
        let mut cursor = self
            .collection
            .find(doc! {})
            .projection(doc! { "word": 1, "frequency": 1, "approx_frequency": 1, "_id": 0 })
            .await?;
        let mut results = Vec::new();

        while cursor.advance().await? {
            let doc = cursor.deserialize_current()?;
            let freq = doc
                .frequency
                .or(doc.approx_frequency)
                .unwrap_or(0);
            results.push((doc.word, freq));
        }

        Ok(results)
    }

    /// 특정 검색어의 빈도수 가져오기
    pub async fn get_frequency(&self, term: &str) -> Result<Option<u64>> {
        let filter = doc! { "word": term };
        let doc = self.collection.find_one(filter).await?;
        Ok(doc.and_then(|d| d.frequency.or(d.approx_frequency)))
    }

    /// 인기 검색어 Top N
    pub async fn get_top_searches(&self, limit: i64) -> Result<Vec<SearchTermDocument>> {
        let mut cursor = self
            .collection
            .find(doc! {})
            .sort(doc! { "frequency": -1, "approx_frequency": -1, "word": 1 })
            .limit(limit)
            .await?;
        let mut results = Vec::new();

        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }

        Ok(results)
    }
}
