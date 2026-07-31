use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU32,
    time::Duration,
};

use anyhow::{Context, bail};
use ipnet::IpNet;
use url::Url;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_LOG_FILTER: &str = "condensr_api=info,info";
const DEFAULT_MAX_REQUEST_BODY_BYTES: usize = 16 * 1024;
const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;
const DEFAULT_REQUEST_TIMEOUT_SECONDS: u64 = 30;
const MAX_REQUEST_TIMEOUT_SECONDS: u64 = 300;
const DEFAULT_RATE_LIMIT_WINDOW_SECONDS: u64 = 60;
const DEFAULT_RATE_LIMIT_BURST: u32 = 10;

pub struct Config {
    pub database_url: String,
    pub base_url: String,
    pub app_port: u16,
    pub log_filter: String,
    pub http: HttpConfig,
}

#[derive(Clone, Debug)]
pub struct HttpConfig {
    pub cors: Option<CorsConfig>,
    pub shorten_rate_limit: Option<RateLimitConfig>,
    pub max_request_body_bytes: usize,
    pub request_timeout: Duration,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            cors: None,
            shorten_rate_limit: None,
            max_request_body_bytes: DEFAULT_MAX_REQUEST_BODY_BYTES,
            request_timeout: Duration::from_secs(
                DEFAULT_REQUEST_TIMEOUT_SECONDS,
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CorsConfig {
    Any,
    Origins(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub requests: NonZeroU32,
    pub window: Duration,
    pub burst: NonZeroU32,
    pub trusted_proxy_cidrs: Vec<IpNet>,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        Self::from_environment(std::env::vars().collect())
    }

    fn from_environment(
        environment: HashMap<String, String>,
    ) -> Result<Self, anyhow::Error> {
        let database_url = required(&environment, "DATABASE_URL")?;
        validate_database_url(&database_url)?;
        let base_url =
            normalize_base_url(&required(&environment, "BASE_URL")?)?;
        let app_port = parse_port(environment.get("PORT"))?;
        let log_filter = environment
            .get("RUST_LOG")
            .cloned()
            .unwrap_or_else(|| DEFAULT_LOG_FILTER.to_string());
        let http = HttpConfig::from_environment(&environment)?;

        Ok(Self {
            database_url,
            base_url,
            app_port,
            log_filter,
            http,
        })
    }
}

impl HttpConfig {
    fn from_environment(
        environment: &HashMap<String, String>,
    ) -> Result<Self, anyhow::Error> {
        let max_request_body_bytes = parse_positive(
            environment.get("MAX_REQUEST_BODY_BYTES"),
            "MAX_REQUEST_BODY_BYTES",
            DEFAULT_MAX_REQUEST_BODY_BYTES,
        )?;
        if max_request_body_bytes > MAX_REQUEST_BODY_BYTES {
            bail!("MAX_REQUEST_BODY_BYTES must not exceed 1048576");
        }
        let request_timeout_seconds = parse_positive(
            environment.get("REQUEST_TIMEOUT_SECONDS"),
            "REQUEST_TIMEOUT_SECONDS",
            DEFAULT_REQUEST_TIMEOUT_SECONDS,
        )?;
        if request_timeout_seconds > MAX_REQUEST_TIMEOUT_SECONDS {
            bail!("REQUEST_TIMEOUT_SECONDS must not exceed 300");
        }

        Ok(Self {
            cors: parse_cors(environment.get("CORS_ALLOWED_ORIGINS"))?,
            shorten_rate_limit: parse_rate_limit(environment)?,
            max_request_body_bytes,
            request_timeout: Duration::from_secs(request_timeout_seconds),
        })
    }
}

fn required(
    environment: &HashMap<String, String>,
    key: &'static str,
) -> Result<String, anyhow::Error> {
    let value = environment
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .with_context(|| format!("{key} must be set"))?;
    Ok(value)
}

fn validate_database_url(database_url: &str) -> Result<(), anyhow::Error> {
    let parsed = Url::parse(database_url).context(
        "DATABASE_URL must be a valid postgres:// or postgresql:// URL",
    )?;
    if !matches!(parsed.scheme(), "postgres" | "postgresql") {
        bail!("DATABASE_URL scheme must be postgres or postgresql");
    }
    if parsed.host_str().is_none() {
        bail!("DATABASE_URL must include a host");
    }
    if parsed.path().trim_matches('/').is_empty() {
        bail!("DATABASE_URL must name an existing database");
    }
    Ok(())
}

fn normalize_base_url(base_url: &str) -> Result<String, anyhow::Error> {
    let base_url = base_url.trim();
    let parsed = Url::parse(base_url)
        .context("BASE_URL must be a valid HTTP or HTTPS origin")?;
    validate_http_origin(&parsed, "BASE_URL")?;
    Ok(base_url.trim_end_matches('/').to_string())
}

fn validate_http_origin(parsed: &Url, key: &str) -> Result<(), anyhow::Error> {
    if !matches!(parsed.scheme(), "http" | "https") {
        bail!("{key} scheme must be http or https");
    }
    if parsed.host_str().is_none() {
        bail!("{key} must include a host");
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        bail!("{key} must not include credentials");
    }
    if !matches!(parsed.path(), "" | "/")
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        bail!("{key} must be an origin without a path, query, or fragment");
    }
    Ok(())
}

fn parse_cors(
    value: Option<&String>,
) -> Result<Option<CorsConfig>, anyhow::Error> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        bail!("CORS_ALLOWED_ORIGINS must not be empty when set");
    }
    if value == "*" {
        return Ok(Some(CorsConfig::Any));
    }

    let raw_origins: Vec<_> = value.split(',').map(str::trim).collect();
    if raw_origins
        .iter()
        .any(|origin| origin.is_empty() || *origin == "*")
    {
        bail!(
            "CORS_ALLOWED_ORIGINS must contain exact origins, or '*' by itself"
        );
    }

    let mut seen = HashSet::new();
    let mut origins = Vec::new();
    for raw_origin in raw_origins {
        let parsed = Url::parse(raw_origin)
            .with_context(|| format!("invalid CORS origin: {raw_origin}"))?;
        validate_http_origin(&parsed, "CORS_ALLOWED_ORIGINS")?;
        let origin = parsed.origin().ascii_serialization();
        if seen.insert(origin.clone()) {
            origins.push(origin);
        }
    }

    Ok(Some(CorsConfig::Origins(origins)))
}

