use std::env;

/// Application configuration loaded from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    // Elasticsearch
    pub es_host: String,
    pub es_username: String,
    pub es_password: String,
    pub es_index_pattern: String,

    // Server
    pub port: u16,

    // RapidAPI
    pub rapidapi_proxy_secret: String,

    // Rate Limits (requests per hour)
    pub rate_limit_basic: u64,
    pub rate_limit_pro: u64,
    pub rate_limit_ultra: u64,
    pub rate_limit_mega: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            es_host: env::var("ES_HOST").unwrap_or_else(|_| "https://local-es.nusarithm.id".into()),
            es_username: env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".into()),
            es_password: env::var("ES_PASSWORD").unwrap_or_else(|_| String::new()),
            es_index_pattern: env::var("ES_INDEX_PATTERN").unwrap_or_else(|_| "online-news-*".into()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap_or(3000),
            rapidapi_proxy_secret: env::var("RAPIDAPI_PROXY_SECRET").unwrap_or_default(),
            rate_limit_basic: env::var("RATE_LIMIT_BASIC").unwrap_or_else(|_| "5".into()).parse().unwrap_or(5),
            rate_limit_pro: env::var("RATE_LIMIT_PRO").unwrap_or_else(|_| "100".into()).parse().unwrap_or(100),
            rate_limit_ultra: env::var("RATE_LIMIT_ULTRA").unwrap_or_else(|_| "1000".into()).parse().unwrap_or(1000),
            rate_limit_mega: env::var("RATE_LIMIT_MEGA").unwrap_or_else(|_| "10000".into()).parse().unwrap_or(10000),
        }
    }
}
