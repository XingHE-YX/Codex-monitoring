use std::{
    collections::HashMap,
    fmt,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    time::Duration,
};

use chrono::NaiveTime;
use chrono_tz::Tz;
use thiserror::Error;
use url::Url;
use zeroize::Zeroize;

const REQUIRED_MODEL_NAME: &str = "gpt-5.6-terra";
const DEFAULT_DATABASE_URL: &str = "sqlite://codex-reset-watch.db";
const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:3000";
const DEFAULT_TIMEZONE: &str = "Asia/Shanghai";
const DEFAULT_MONITOR_START: &str = "08:00";
const DEFAULT_MONITOR_END: &str = "23:00";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub base_url: Option<Url>,
    pub api_key: Option<SecretString>,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct GmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: Option<String>,
    pub app_password: Option<SecretString>,
    pub recipient: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FcmConfig {
    pub project_id: Option<String>,
    pub service_account_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    pub timezone: Tz,
    pub monitor_start: NaiveTime,
    pub monitor_end: NaiveTime,
}

#[derive(Debug, Clone)]
pub struct PairingLimits {
    pub code_length: usize,
    pub code_ttl: Duration,
    pub max_attempts: u32,
    pub max_active_devices: u32,
    pub manual_check_cooldown: Duration,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub environment: Environment,
    pub database_url: String,
    pub bind_address: SocketAddr,
    pub fixed_public_ip: Option<IpAddr>,
    pub model: ModelConfig,
    pub gmail: GmailConfig,
    pub fcm: FcmConfig,
    pub schedule: ScheduleConfig,
    pub retention_days: u32,
    pub pairing: PairingLimits,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing production configuration: {names:?}")]
    MissingProductionValues { names: Vec<String> },
    #[error("invalid value for {name}: {reason}")]
    InvalidValue { name: String, reason: String },
    #[error("could not load local environment file: {reason}")]
    Dotenv { reason: String },
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        if let Err(error) = dotenvy::dotenv()
            && !matches!(&error, dotenvy::Error::Io(io_error) if io_error.kind() == std::io::ErrorKind::NotFound)
        {
            return Err(ConfigError::Dotenv {
                reason: error.to_string(),
            });
        }

        let values = std::env::vars().collect::<HashMap<_, _>>();
        Self::from_map(values)
    }

    pub fn from_map<I, K, V>(values: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values = values
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect::<HashMap<_, _>>();

        let environment = match value_or(&values, "APP_ENV", "development") {
            "development" => Environment::Development,
            "production" => Environment::Production,
            _ => return Err(invalid("APP_ENV", "must equal development or production")),
        };

        let model_name = value_or(&values, "MODEL_NAME", REQUIRED_MODEL_NAME).to_string();
        if model_name != REQUIRED_MODEL_NAME {
            return Err(invalid("MODEL_NAME", "must equal gpt-5.6-terra"));
        }

        if environment == Environment::Production {
            let mut names = [
                "FCM_PROJECT_ID",
                "FCM_SERVICE_ACCOUNT_PATH",
                "FIXED_PUBLIC_IP",
                "GMAIL_APP_PASSWORD",
                "GMAIL_RECIPIENT",
                "GMAIL_USERNAME",
                "MODEL_API_KEY",
                "MODEL_BASE_URL",
            ]
            .into_iter()
            .filter(|name| optional(&values, name).is_none())
            .map(str::to_string)
            .collect::<Vec<_>>();
            names.sort();

            if !names.is_empty() {
                return Err(ConfigError::MissingProductionValues { names });
            }
        }

        let database_url = value_or(&values, "DATABASE_URL", DEFAULT_DATABASE_URL).to_string();
        let bind_address = parse(
            "BIND_ADDRESS",
            value_or(&values, "BIND_ADDRESS", DEFAULT_BIND_ADDRESS),
        )?;
        let fixed_public_ip = optional(&values, "FIXED_PUBLIC_IP")
            .map(|value| parse("FIXED_PUBLIC_IP", value))
            .transpose()?;

        let model = ModelConfig {
            base_url: optional(&values, "MODEL_BASE_URL")
                .map(|value| parse_url("MODEL_BASE_URL", value))
                .transpose()?,
            api_key: optional(&values, "MODEL_API_KEY").map(SecretString::new),
            name: model_name,
        };

        let gmail = GmailConfig {
            smtp_host: value_or(&values, "GMAIL_SMTP_HOST", "smtp.gmail.com").to_string(),
            smtp_port: parse(
                "GMAIL_SMTP_PORT",
                value_or(&values, "GMAIL_SMTP_PORT", "465"),
            )?,
            username: optional(&values, "GMAIL_USERNAME").map(str::to_string),
            app_password: optional(&values, "GMAIL_APP_PASSWORD").map(SecretString::new),
            recipient: optional(&values, "GMAIL_RECIPIENT").map(str::to_string),
        };

        let fcm = FcmConfig {
            project_id: optional(&values, "FCM_PROJECT_ID").map(str::to_string),
            service_account_path: optional(&values, "FCM_SERVICE_ACCOUNT_PATH").map(PathBuf::from),
        };

        let schedule = ScheduleConfig {
            timezone: parse("TIMEZONE", value_or(&values, "TIMEZONE", DEFAULT_TIMEZONE))?,
            monitor_start: parse_time(
                "MONITOR_START",
                value_or(&values, "MONITOR_START", DEFAULT_MONITOR_START),
            )?,
            monitor_end: parse_time(
                "MONITOR_END",
                value_or(&values, "MONITOR_END", DEFAULT_MONITOR_END),
            )?,
        };

        let retention_days = parse_positive(
            "HISTORY_RETENTION_DAYS",
            value_or(&values, "HISTORY_RETENTION_DAYS", "30"),
        )?;
        let pairing = PairingLimits {
            code_length: parse_positive(
                "PAIRING_CODE_LENGTH",
                value_or(&values, "PAIRING_CODE_LENGTH", "8"),
            )?,
            code_ttl: Duration::from_secs(
                u64::from(parse_positive::<u32>(
                    "PAIRING_CODE_TTL_MINUTES",
                    value_or(&values, "PAIRING_CODE_TTL_MINUTES", "10"),
                )?) * 60,
            ),
            max_attempts: parse_positive(
                "PAIRING_MAX_ATTEMPTS",
                value_or(&values, "PAIRING_MAX_ATTEMPTS", "5"),
            )?,
            max_active_devices: parse_positive(
                "MAX_ACTIVE_DEVICES",
                value_or(&values, "MAX_ACTIVE_DEVICES", "1"),
            )?,
            manual_check_cooldown: Duration::from_secs(
                u64::from(parse_positive::<u32>(
                    "MANUAL_CHECK_COOLDOWN_MINUTES",
                    value_or(&values, "MANUAL_CHECK_COOLDOWN_MINUTES", "5"),
                )?) * 60,
            ),
        };

        Ok(Self {
            environment,
            database_url,
            bind_address,
            fixed_public_ip,
            model,
            gmail,
            fcm,
            schedule,
            retention_days,
            pairing,
        })
    }
}

