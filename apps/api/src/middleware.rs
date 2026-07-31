use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderValue, header::RETRY_AFTER},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    DefaultKeyedRateLimiter, Quota, RateLimiter,
    clock::{Clock, DefaultClock},
};
use ipnet::IpNet;

use crate::{config::RateLimitConfig, models::error::AppError};

#[derive(Clone)]
pub struct ShortenRateLimit {
    limiter: Arc<DefaultKeyedRateLimiter<IpAddr>>,
    trusted_proxy_cidrs: Arc<[IpNet]>,
    cleanup_interval_seconds: u64,
    last_cleanup: Arc<AtomicU64>,
}

impl ShortenRateLimit {
    pub fn new(config: &RateLimitConfig) -> Self {
        let replenish_interval = config.window / config.requests.get();
        let quota = Quota::with_period(replenish_interval)
            .expect("validated rate-limit period must be non-zero")
            .allow_burst(config.burst);
        Self {
            limiter: Arc::new(RateLimiter::keyed(quota)),
            trusted_proxy_cidrs: config.trusted_proxy_cidrs.clone().into(),
            cleanup_interval_seconds: config.window.as_secs().max(60),
            last_cleanup: Arc::new(AtomicU64::new(unix_seconds())),
        }
    }

    fn client_ip(&self, request: &Request) -> IpAddr {
        let peer = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|connect_info| connect_info.0.ip())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        if !self.is_trusted_proxy(peer) {
            return peer;
        }

        request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| self.forwarded_client_ip(value))
            .unwrap_or(peer)
    }

    fn forwarded_client_ip(&self, value: &str) -> Option<IpAddr> {
        let addresses = value
            .split(',')
            .map(str::trim)
            .map(str::parse::<IpAddr>)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        if addresses.is_empty() {
            return None;
        }

        addresses
            .iter()
            .rev()
            .copied()
            .find(|address| !self.is_trusted_proxy(*address))
    }

    fn is_trusted_proxy(&self, address: IpAddr) -> bool {
        self.trusted_proxy_cidrs
            .iter()
            .any(|network| network.contains(&address))
    }

    fn maybe_cleanup(&self) {
        let now = unix_seconds();
        let previous = self.last_cleanup.load(Ordering::Relaxed);
        if now.saturating_sub(previous) < self.cleanup_interval_seconds {
            return;
        }
        if self
            .last_cleanup
            .compare_exchange(
                previous,
                now,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            self.limiter.retain_recent();
            self.limiter.shrink_to_fit();
        }
    }
}

pub async fn enforce_shorten_rate_limit(
    State(rate_limit): State<ShortenRateLimit>,
    request: Request,
    next: Next,
) -> Response {
    rate_limit.maybe_cleanup();
    let client_ip = rate_limit.client_ip(&request);
    match rate_limit.limiter.check_key(&client_ip) {
        Ok(()) => next.run(request).await,
        Err(not_until) => {
            let wait = not_until.wait_time_from(DefaultClock::default().now());
            let retry_after = wait
                .as_secs()
                .saturating_add(u64::from(wait.subsec_nanos() > 0))
                .max(1);
            let mut response = AppError::too_many_requests().into_response();
            response.headers_mut().insert(
                RETRY_AFTER,
                HeaderValue::from_str(&retry_after.to_string())
                    .expect("retry-after seconds must be a valid header value"),
            );
            response
        }
    }
}

pub async fn enforce_request_timeout(
    State(timeout): State<Duration>,
    request: Request,
    next: Next,
) -> Response {
    match tokio::time::timeout(timeout, next.run(request)).await {
        Ok(response) => response,
        Err(_) => AppError::request_timeout().into_response(),
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, SocketAddr},
        num::NonZeroU32,
        time::Duration,
    };

    use axum::{
        Router,
        body::Body,
        extract::ConnectInfo,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
    };
    use tower::ServiceExt;

    use super::{ShortenRateLimit, enforce_request_timeout};
    use crate::config::RateLimitConfig;

    fn rate_limit(trusted: &[&str]) -> ShortenRateLimit {
        ShortenRateLimit::new(&RateLimitConfig {
            requests: NonZeroU32::new(1).unwrap(),
            window: Duration::from_secs(60),
            burst: NonZeroU32::new(1).unwrap(),
            trusted_proxy_cidrs: trusted
                .iter()
                .map(|value| value.parse().unwrap())
                .collect(),
        })
    }

    fn request(peer: &str, forwarded_for: Option<&str>) -> Request<Body> {
        let mut request = Request::builder().uri("/");
        if let Some(forwarded_for) = forwarded_for {
            request = request.header("x-forwarded-for", forwarded_for);
        }
        let mut request = request.body(Body::empty()).unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
        request
    }

    #[test]
    fn ignores_forwarding_headers_from_untrusted_peers() {
        let limit = rate_limit(&["10.0.0.0/8"]);
        let request = request("203.0.113.10:1234", Some("198.51.100.1"));
        assert_eq!(
            limit.client_ip(&request),
            "203.0.113.10".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn walks_trusted_proxy_chain_from_right_to_left() {
        let limit = rate_limit(&["10.0.0.0/8"]);
        let request = request(
            "10.0.0.3:1234",
            Some("192.0.2.99, 198.51.100.5, 10.0.0.2"),
        );
        assert_eq!(
            limit.client_ip(&request),
            "198.51.100.5".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn malformed_forwarding_header_falls_back_to_peer() {
        let limit = rate_limit(&["10.0.0.0/8"]);
        let request = request("10.0.0.3:1234", Some("not-an-ip"));
        assert_eq!(
            limit.client_ip(&request),
            "10.0.0.3".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn all_trusted_forwarding_chain_falls_back_to_peer() {
        let limit = rate_limit(&["10.0.0.0/8"]);
        let request = request("10.0.0.3:1234", Some("10.0.0.1, 10.0.0.2"));
        assert_eq!(
            limit.client_ip(&request),
            "10.0.0.3".parse::<IpAddr>().unwrap()
        );
    }

    #[tokio::test]
    async fn timeout_uses_the_api_error_contract() {
        async fn slow() -> impl IntoResponse {
            tokio::time::sleep(Duration::from_secs(1)).await;
            StatusCode::OK
        }

        let app = Router::new().route("/", get(slow)).layer(
            middleware::from_fn_with_state(
                Duration::from_millis(1),
                enforce_request_timeout,
            ),
        );
        let response = app
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
    }

    #[tokio::test]
    async fn rate_limit_replenishes() {
        let limit = ShortenRateLimit::new(&RateLimitConfig {
            requests: NonZeroU32::new(1).unwrap(),
            window: Duration::from_millis(20),
            burst: NonZeroU32::new(1).unwrap(),
            trusted_proxy_cidrs: Vec::new(),
        });
        let client = "192.0.2.1".parse::<IpAddr>().unwrap();
        assert!(limit.limiter.check_key(&client).is_ok());
        assert!(limit.limiter.check_key(&client).is_err());
        tokio::time::sleep(Duration::from_millis(25)).await;
        assert!(limit.limiter.check_key(&client).is_ok());
    }
}
