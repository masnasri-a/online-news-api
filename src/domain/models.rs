use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════
//  News Article (core domain model)
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsArticle {
    #[serde(skip_deserializing)]
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headline_image: Option<String>,
    #[serde(default)]
    pub headline_caption: Option<String>,
    #[serde(default)]
    pub publish_date: Option<String>,
    #[serde(default)]
    pub publish_date_timestamp: Option<i64>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub extracted_at: Option<String>,
    #[serde(default)]
    pub ingested_at: Option<String>,
    #[serde(default)]
    pub annotate: Option<Annotation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotation {
    #[serde(default)]
    pub sentiment: Option<SentimentData>,
    #[serde(default)]
    pub emotion: Option<EmotionData>,
    #[serde(default)]
    pub entities: Option<Vec<EntityData>>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SentimentData {
    pub label: Option<String>,
    pub score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmotionData {
    pub label: Option<String>,
    pub score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntityData {
    pub word: Option<String>,
    pub entity_group: Option<String>,
    pub score: Option<f64>,
    #[serde(default)]
    pub start: Option<i64>,
    #[serde(default)]
    pub end: Option<i64>,
}

// ═══════════════════════════════════════════════════════════
//  Aggregation Data
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Clone)]
pub struct SourceInfo {
    pub name: String,
    pub doc_count: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct StatsData {
    pub total_articles: u64,
    pub sources: Vec<SourceInfo>,
    pub date_range: DateRange,
}

#[derive(Debug, Serialize, Clone)]
pub struct DateRange {
    pub earliest: Option<String>,
    pub latest: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrendingItem {
    pub keyword: String,
    pub category: String,
    pub count: u64,
}

// ═══════════════════════════════════════════════════════════
//  Search Parameters
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct NewsSearchParams {
    pub q: Option<String>,
    pub source: Option<String>,
    pub tag: Option<String>,
    pub sentiment: Option<String>,
    pub emotion: Option<String>,
    pub author: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u64>,
    pub size: Option<u64>,
}
