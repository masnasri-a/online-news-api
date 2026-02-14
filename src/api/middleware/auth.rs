use std::future::{Ready, ready};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse, Transform, Service},
    Error, HttpResponse, body::EitherBody,
};
use serde::Serialize;
use log::warn;

/// Actix-web middleware that validates the `X-RapidAPI-Proxy-Secret` header.
/// Skips validation in dev mode (empty or placeholder secret).
pub struct RapidApiAuth {
    pub proxy_secret: String,
}

impl<S, B> Transform<S, ServiceRequest> for RapidApiAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RapidApiAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RapidApiAuthMiddleware {
            service,
            proxy_secret: self.proxy_secret.clone(),
        }))
    }
}

pub struct RapidApiAuthMiddleware<S> {
    service: S,
    proxy_secret: String,
}

#[derive(Serialize)]
struct AuthError {
    success: bool,
    error: AuthErrorDetail,
}

#[derive(Serialize)]
struct AuthErrorDetail {
    code: u16,
    message: String,
}

impl<S, B> Service<ServiceRequest> for RapidApiAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let is_dev = self.proxy_secret.is_empty()
            || self.proxy_secret == "your-rapidapi-proxy-secret-here";

        // Skip auth for health endpoint or dev mode
        if req.path() == "/api/health" || is_dev {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        // Validate proxy secret
        let header = req.headers()
            .get("X-RapidAPI-Proxy-Secret")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if header != self.proxy_secret {
            warn!("Rejected: invalid X-RapidAPI-Proxy-Secret");
            let body = AuthError {
                success: false,
                error: AuthErrorDetail {
                    code: 403,
                    message: "Forbidden: Invalid API proxy secret".into(),
                },
            };
            let response = HttpResponse::Forbidden().json(body);
            return Box::pin(async move {
                Ok(req.into_response(response).map_into_right_body())
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
