use actix_web::web;
use crate::api::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .route("ping", web::get().to(handlers::health))
    )
    .service(
        web::scope("/api")
            .route("/health", web::get().to(handlers::health))
            .route("/news", web::get().to(handlers::search_news))
            .route("/news/sources", web::get().to(handlers::list_sources))
            .route("/news/stats", web::get().to(handlers::get_stats))
            .route("/news/trending", web::get().to(handlers::get_trending))
            .route("/news/{id}", web::get().to(handlers::get_article))
    );
}