fn parse_rate_limit(
    environment: &HashMap<String, String>,
) -> Result<Option<RateLimitConfig>, anyhow::Error> {
    const DEPENDENT_KEYS: [&str; 3] = [
        "SHORTEN_RATE_LIMIT_WINDOW_SECONDS",
        "SHORTEN_RATE_LIMIT_BURST",
        "RATE_LIMIT_TRUSTED_PROXY_CIDRS",
    ];

    let Some(raw_requests) = environment.get("SHORTEN_RATE_LIMIT_REQUESTS")
    else {
        if let Some(key) = DEPENDENT_KEYS
            .iter()
            .find(|key| environment.contains_key(**key))
        {
            bail!(
                "{key} requires SHORTEN_RATE_LIMIT_REQUESTS to enable throttling"
            );
        }
        return Ok(None);
    };

    let requests =
        parse_non_zero_u32(raw_requests, "SHORTEN_RATE_LIMIT_REQUESTS")?;
    let window_seconds = parse_positive(
        environment.get("SHORTEN_RATE_LIMIT_WINDOW_SECONDS"),
        "SHORTEN_RATE_LIMIT_WINDOW_SECONDS",
        DEFAULT_RATE_LIMIT_WINDOW_SECONDS,
    )?;
    let default_burst = requests.get().min(DEFAULT_RATE_LIMIT_BURST);
    let burst = match environment.get("SHORTEN_RATE_LIMIT_BURST") {
        Some(value) => parse_non_zero_u32(value, "SHORTEN_RATE_LIMIT_BURST")?,
        None => NonZeroU32::new(default_burst).unwrap(),
    };
    let trusted_proxy_cidrs =
        parse_cidrs(environment.get("RATE_LIMIT_TRUSTED_PROXY_CIDRS"))?;

    let window = Duration::from_secs(window_seconds);
    if window.as_nanos() / u128::from(requests.get()) == 0 {
        bail!(
            "SHORTEN_RATE_LIMIT_REQUESTS is too large for SHORTEN_RATE_LIMIT_WINDOW_SECONDS"
        );
    }

    Ok(Some(RateLimitConfig {
        requests,
        window,
        burst,
        trusted_proxy_cidrs,
    }))
}

fn parse_cidrs(value: Option<&String>) -> Result<Vec<IpNet>, anyhow::Error> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.trim().is_empty() {
        bail!("RATE_LIMIT_TRUSTED_PROXY_CIDRS must not be empty when set");
    }
    value
        .split(',')
        .map(|value| {
            let value = value.trim();
            value.parse::<IpNet>().with_context(|| {
                format!(
                    "invalid CIDR in RATE_LIMIT_TRUSTED_PROXY_CIDRS: {value}"
                )
            })
        })
        .collect()
}

fn parse_port(value: Option<&String>) -> Result<u16, anyhow::Error> {
    let Some(value) = value else {
        return Ok(DEFAULT_PORT);
    };
    let port = value
        .parse::<u16>()
        .context("PORT must be an integer from 1 to 65535")?;
    if port == 0 {
        bail!("PORT must be an integer from 1 to 65535");
    }
    Ok(port)
}

