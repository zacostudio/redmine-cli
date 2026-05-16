// flag > env > toml 우선순위로 Redmine 설정을 머지한다.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct FileConfig {
    pub server_url: Option<String>,
    pub api_token: Option<String>,
    #[serde(default)]
    pub custom_fields: HashMap<String, u64>,
}

impl std::fmt::Debug for FileConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileConfig")
            .field("server_url", &self.server_url)
            .field("api_token", &self.api_token.as_ref().map(|_| "<REDACTED>"))
            .field("custom_fields", &self.custom_fields)
            .finish()
    }
}

#[derive(Clone)]
pub struct Config {
    pub server_url: String,
    pub api_token: String,
    pub cf_aliases: HashMap<String, u64>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("server_url", &self.server_url)
            .field("api_token", &"<REDACTED>")
            .field("cf_aliases", &self.cf_aliases)
            .finish()
    }
}

pub struct CliOverrides {
    pub server_url: Option<String>,
    pub api_token: Option<String>,
    pub config_path: Option<PathBuf>,
}

impl std::fmt::Debug for CliOverrides {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliOverrides")
            .field("server_url", &self.server_url)
            .field("api_token", &self.api_token.as_ref().map(|_| "<REDACTED>"))
            .field("config_path", &self.config_path)
            .finish()
    }
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

/// 외부에서 사용 가능한 형태로 config.toml 경로를 해석한다.
pub fn resolve_path(explicit: Option<PathBuf>) -> Option<PathBuf> {
    explicit.or_else(default_config_path)
}

/// config.toml 을 읽어 들이거나 없으면 빈 구조체를 돌려준다.
pub fn load_or_empty(path: Option<PathBuf>) -> Result<FileConfig, ConfigError> {
    load_file(path)
}

/// FileConfig 를 config.toml 형식으로 저장한다. 부모 디렉터리는 필요 시 생성한다.
///
/// 보안: 토큰이 평문으로 저장되므로 Unix 에서는 0600 (소유자 r/w 만) 으로 생성한다.
/// `fs::write` 의 기본 0644 와 달리 OpenOptions 로 atomic 하게 모드를 지정해
/// write 와 chmod 사이의 race 를 차단한다. Windows 는 mode 인자가 무시된다.
pub fn save(path: &std::path::Path, file: &FileConfig) -> Result<(), ConfigError> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ConfigError::Io(parent.to_path_buf(), e.to_string()))?;
    }
    let text =
        toml::to_string(file).map_err(|e| ConfigError::Parse(path.to_path_buf(), e.to_string()))?;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts
        .open(path)
        .map_err(|e| ConfigError::Io(path.to_path_buf(), e.to_string()))?;
    f.write_all(text.as_bytes())
        .map_err(|e| ConfigError::Io(path.to_path_buf(), e.to_string()))?;
    // OpenOptions::mode 는 새 파일에만 적용된다. 0644 로 저장돼 있던 구버전 파일을
    // 그대로 갱신한 경우에도 0600 으로 강등시키기 위해 명시 chmod 한 번 더 호출한다.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| ConfigError::Io(path.to_path_buf(), e.to_string()))?;
    }
    Ok(())
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
