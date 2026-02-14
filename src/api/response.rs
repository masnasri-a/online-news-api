use actix_web::HttpResponse;
use serde::Serialize;

/// Standard paginated API response.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u64,
    pub size: u64,
    pub total: u64,
    pub total_pages: u64,
}

/// Builder for consistent API responses with rate-limit headers.
pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn ok<T: Serialize>(data: T) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse::<T> {
            success: true,
            data,
            meta: None,
        })
    }

    pub fn ok_paged<T: Serialize>(
        data: T,
        page: u64,
        size: u64,
        total: u64,
    ) -> HttpResponse {
        let total_pages = if total > 0 { (total + size - 1) / size } else { 0 };
        HttpResponse::Ok().json(ApiResponse::<T> {
            success: true,
            data,
            meta: Some(PaginationMeta { page, size, total, total_pages }),
        })
    }

    /// Attach rate-limit headers to an already-built response.
    pub fn with_rate_headers(
        mut resp: HttpResponse,
        limit: u64,
        remaining: u64,
        reset_at: &str,
        tier: &str,
    ) -> HttpResponse {
        let headers = resp.headers_mut();
        headers.insert("X-RateLimit-Limit".parse().unwrap(), limit.to_string().parse().unwrap());
        headers.insert("X-RateLimit-Remaining".parse().unwrap(), remaining.to_string().parse().unwrap());
        headers.insert("X-RateLimit-Reset".parse().unwrap(), reset_at.parse().unwrap());
        headers.insert("X-Subscription-Tier".parse().unwrap(), tier.parse().unwrap());
        resp
    }
}
