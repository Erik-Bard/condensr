use std::collections::HashMap;

use anyhow::{Context, bail};
use url::Url;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_LOG_FILTER: &str = "condensr_api=info,info";

pub struct Config {
    pub database_url: String,
    pub base_url: String,
    pub app_port: u16,
    pub log_filter: String,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        Self::from_environment(std::env::vars().collect())
    }

    fn from_environment(
        process_environment: HashMap<String, String>,
    ) -> Result<Self, anyhow::Error> {
        if process_environment.contains_key("CONDENSR_ENV_FILE") {
            bail!(
                "CONDENSR_ENV_FILE is not supported; inject variables directly or use Docker --env-file"
            );
        }

        let environment = process_environment;
        let database_url = required(&environment, "DATABASE_URL")?;
        validate_database_url(&database_url)?;
        let base_url =
            normalize_base_url(&required(&environment, "BASE_URL")?)?;
        let app_port = parse_port(environment.get("PORT"))?;
        let log_filter = environment
            .get("RUST_LOG")
            .cloned()
            .unwrap_or_else(|| DEFAULT_LOG_FILTER.to_string());

        Ok(Self {
            database_url,
            base_url,
            app_port,
            log_filter,
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
    let parsed = Url::parse(base_url)
        .context("BASE_URL must be a valid HTTP or HTTPS origin")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        bail!("BASE_URL scheme must be http or https");
    }
    if parsed.host_str().is_none() {
        bail!("BASE_URL must include a host");
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        bail!("BASE_URL must not include credentials");
    }
    if !matches!(parsed.path(), "" | "/")
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        bail!("BASE_URL must be an origin without a path, query, or fragment");
    }
    Ok(base_url.trim_end_matches('/').to_string())
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Config;

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
    fn loads_direct_environment_with_defaults() {
        let config = Config::from_environment(valid_environment()).unwrap();

        assert_eq!(config.app_port, 8080);
        assert_eq!(config.base_url, "https://short.example");
        assert_eq!(config.log_filter, "condensr_api=info,info");
    }

    #[test]
    fn rejects_retired_application_environment_file_setting() {
        let mut environment = valid_environment();
        environment.insert(
            "CONDENSR_ENV_FILE".to_string(),
            "condensr.env".to_string(),
        );

        match Config::from_environment(environment) {
            Err(error) => {
                assert!(error.to_string().contains("Docker --env-file"));
            }
            Ok(_) => panic!("CONDENSR_ENV_FILE must be rejected"),
        }
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
}
