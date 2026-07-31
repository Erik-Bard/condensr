pub mod config;
pub mod database;
pub mod middleware;
pub mod models;
pub mod routes;

use axum::{
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    middleware as axum_middleware,
};
use sqlx::PgPool;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
};

use crate::{
    config::{Config, CorsConfig, HttpConfig},
    database::pg_database,
    middleware::{
        ShortenRateLimit, enforce_request_timeout, enforce_shorten_rate_limit,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub base_url: String,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self, anyhow::Error> {
        Ok(AppState {
            db: pg_database::connect(&config.database_url).await?,
            base_url: config.base_url.to_string(),
        })
    }
}

pub fn build_router(state: AppState, config: &HttpConfig) -> axum::Router {
    let timeout_layer = || {
        axum_middleware::from_fn_with_state(
            config.request_timeout,
            enforce_request_timeout,
        )
    };

    let mut shorten = routes::shorten::router()
        .layer(RequestBodyLimitLayer::new(config.max_request_body_bytes));
    if let Some(rate_limit) = &config.shorten_rate_limit {
        shorten = shorten.layer(axum_middleware::from_fn_with_state(
            ShortenRateLimit::new(rate_limit),
            enforce_shorten_rate_limit,
        ));
    }
    shorten = shorten.layer(timeout_layer());

    let links = routes::links::api_router().layer(timeout_layer());
    let mut api = shorten.merge(links);
    if let Some(cors) = &config.cors {
        api = api.layer(cors_layer(cors));
    }

    let redirects = routes::links::redirect_router().layer(timeout_layer());

    axum::Router::new()
        .merge(routes::health::router())
        .merge(api)
        .merge(redirects)
        .with_state(state)
}

fn cors_layer(config: &CorsConfig) -> CorsLayer {
    let allow_origin = match config {
        CorsConfig::Any => AllowOrigin::any(),
        CorsConfig::Origins(origins) => AllowOrigin::list(
            origins
                .iter()
                .map(|origin| {
                    HeaderValue::from_str(origin)
                        .expect("validated CORS origin must be a valid header")
                })
                .collect::<Vec<_>>(),
        ),
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE])
        .max_age(std::time::Duration::from_secs(600))
}
