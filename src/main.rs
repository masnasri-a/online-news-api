mod config;
mod domain;
mod infrastructure;
mod services;
mod api;
mod errors;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware as actix_middleware};
use log::info;

use crate::config::Config;
use crate::infrastructure::elasticsearch::EsRepository;
use crate::services::news_service::NewsService;
use crate::api::middleware::auth::RapidApiAuth;
use crate::api::middleware::rate_limiter::RateLimiter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config = Config::from_env();
    let port = config.port;

    info!("🚀 Starting Indonesian Online News API (Clean Architecture Edition)");
    info!("🔌 Port: {}", port);
    info!("📊 Elasticsearch: {}", config.es_host);
    
    // Initialize Layers
    let es_repo = EsRepository::new(&config);
    let news_service = NewsService::new(es_repo);
    let rate_limiter = RateLimiter::new(config.clone());

    info!("🔒 Rate Limits (Hourly): Basic={}, Pro={}, Ultra={}, Mega={}", 
        config.rate_limit_basic, config.rate_limit_pro, 
        config.rate_limit_ultra, config.rate_limit_mega);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(actix_middleware::Logger::default())
            // Register Middlewares
            .wrap(RapidApiAuth {
                proxy_secret: config.rapidapi_proxy_secret.clone(),
            })
            // Inject Dependencies
            .app_data(web::Data::new(news_service.clone()))
            .app_data(web::Data::new(rate_limiter.clone()))
            // Register Routes
            .configure(api::routes::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
