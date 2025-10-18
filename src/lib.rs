pub mod api;
pub mod domain;
pub mod repository;
pub mod service;

pub use domain::Trie;
pub use repository::{MongoRepository, RedisRepository};
pub use service::SuggestionService;
