use crate::domain::models::*;
use crate::domain::tier::SubscriptionTier;
use crate::errors::AppError;
use crate::infrastructure::elasticsearch::EsRepository;

/// Service layer — contains business logic for news operations.
/// Applies tier-based content gating on top of raw repository data.
#[derive(Clone)]
pub struct NewsService {
    repo: EsRepository,
}

impl NewsService {
    pub fn new(repo: EsRepository) -> Self {
        Self { repo }
    }

    /// Search news with tier-appropriate content and page limits.
    pub async fn search(
        &self,
        params: &NewsSearchParams,
        tier: &SubscriptionTier,
    ) -> Result<(Vec<NewsArticle>, u64), AppError> {
        let max_size = tier.max_page_size();
        let (articles, total) = self.repo.search(params, max_size).await?;
        let gated = self.apply_content_gating(articles, tier);
        Ok((gated, total))
    }

    /// Get a single article with tier-appropriate content.
    pub async fn get_by_id(
        &self,
        id: &str,
        tier: &SubscriptionTier,
    ) -> Result<NewsArticle, AppError> {
        let article = self.repo.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound(format!("Article '{}' not found", id)))?;

        Ok(self.gate_article(article, tier))
    }

    /// List all news sources.
    pub async fn list_sources(&self) -> Result<Vec<SourceInfo>, AppError> {
        self.repo.aggregate_sources().await
    }

    /// Get dataset statistics.
    pub async fn stats(&self) -> Result<StatsData, AppError> {
        self.repo.aggregate_stats().await
    }

    /// Get trending topics.
    pub async fn trending(&self) -> Result<Vec<TrendingItem>, AppError> {
        self.repo.trending().await
    }

    /// Check Elasticsearch health.
    pub async fn health(&self) -> Result<String, AppError> {
        self.repo.health().await
    }

    // ─── Private: Content Gating ─────────────────────────────

    fn apply_content_gating(
        &self,
        articles: Vec<NewsArticle>,
        tier: &SubscriptionTier,
    ) -> Vec<NewsArticle> {
        articles.into_iter()
            .map(|a| self.gate_article(a, tier))
            .collect()
    }

    fn gate_article(&self, mut article: NewsArticle, tier: &SubscriptionTier) -> NewsArticle {
        // Truncate content for tiers without full access
        if !tier.has_full_content() {
            if let Some(ref content) = article.content {
                let truncated: String = content.chars().take(200).collect();
                article.content = Some(if content.chars().count() > 200 {
                    format!("{}...", truncated)
                } else {
                    truncated
                });
            }
        }

        // Remove entities for tiers without entity access
        if !tier.has_entities() {
            if let Some(ref mut annotate) = article.annotate {
                annotate.entities = None;
            }
        }

        article
    }
}