fn parse_positive<T>(
    value: Option<&String>,
    key: &str,
    default: T,
) -> Result<T, anyhow::Error>
where
    T: std::str::FromStr + PartialEq + Default + Copy,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let Some(value) = value else {
        return Ok(default);
    };
    let parsed = value
        .parse::<T>()
        .with_context(|| format!("{key} must be a positive integer"))?;
    if parsed == T::default() {
        bail!("{key} must be a positive integer");
    }
    Ok(parsed)
}

fn parse_non_zero_u32(
    value: &str,
    key: &str,
) -> Result<NonZeroU32, anyhow::Error> {
    value
        .parse::<NonZeroU32>()
        .with_context(|| format!("{key} must be a positive integer"))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use super::{Config, CorsConfig};

    fn valid_environment() -> HashMap<String, String> {
        HashMap::from([
            (
                "DATABASE_URL".to_string(),
                "postgres://user:password@db.example/condensr?sslmode=require"
                    .to_string(),
            ),
            ("BASE_URL".to_string(), "https://short.example".to_string()),
        ])
    }

    #[test]
    fn loads_direct_environment_with_secure_defaults() {
        let config = Config::from_environment(valid_environment()).unwrap();

        assert_eq!(config.app_port, 8080);
        assert_eq!(config.base_url, "https://short.example");
        assert_eq!(config.log_filter, "condensr_api=info,info");
        assert!(config.http.cors.is_none());
        assert!(config.http.shorten_rate_limit.is_none());
        assert_eq!(config.http.max_request_body_bytes, 16 * 1024);
        assert_eq!(config.http.request_timeout, Duration::from_secs(30));
    }

    #[test]
    fn requires_database_and_base_urls() {
        for missing in ["DATABASE_URL", "BASE_URL"] {
            let mut environment = valid_environment();
            environment.remove(missing);
            assert!(Config::from_environment(environment).is_err());
        }
    }

    #[test]
    fn validates_database_url() {
        for invalid in [
            "mysql://db.example/condensr",
            "postgres:///condensr",
            "postgres://db.example",
            "not-a-url",
        ] {
            let mut environment = valid_environment();
            environment.insert("DATABASE_URL".to_string(), invalid.to_string());
            assert!(Config::from_environment(environment).is_err());
        }

        let mut environment = valid_environment();
        environment.insert(
            "DATABASE_URL".to_string(),
            "postgresql://db.example/condensr".to_string(),
        );
        assert!(Config::from_environment(environment).is_ok());
    }

    #[test]
    fn validates_and_normalizes_base_url() {
        for invalid in [
            "ftp://short.example",
            "https://user@short.example",
            "https://short.example/path",
            "https://short.example?query=1",
            "https://short.example#fragment",
            "not-a-url",
        ] {
            let mut environment = valid_environment();
            environment.insert("BASE_URL".to_string(), invalid.to_string());
            assert!(Config::from_environment(environment).is_err());
        }

        let mut environment = valid_environment();
        environment.insert(
            "BASE_URL".to_string(),
            "https://short.example/".to_string(),
        );
        let config = Config::from_environment(environment).unwrap();
        assert_eq!(config.base_url, "https://short.example");
    }

    #[test]
    fn validates_explicit_port() {
        for invalid in ["", "0", "65536", "not-a-port"] {
            let mut environment = valid_environment();
            environment.insert("PORT".to_string(), invalid.to_string());
            assert!(Config::from_environment(environment).is_err());
        }

        let mut environment = valid_environment();
        environment.insert("PORT".to_string(), "65535".to_string());
        assert_eq!(
            Config::from_environment(environment).unwrap().app_port,
            65535
        );
    }

    #[test]
    fn parses_exact_and_public_cors_configuration() {
        let mut environment = valid_environment();
        environment.insert(
            "CORS_ALLOWED_ORIGINS".to_string(),
            "https://App.Example:443, http://localhost:3000, https://app.example"
                .to_string(),
        );
        assert_eq!(
            Config::from_environment(environment).unwrap().http.cors,
            Some(CorsConfig::Origins(vec![
                "https://app.example".to_string(),
                "http://localhost:3000".to_string(),
            ]))
        );

        let mut environment = valid_environment();
        environment.insert("CORS_ALLOWED_ORIGINS".to_string(), "*".to_string());
        assert_eq!(
            Config::from_environment(environment).unwrap().http.cors,
            Some(CorsConfig::Any)
        );
    }

    #[test]
    fn rejects_malformed_cors_configuration() {
        for invalid in [
            "",
            "* , https://app.example",
            "https://app.example/path",
            "ftp://app.example",
            "https://user@app.example",
            "not-an-origin",
        ] {
            let mut environment = valid_environment();
            environment.insert(
                "CORS_ALLOWED_ORIGINS".to_string(),
                invalid.to_string(),
            );
            assert!(Config::from_environment(environment).is_err());
        }
    }

    #[test]
    fn parses_rate_limit_and_its_defaults() {
        let mut environment = valid_environment();
        environment.insert(
            "SHORTEN_RATE_LIMIT_REQUESTS".to_string(),
            "60".to_string(),
        );
        let rate = Config::from_environment(environment)
            .unwrap()
            .http
            .shorten_rate_limit
            .unwrap();
        assert_eq!(rate.requests.get(), 60);
        assert_eq!(rate.window, Duration::from_secs(60));
        assert_eq!(rate.burst.get(), 10);
        assert!(rate.trusted_proxy_cidrs.is_empty());

        let mut environment = valid_environment();
        environment
            .insert("SHORTEN_RATE_LIMIT_REQUESTS".to_string(), "5".to_string());
        assert_eq!(
            Config::from_environment(environment)
                .unwrap()
                .http
                .shorten_rate_limit
                .unwrap()
                .burst
                .get(),
            5
        );
    }

    #[test]
    fn parses_rate_limit_overrides_and_trusted_proxies() {
        let mut environment = valid_environment();
        environment.extend([
            ("SHORTEN_RATE_LIMIT_REQUESTS".to_string(), "12".to_string()),
            (
                "SHORTEN_RATE_LIMIT_WINDOW_SECONDS".to_string(),
                "30".to_string(),
            ),
            ("SHORTEN_RATE_LIMIT_BURST".to_string(), "3".to_string()),
            (
                "RATE_LIMIT_TRUSTED_PROXY_CIDRS".to_string(),
                "10.0.0.0/8, 2001:db8::/32".to_string(),
            ),
            ("MAX_REQUEST_BODY_BYTES".to_string(), "4096".to_string()),
            ("REQUEST_TIMEOUT_SECONDS".to_string(), "5".to_string()),
        ]);
        let http = Config::from_environment(environment).unwrap().http;
        let rate = http.shorten_rate_limit.unwrap();
        assert_eq!(rate.requests.get(), 12);
        assert_eq!(rate.window, Duration::from_secs(30));
        assert_eq!(rate.burst.get(), 3);
        assert_eq!(rate.trusted_proxy_cidrs.len(), 2);
        assert_eq!(http.max_request_body_bytes, 4096);
        assert_eq!(http.request_timeout, Duration::from_secs(5));
    }

    #[test]
    fn rejects_invalid_or_incomplete_security_settings() {
        for (key, value) in [
            ("SHORTEN_RATE_LIMIT_REQUESTS", "0"),
            ("SHORTEN_RATE_LIMIT_REQUESTS", "nope"),
            ("SHORTEN_RATE_LIMIT_WINDOW_SECONDS", "0"),
            ("SHORTEN_RATE_LIMIT_BURST", "0"),
            ("RATE_LIMIT_TRUSTED_PROXY_CIDRS", "not-a-cidr"),
            ("MAX_REQUEST_BODY_BYTES", "0"),
            ("REQUEST_TIMEOUT_SECONDS", "0"),
        ] {
            let mut environment = valid_environment();
            environment.insert(key.to_string(), value.to_string());
            if key != "SHORTEN_RATE_LIMIT_REQUESTS"
                && !matches!(
                    key,
                    "MAX_REQUEST_BODY_BYTES" | "REQUEST_TIMEOUT_SECONDS"
                )
            {
                environment.insert(
                    "SHORTEN_RATE_LIMIT_REQUESTS".to_string(),
                    "10".to_string(),
                );
            }
            assert!(Config::from_environment(environment).is_err(), "{key}");
        }

        for dependent in [
            "SHORTEN_RATE_LIMIT_WINDOW_SECONDS",
            "SHORTEN_RATE_LIMIT_BURST",
            "RATE_LIMIT_TRUSTED_PROXY_CIDRS",
        ] {
            let mut environment = valid_environment();
            environment.insert(dependent.to_string(), "1".to_string());
            assert!(
                Config::from_environment(environment).is_err(),
                "{dependent}"
            );
        }

        for (key, value) in [
            ("MAX_REQUEST_BODY_BYTES", "1048577"),
            ("REQUEST_TIMEOUT_SECONDS", "301"),
        ] {
            let mut environment = valid_environment();
            environment.insert(key.to_string(), value.to_string());
            assert!(Config::from_environment(environment).is_err(), "{key}");
        }
    }
}
