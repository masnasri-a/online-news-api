use reqwest::Client;
use serde_json::{json, Value};
use log::{info, error};

use crate::config::Config;
use crate::domain::models::*;
use crate::errors::AppError;

/// Elasticsearch repository — handles all communication with ES.
#[derive(Clone)]
pub struct EsRepository {
    client: Client,
    base_url: String,
    index_pattern: String,
    username: String,
    password: String,
}

impl EsRepository {
    pub fn new(config: &Config) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.es_host.clone(),
            index_pattern: config.es_index_pattern.clone(),
            username: config.es_username.clone(),
            password: config.es_password.clone(),
        }
    }

    fn search_url(&self) -> String {
        format!("{}/{}/_search", self.base_url, self.index_pattern)
    }

    /// Execute an ES request and parse the JSON response.
    async fn execute(&self, body: &Value) -> Result<Value, AppError> {
        let resp = self.client
            .post(&self.search_url())
            .basic_auth(&self.username, Some(&self.password))
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Elasticsearch(format!("Request failed: {}", e)))?;

        let json: Value = resp.json().await
            .map_err(|e| AppError::Elasticsearch(format!("Parse failed: {}", e)))?;

        if let Some(err) = json.get("error") {
            error!("ES error: {}", err);
            return Err(AppError::Elasticsearch(err.to_string()));
        }

        Ok(json)
    }

    /// Extract hits from an ES response into NewsArticle vec.
    fn parse_hits(json: &Value) -> Vec<NewsArticle> {
        json["hits"]["hits"]
            .as_array()
            .map(|hits| {
                hits.iter().filter_map(|hit| {
                    let mut article: NewsArticle =
                        serde_json::from_value(hit["_source"].clone()).ok()?;
                    article.id = hit["_id"].as_str().unwrap_or("").to_string();
                    Some(article)
                }).collect()
            })
            .unwrap_or_default()
    }

    fn parse_total(json: &Value) -> u64 {
        json["hits"]["total"]["value"].as_u64().unwrap_or(0)
    }

    // ─── Public Repository Methods ───────────────────────────

    /// Full-text search with filters and pagination.
    pub async fn search(
        &self,
        params: &NewsSearchParams,
        max_size: u64,
    ) -> Result<(Vec<NewsArticle>, u64), AppError> {
        let page = params.page.unwrap_or(1).max(1);
        let size = params.size.unwrap_or(10).min(max_size);
        let from = (page - 1) * size;

        let mut must: Vec<Value> = Vec::new();
        let mut filter: Vec<Value> = Vec::new();

        if let Some(ref q) = params.q {
            if !q.is_empty() {
                must.push(json!({
                    "multi_match": {
                        "query": q,
                        "fields": ["title^3", "content"],
                        "type": "best_fields",
                        "fuzziness": "AUTO"
                    }
                }));
            }
        }

        if let Some(ref v) = params.source    { filter.push(json!({"term": {"source": v}})); }
        if let Some(ref v) = params.tag       { filter.push(json!({"term": {"tags": v}})); }
        if let Some(ref v) = params.sentiment { filter.push(json!({"term": {"annotate.sentiment.label.keyword": v}})); }
        if let Some(ref v) = params.emotion   { filter.push(json!({"term": {"annotate.emotion.label.keyword": v}})); }
        if let Some(ref v) = params.author    { filter.push(json!({"term": {"author": v}})); }

        let mut range = serde_json::Map::new();
        if let Some(ref v) = params.date_from { range.insert("gte".into(), json!(v)); }
        if let Some(ref v) = params.date_to   { range.insert("lte".into(), json!(v)); }
        if !range.is_empty() {
            filter.push(json!({"range": {"ingested_at": range}}));
        }

        let query = if must.is_empty() && filter.is_empty() {
            json!({"match_all": {}})
        } else {
            let mut bool_q = serde_json::Map::new();
            if !must.is_empty()   { bool_q.insert("must".into(), json!(must)); }
            if !filter.is_empty() { bool_q.insert("filter".into(), json!(filter)); }
            json!({"bool": bool_q})
        };

        let sort = match params.sort.as_deref() {
            Some("oldest") => json!([{"ingested_at": {"order": "asc"}}]),
            Some("relevance") if params.q.is_some() => json!(["_score"]),
            _ => json!([{"ingested_at": {"order": "desc"}}]),
        };

        let body = json!({
            "query": query,
            "sort": sort,
            "from": from,
            "size": size,
            "track_total_hits": true
        });

        info!("ES search: {}", serde_json::to_string(&body).unwrap_or_default());

        let json = self.execute(&body).await?;
        let total = Self::parse_total(&json);
        let articles = Self::parse_hits(&json);

        Ok((articles, total))
    }

    /// Get a single article by its document ID.
    pub async fn find_by_id(&self, id: &str) -> Result<Option<NewsArticle>, AppError> {
        let body = json!({
            "query": { "ids": { "values": [id] } },
            "size": 1
        });

        let json = self.execute(&body).await?;
        let articles = Self::parse_hits(&json);
        Ok(articles.into_iter().next())
    }

    /// Aggregate all news sources with document counts.
    pub async fn aggregate_sources(&self) -> Result<Vec<SourceInfo>, AppError> {
        let body = json!({
            "size": 0,
            "aggs": { "sources": { "terms": { "field": "source", "size": 100 } } }
        });

        let json = self.execute(&body).await?;
        Ok(Self::parse_buckets(&json["aggregations"]["sources"]["buckets"]))
    }

    /// Aggregate overall statistics.
    pub async fn aggregate_stats(&self) -> Result<StatsData, AppError> {
        let body = json!({
            "size": 0,
            "track_total_hits": true,
            "aggs": {
                "sources":  { "terms": { "field": "source", "size": 100 } },
                "date_min": { "min": { "field": "ingested_at" } },
                "date_max": { "max": { "field": "ingested_at" } }
            }
        });

        let json = self.execute(&body).await?;

        Ok(StatsData {
            total_articles: Self::parse_total(&json),
            sources: Self::parse_buckets(&json["aggregations"]["sources"]["buckets"]),
            date_range: DateRange {
                earliest: json["aggregations"]["date_min"]["value_as_string"]
                    .as_str().map(String::from),
                latest: json["aggregations"]["date_max"]["value_as_string"]
                    .as_str().map(String::from),
            },
        })
    }

    /// Get trending entities and tags from the last 7 days.
    pub async fn trending(&self) -> Result<Vec<TrendingItem>, AppError> {
        let body = json!({
            "size": 0,
            "query": { "range": { "ingested_at": { "gte": "now-7d/d" } } },
            "aggs": {
                "entities": { "terms": { "field": "annotate.entities.word.keyword", "size": 20 } },
                "tags":     { "terms": { "field": "tags", "size": 20 } }
            }
        });

        let json = self.execute(&body).await?;

        let mut items: Vec<TrendingItem> = Vec::new();

        Self::collect_trending(&json["aggregations"]["entities"]["buckets"], "entity", &mut items);
        Self::collect_trending(&json["aggregations"]["tags"]["buckets"], "tag", &mut items);

        items.sort_by(|a, b| b.count.cmp(&a.count));
        Ok(items)
    }

    /// Check cluster health status.
    pub async fn health(&self) -> Result<String, AppError> {
        let url = format!("{}/_cluster/health", self.base_url);
        let resp = self.client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| AppError::Elasticsearch(format!("Health check failed: {}", e)))?;

        let json: Value = resp.json().await
            .map_err(|e| AppError::Elasticsearch(format!("Parse failed: {}", e)))?;

        Ok(json["status"].as_str().unwrap_or("unknown").to_string())
    }

    // ─── Private Helpers ─────────────────────────────────────

    fn parse_buckets(buckets: &Value) -> Vec<SourceInfo> {
        buckets.as_array()
            .map(|arr| {
                arr.iter().filter_map(|b| {
                    Some(SourceInfo {
                        name: b["key"].as_str()?.to_string(),
                        doc_count: b["doc_count"].as_u64()?,
                    })
                }).collect()
            })
            .unwrap_or_default()
    }

    fn collect_trending(buckets: &Value, category: &str, items: &mut Vec<TrendingItem>) {
        if let Some(arr) = buckets.as_array() {
            for b in arr {
                if let (Some(key), Some(count)) = (b["key"].as_str(), b["doc_count"].as_u64()) {
                    items.push(TrendingItem {
                        keyword: key.to_string(),
                        category: category.to_string(),
                        count,
                    });
                }
            }
        }
    }
}
