use actix_web::{web, HttpRequest, HttpResponse};


use crate::api::middleware::rate_limiter::RateLimiter;
use crate::api::response::ResponseBuilder;
use crate::domain::models::NewsSearchParams;
use crate::domain::tier::SubscriptionTier;
use crate::errors::AppError;
use crate::services::news_service::NewsService;

// ─── Helpers ─────────────────────────────────────────────────

fn get_tier(req: &HttpRequest) -> SubscriptionTier {
    let header = req.headers()
        .get("X-RapidAPI-Subscription")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("BASIC");
    SubscriptionTier::from_header(header)
}

fn get_user(req: &HttpRequest) -> String {
    req.headers()
        .get("X-RapidAPI-User")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string()
}

/// Check rate limit and return headers or ErrorResponse.
fn check_rate_limit(
    req: &HttpRequest,
    limiter: &RateLimiter,
) -> Result<(SubscriptionTier, u64, u64), AppError> {
    let tier = get_tier(req);
    let user = get_user(req);
    
    // In dev mode with no headers, we might want to be lenient or default to Basic
    let (limit, remaining) = limiter.check(&user, &tier)?;
    Ok((tier, limit, remaining))
}

// ─── Handlers ────────────────────────────────────────────────

pub async fn health(service: web::Data<NewsService>) -> HttpResponse {
    let status = service.health().await.unwrap_or_else(|_| "unavailable".to_string());
    ResponseBuilder::ok(serde_json::json!({
        "status": "ok",
        "version": "1.1.0",
        "elasticsearch": status
    }))
}

pub async fn search_news(
    req: HttpRequest,
    params: web::Query<NewsSearchParams>,
    service: web::Data<NewsService>,
    limiter: web::Data<RateLimiter>,
) -> HttpResponse {
    let (tier, limit, remaining) = match check_rate_limit(&req, &limiter) {
        Ok(v) => v,
        Err(e) => return e.to_response(),
    };

    match service.search(&params, &tier).await {
        Ok((articles, total)) => {
            let page = params.page.unwrap_or(1).max(1);
            let size = params.size.unwrap_or(10).min(tier.max_page_size());
            
            let resp = ResponseBuilder::ok_paged(articles, page, size, total);
            ResponseBuilder::with_rate_headers(
                resp, 
                limit, 
                remaining, 
                &RateLimiter::reset_time(), 
                tier.name()
            )
        }
        Err(e) => e.to_response(),
    }
}

pub async fn get_article(
    req: HttpRequest,
    id: web::Path<String>,
    service: web::Data<NewsService>,
    limiter: web::Data<RateLimiter>,
) -> HttpResponse {
    let (tier, limit, remaining) = match check_rate_limit(&req, &limiter) {
        Ok(v) => v,
        Err(e) => return e.to_response(),
    };

    match service.get_by_id(&id, &tier).await {
        Ok(article) => {
            let resp = ResponseBuilder::ok(article);
            ResponseBuilder::with_rate_headers(
                resp, 
                limit, 
                remaining, 
                &RateLimiter::reset_time(), 
                tier.name()
            )
        }
        Err(e) => e.to_response(),
    }
}

pub async fn list_sources(
    req: HttpRequest,
    service: web::Data<NewsService>,
    limiter: web::Data<RateLimiter>,
) -> HttpResponse {
    let (tier, limit, remaining) = match check_rate_limit(&req, &limiter) {
        Ok(v) => v,
        Err(e) => return e.to_response(),
    };

    match service.list_sources().await {
        Ok(sources) => {
            let resp = ResponseBuilder::ok(sources);
            ResponseBuilder::with_rate_headers(
                resp, 
                limit, 
                remaining, 
                &RateLimiter::reset_time(), 
                tier.name()
            )
        }
        Err(e) => e.to_response(),
    }
}

pub async fn get_stats(
    req: HttpRequest,
    service: web::Data<NewsService>,
    limiter: web::Data<RateLimiter>,
) -> HttpResponse {
    let (tier, limit, remaining) = match check_rate_limit(&req, &limiter) {
        Ok(v) => v,
        Err(e) => return e.to_response(),
    };

    match service.stats().await {
        Ok(stats) => {
            let resp = ResponseBuilder::ok(stats);
            ResponseBuilder::with_rate_headers(
                resp, 
                limit, 
                remaining, 
                &RateLimiter::reset_time(), 
                tier.name()
            )
        }
        Err(e) => e.to_response(),
    }
}

pub async fn get_trending(
    req: HttpRequest,
    service: web::Data<NewsService>,
    limiter: web::Data<RateLimiter>,
) -> HttpResponse {
    let (tier, limit, remaining) = match check_rate_limit(&req, &limiter) {
        Ok(v) => v,
        Err(e) => return e.to_response(),
    };

    match service.trending().await {
        Ok(items) => {
            let resp = ResponseBuilder::ok(items);
            ResponseBuilder::with_rate_headers(
                resp, 
                limit, 
                remaining, 
                &RateLimiter::reset_time(), 
                tier.name()
            )
        }
        Err(e) => e.to_response(),
    }
}