fn optional<'a>(values: &'a HashMap<String, String>, name: &str) -> Option<&'a str> {
    values
        .get(name)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn value_or<'a>(values: &'a HashMap<String, String>, name: &str, default: &'a str) -> &'a str {
    optional(values, name).unwrap_or(default)
}

fn parse<T>(name: &str, value: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    value
        .parse()
        .map_err(|_| invalid(name, "has an invalid format"))
}

fn parse_positive<T>(name: &str, value: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr + Default + PartialOrd,
{
    let parsed = parse(name, value)?;
    if parsed <= T::default() {
        return Err(invalid(name, "must be greater than zero"));
    }
    Ok(parsed)
}

fn parse_url(name: &str, value: &str) -> Result<Url, ConfigError> {
    let url = Url::parse(value).map_err(|_| invalid(name, "must be an absolute URL"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(invalid(name, "must use http or https"));
    }
    Ok(url)
}

fn parse_time(name: &str, value: &str) -> Result<NaiveTime, ConfigError> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .map_err(|_| invalid(name, "must use HH:MM in 24-hour time"))
}

fn invalid(name: &str, reason: &str) -> ConfigError {
    ConfigError::InvalidValue {
        name: name.to_string(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveTime;
    use chrono_tz::Asia::Shanghai;

    use super::{Config, ConfigError, SecretString};

    #[test]
    fn production_rejects_missing_required_values() {
        let error = Config::from_map([("APP_ENV", "production")]).unwrap_err();

        let ConfigError::MissingProductionValues { names } = error else {
            panic!("expected missing production values");
        };

        assert_eq!(
            names,
            vec![
                "FCM_PROJECT_ID",
                "FCM_SERVICE_ACCOUNT_PATH",
                "FIXED_PUBLIC_IP",
                "GMAIL_APP_PASSWORD",
                "GMAIL_RECIPIENT",
                "GMAIL_USERNAME",
                "MODEL_API_KEY",
                "MODEL_BASE_URL",
            ]
        );
    }

    #[test]
    fn development_uses_documented_schedule_and_retention_defaults() {
        let config = Config::from_map(HashMap::<String, String>::new()).unwrap();

        assert_eq!(config.schedule.timezone, Shanghai);
        assert_eq!(
            config.schedule.monitor_start,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            config.schedule.monitor_end,
            NaiveTime::from_hms_opt(23, 0, 0).unwrap()
        );
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.pairing.code_length, 8);
        assert_eq!(config.pairing.code_ttl.as_secs(), 600);
        assert_eq!(config.pairing.max_attempts, 5);
        assert_eq!(config.pairing.max_active_devices, 1);
        assert_eq!(config.pairing.manual_check_cooldown.as_secs(), 300);
    }

    #[test]
    fn model_name_must_remain_locked() {
        let error = Config::from_map([("MODEL_NAME", "another-model")]).unwrap_err();

        assert_eq!(
            error,
            ConfigError::InvalidValue {
                name: "MODEL_NAME".to_string(),
                reason: "must equal gpt-5.6-terra".to_string(),
            }
        );
    }

    #[test]
    fn secret_debug_output_is_redacted() {
        let secret = SecretString::new("never-print-this");

        assert_eq!(format!("{secret:?}"), "[REDACTED]");
        assert!(!format!("{secret:?}").contains(secret.expose()));
    }
}
