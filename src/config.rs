// flag > env > toml 우선순위로 Redmine 설정을 머지한다.
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
struct FileConfig {
    server_url: Option<String>,
    api_token: Option<String>,
    #[serde(default)]
    custom_fields: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server_url: String,
    pub api_token: String,
    pub cf_aliases: HashMap<String, u64>,
}

pub struct CliOverrides {
    pub server_url: Option<String>,
    pub api_token: Option<String>,
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing server URL (set --server-url, REDMINE_URL, or server_url in config.toml)")]
    MissingServer,
    #[error("missing API token (set --api-token, REDMINE_API_TOKEN, or api_token in config.toml)")]
    MissingToken,
    #[error("failed to read config file at {0}: {1}")]
    Io(PathBuf, String),
    #[error("failed to parse config file at {0}: {1}")]
    Parse(PathBuf, String),
}

pub fn resolve(overrides: &CliOverrides) -> Result<Config, ConfigError> {
    let file = load_file(overrides.config_path.clone())?;
    let server_url = overrides
        .server_url
        .clone()
        .or_else(|| std::env::var("REDMINE_URL").ok())
        .or(file.server_url)
        .filter(|s| !s.is_empty())
        .ok_or(ConfigError::MissingServer)?;
    let api_token = overrides
        .api_token
        .clone()
        .or_else(|| std::env::var("REDMINE_API_TOKEN").ok())
        .or(file.api_token)
        .filter(|s| !s.is_empty())
        .ok_or(ConfigError::MissingToken)?;
    Ok(Config {
        server_url,
        api_token,
        cf_aliases: file.custom_fields,
    })
}

fn default_config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "redmine-cli")
        .map(|p| p.config_dir().join("config.toml"))
}

fn load_file(explicit: Option<PathBuf>) -> Result<FileConfig, ConfigError> {
    let path = explicit.or_else(default_config_path);
    let Some(path) = path else {
        return Ok(FileConfig::default());
    };
    if !path.exists() {
        return Ok(FileConfig::default());
    }
    let text =
        std::fs::read_to_string(&path).map_err(|e| ConfigError::Io(path.clone(), e.to_string()))?;
    toml::from_str::<FileConfig>(&text).map_err(|e| ConfigError::Parse(path, e.to_string()))
}

/// `--custom-field` 입력(`id=value` 또는 `alias=value`)을 (cf_id, value) 로 분해한다.
pub fn parse_custom_field(
    spec: &str,
    aliases: &HashMap<String, u64>,
) -> Result<(u64, String), String> {
    let (k, v) = spec
        .split_once('=')
        .ok_or_else(|| format!("--custom-field expects id=value, got: {spec}"))?;
    let id = match k.parse::<u64>() {
        Ok(n) => n,
        Err(_) => *aliases
            .get(k)
            .ok_or_else(|| format!("unknown custom field alias: {k}"))?,
    };
    Ok((id, v.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_custom_field_numeric() {
        let aliases = HashMap::new();
        assert_eq!(
            parse_custom_field("7=Dev", &aliases).unwrap(),
            (7, "Dev".into())
        );
    }

    #[test]
    fn parse_custom_field_alias() {
        let mut aliases = HashMap::new();
        aliases.insert("state".to_string(), 7);
        assert_eq!(
            parse_custom_field("state=Dev", &aliases).unwrap(),
            (7, "Dev".into())
        );
    }

    #[test]
    fn parse_custom_field_unknown_alias() {
        let aliases = HashMap::new();
        assert!(parse_custom_field("zz=Dev", &aliases).is_err());
    }

    #[test]
    fn parse_custom_field_missing_equals() {
        let aliases = HashMap::new();
        assert!(parse_custom_field("Dev", &aliases).is_err());
    }
}
