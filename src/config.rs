// flag > config.yml 우선순위로 Redmine 설정을 해석한다. 이름 붙은 서버를 여러 개 두고 고른다.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// config.toml 을 변환할 때 만들어지는 서버 이름.
pub const LEGACY_SERVER_NAME: &str = "default";

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct FileConfig {
    /// `--server` 가 없을 때 고를 서버 이름.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_server: Option<String>,
    /// 이름이 key 라 중복이 구조적으로 불가능하다. BTreeMap 이므로 저장 시 순서가 고정된다.
    #[serde(default)]
    pub servers: BTreeMap<String, ServerConfig>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct ServerConfig {
    /// 비어 있을 수 있다. config.toml 변환이 손실 없이 끝나도록 허용하고,
    /// 실제로 그 서버를 쓸 때 MissingServer 로 걸러낸다.
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub api_token: String,
    /// alias -> custom field id. 서버마다 id 가 다르므로 서버 단위로 둔다.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub custom_fields: BTreeMap<String, u64>,
}

impl std::fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerConfig")
            .field("url", &self.url)
            .field("api_token", &"<REDACTED>")
            .field("custom_fields", &self.custom_fields)
            .finish()
    }
}

impl std::fmt::Debug for FileConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileConfig")
            .field("default_server", &self.default_server)
            .field("servers", &self.servers)
            .finish()
    }
}

#[derive(Clone)]
pub struct Config {
    pub server_url: String,
    pub api_token: String,
    pub cf_aliases: BTreeMap<String, u64>,
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
    /// `--server <name>`: config.yml 의 어떤 서버를 쓸지.
    pub server: Option<String>,
    pub server_url: Option<String>,
    pub api_token: Option<String>,
    pub config_path: Option<PathBuf>,
}

impl std::fmt::Debug for CliOverrides {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliOverrides")
            .field("server", &self.server)
            .field("server_url", &self.server_url)
            .field("api_token", &self.api_token.as_ref().map(|_| "<REDACTED>"))
            .field("config_path", &self.config_path)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(
        "missing server URL (set url for the selected server in config.yml, or pass --server-url)"
    )]
    MissingServer,
    #[error("missing API token (set api_token for the selected server in config.yml, or pass --api-token)")]
    MissingToken,
    #[error("no Redmine server configured in {0} (add a servers: entry, or pass both --server-url and --api-token)")]
    NoServerConfigured(PathBuf),
    #[error("Redmine server '{name}' not found in config (available: {available})")]
    ServerNotFound { name: String, available: String },
    #[error("default_server '{name}' is not in servers (available: {available})")]
    DefaultServerNotFound { name: String, available: String },
    #[error("several servers configured and no default_server; pick one with --server <name> (available: {available})")]
    NoServerSelected { available: String },
    #[error("failed to read config file at {0}: {1}")]
    Io(PathBuf, String),
    #[error("failed to parse config file at {0}: {1}")]
    Parse(PathBuf, String),
}

fn names(file: &FileConfig) -> String {
    file.servers.keys().cloned().collect::<Vec<_>>().join(", ")
}

/// `--server` / `default_server` / 단일 서버 폴백 순으로 서버 이름을 고른다.
pub fn select_server_name(
    file: &FileConfig,
    requested: Option<&str>,
    path: &Path,
) -> Result<String, ConfigError> {
    if file.servers.is_empty() {
        return Err(ConfigError::NoServerConfigured(path.to_path_buf()));
    }
    if let Some(name) = requested {
        return match file.servers.contains_key(name) {
            true => Ok(name.to_string()),
            false => Err(ConfigError::ServerNotFound {
                name: name.to_string(),
                available: names(file),
            }),
        };
    }
    if let Some(name) = &file.default_server {
        return match file.servers.contains_key(name) {
            true => Ok(name.clone()),
            false => Err(ConfigError::DefaultServerNotFound {
                name: name.clone(),
                available: names(file),
            }),
        };
    }
    // 기본값이 없어도 서버가 하나뿐이면 고민할 여지가 없다.
    match file.servers.len() {
        1 => Ok(file.servers.keys().next().expect("len == 1").clone()),
        _ => Err(ConfigError::NoServerSelected {
            available: names(file),
        }),
    }
}

pub fn resolve(overrides: &CliOverrides) -> Result<Config, ConfigError> {
    // 자격증명이 flag 로 다 왔으면 설정 파일을 아예 열지 않는다. 파일을 읽는 것만으로
    // legacy 변환이 일어나므로, 파일과 무관한 호출이 사용자 설정을 건드리면 안 된다.
    if let Some(cfg) = ad_hoc(overrides) {
        return Ok(cfg);
    }
    let path = resolve_path(overrides.config_path.clone());
    let file = match &path {
        Some(p) => load_at(p)?,
        None => FileConfig::default(),
    };
    // 경로를 알 수 없는 환경(HOME 없음 등)에서도 에러 문구는 나와야 한다.
    let shown = path.unwrap_or_else(|| PathBuf::from(CONFIG_FILE_NAME));
    resolve_from(&file, overrides, &shown)
}

/// `--server` 없이 URL 과 토큰이 모두 flag 로 오면 설정 파일과 무관한 ad-hoc 서버다.
/// 이 경로가 있어야 CLI 한 줄 호출과 통합 테스트가 사용자 설정에 좌우되지 않는다.
fn ad_hoc(overrides: &CliOverrides) -> Option<Config> {
    if overrides.server.is_some() {
        return None;
    }
    let server_url = overrides.server_url.clone().filter(|s| !s.is_empty())?;
    let api_token = overrides.api_token.clone().filter(|s| !s.is_empty())?;
    Some(Config {
        server_url,
        api_token,
        cf_aliases: BTreeMap::new(),
    })
}

/// 파일 I/O 와 분리된 순수 해석부. 테스트가 여기를 직접 부른다.
pub fn resolve_from(
    file: &FileConfig,
    overrides: &CliOverrides,
    path: &Path,
) -> Result<Config, ConfigError> {
    if let Some(cfg) = ad_hoc(overrides) {
        return Ok(cfg);
    }
    let flag_url = overrides.server_url.clone().filter(|s| !s.is_empty());
    let flag_token = overrides.api_token.clone().filter(|s| !s.is_empty());

    let name = select_server_name(file, overrides.server.as_deref(), path)?;
    let server = &file.servers[&name];
    let server_url = flag_url.unwrap_or_else(|| server.url.clone());
    if server_url.is_empty() {
        return Err(ConfigError::MissingServer);
    }
    let api_token = flag_token.unwrap_or_else(|| server.api_token.clone());
    if api_token.is_empty() {
        return Err(ConfigError::MissingToken);
    }
    Ok(Config {
        server_url,
        api_token,
        cf_aliases: server.custom_fields.clone(),
    })
}

const CONFIG_FILE_NAME: &str = "config.yml";

fn default_config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "redmine-cli")
        .map(|p| p.config_dir().join(CONFIG_FILE_NAME))
}

fn load_file(explicit: Option<PathBuf>) -> Result<FileConfig, ConfigError> {
    match resolve_path(explicit) {
        Some(path) => load_at(&path),
        None => Ok(FileConfig::default()),
    }
}

fn load_at(path: &Path) -> Result<FileConfig, ConfigError> {
    if !path.exists() {
        // config.yml 이 없고 예전 config.toml 만 있으면 1회 변환한다.
        let legacy = path.with_extension("toml");
        if legacy.exists() {
            let text = std::fs::read_to_string(&legacy)
                .map_err(|e| ConfigError::Io(legacy.clone(), e.to_string()))?;
            let file = parse_legacy(&text, &legacy)?;
            save(path, &file)?;
            // stdout 은 JSON 전용이므로 안내는 stderr 로만 낸다.
            eprintln!(
                "migrated {} -> {} (the old file is left in place)",
                legacy.display(),
                path.display()
            );
            return Ok(file);
        }
        return Ok(FileConfig::default());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::Io(path.to_path_buf(), e.to_string()))?;
    serde_norway::from_str::<FileConfig>(&text)
        .map_err(|e| ConfigError::Parse(path.to_path_buf(), e.to_string()))
}

/// 예전 단일 서버 config.toml 을 servers.default 하나짜리 FileConfig 로 옮긴다.
fn parse_legacy(text: &str, path: &Path) -> Result<FileConfig, ConfigError> {
    #[derive(Default, Deserialize)]
    struct LegacyConfig {
        server_url: Option<String>,
        api_token: Option<String>,
        #[serde(default)]
        custom_fields: BTreeMap<String, u64>,
    }
    let legacy: LegacyConfig =
        toml::from_str(text).map_err(|e| ConfigError::Parse(path.to_path_buf(), e.to_string()))?;
    let mut servers = BTreeMap::new();
    servers.insert(
        LEGACY_SERVER_NAME.to_string(),
        ServerConfig {
            url: legacy.server_url.unwrap_or_default(),
            api_token: legacy.api_token.unwrap_or_default(),
            custom_fields: legacy.custom_fields,
        },
    );
    Ok(FileConfig {
        default_server: Some(LEGACY_SERVER_NAME.to_string()),
        servers,
    })
}

/// 외부에서 사용 가능한 형태로 config.yml 경로를 해석한다.
pub fn resolve_path(explicit: Option<PathBuf>) -> Option<PathBuf> {
    explicit.or_else(default_config_path)
}

/// config.yml 을 읽어 들이거나 없으면 빈 구조체를 돌려준다.
pub fn load_or_empty(path: Option<PathBuf>) -> Result<FileConfig, ConfigError> {
    load_file(path)
}

/// FileConfig 를 config.yml 형식으로 저장한다. 부모 디렉터리는 필요 시 생성한다.
///
/// 주의: 파일을 통째로 다시 쓰므로 손으로 넣은 YAML 주석은 사라진다.
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
    let text = serde_norway::to_string(file)
        .map_err(|e| ConfigError::Parse(path.to_path_buf(), e.to_string()))?;
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
    aliases: &BTreeMap<String, u64>,
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

    fn srv(url: &str, token: &str, cf: &[(&str, u64)]) -> ServerConfig {
        ServerConfig {
            url: url.to_string(),
            api_token: token.to_string(),
            custom_fields: cf.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    fn two_servers() -> FileConfig {
        let mut servers = BTreeMap::new();
        servers.insert(
            "company".to_string(),
            srv("https://company.example.com", "ctok", &[("state", 7)]),
        );
        servers.insert(
            "personal".to_string(),
            srv("https://personal.example.com", "ptok", &[("state", 3)]),
        );
        FileConfig {
            default_server: Some("company".to_string()),
            servers,
        }
    }

    fn overrides(server: Option<&str>) -> CliOverrides {
        CliOverrides {
            server: server.map(str::to_string),
            server_url: None,
            api_token: None,
            config_path: None,
        }
    }

    fn dummy_path() -> PathBuf {
        PathBuf::from("/nonexistent/config.yml")
    }

    #[test]
    fn selects_named_server() {
        let cfg =
            resolve_from(&two_servers(), &overrides(Some("personal")), &dummy_path()).unwrap();
        assert_eq!(cfg.server_url, "https://personal.example.com");
        assert_eq!(cfg.api_token, "ptok");
    }

    #[test]
    fn falls_back_to_default_server() {
        let cfg = resolve_from(&two_servers(), &overrides(None), &dummy_path()).unwrap();
        assert_eq!(cfg.server_url, "https://company.example.com");
    }

    #[test]
    fn single_server_needs_no_default() {
        let mut file = two_servers();
        file.default_server = None;
        file.servers.remove("personal");
        let cfg = resolve_from(&file, &overrides(None), &dummy_path()).unwrap();
        assert_eq!(cfg.server_url, "https://company.example.com");
    }

    #[test]
    fn ambiguous_without_default_is_an_error() {
        let mut file = two_servers();
        file.default_server = None;
        let err = resolve_from(&file, &overrides(None), &dummy_path()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("company") && msg.contains("personal"), "{msg}");
    }

    #[test]
    fn unknown_server_lists_available_names() {
        let err =
            resolve_from(&two_servers(), &overrides(Some("nope")), &dummy_path()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("nope"), "{msg}");
        assert!(msg.contains("company") && msg.contains("personal"), "{msg}");
    }

    #[test]
    fn dangling_default_server_is_an_error() {
        let mut file = two_servers();
        file.default_server = Some("gone".to_string());
        assert!(resolve_from(&file, &overrides(None), &dummy_path()).is_err());
    }

    #[test]
    fn aliases_come_from_the_selected_server() {
        let cfg =
            resolve_from(&two_servers(), &overrides(Some("personal")), &dummy_path()).unwrap();
        assert_eq!(cfg.cf_aliases.get("state"), Some(&3));
        let cfg = resolve_from(&two_servers(), &overrides(Some("company")), &dummy_path()).unwrap();
        assert_eq!(cfg.cf_aliases.get("state"), Some(&7));
    }

    #[test]
    fn flags_override_fields_of_the_selected_server() {
        let mut ov = overrides(Some("company"));
        ov.server_url = Some("https://staging.example.com".to_string());
        let cfg = resolve_from(&two_servers(), &ov, &dummy_path()).unwrap();
        assert_eq!(cfg.server_url, "https://staging.example.com");
        // 토큰과 alias 는 선택된 서버 것이 남는다.
        assert_eq!(cfg.api_token, "ctok");
        assert_eq!(cfg.cf_aliases.get("state"), Some(&7));
    }

    #[test]
    fn both_credential_flags_skip_server_selection() {
        // --server 없이 두 flag 가 다 오면 설정 파일과 무관한 ad-hoc 서버다.
        let mut file = two_servers();
        file.default_server = None; // 원래대로면 모호해서 에러
        let mut ov = overrides(None);
        ov.server_url = Some("https://adhoc.example.com".to_string());
        ov.api_token = Some("atok".to_string());
        let cfg = resolve_from(&file, &ov, &dummy_path()).unwrap();
        assert_eq!(cfg.server_url, "https://adhoc.example.com");
        assert_eq!(cfg.api_token, "atok");
        assert!(cfg.cf_aliases.is_empty());
    }

    #[test]
    fn empty_config_without_flags_is_an_error() {
        let err =
            resolve_from(&FileConfig::default(), &overrides(None), &dummy_path()).unwrap_err();
        assert!(err.to_string().contains("no Redmine server"), "{err}");
    }

    #[test]
    fn empty_server_fields_report_what_is_missing() {
        let mut file = FileConfig::default();
        file.servers.insert("bare".to_string(), srv("", "", &[]));
        let err = resolve_from(&file, &overrides(None), &dummy_path()).unwrap_err();
        assert!(matches!(err, ConfigError::MissingServer), "{err}");

        let mut file = FileConfig::default();
        file.servers
            .insert("bare".to_string(), srv("https://x.example.com", "", &[]));
        let err = resolve_from(&file, &overrides(None), &dummy_path()).unwrap_err();
        assert!(matches!(err, ConfigError::MissingToken), "{err}");
    }

    #[test]
    fn yaml_round_trip_preserves_servers() {
        let file = two_servers();
        let text = serde_norway::to_string(&file).unwrap();
        let back: FileConfig = serde_norway::from_str(&text).unwrap();
        assert_eq!(back.default_server.as_deref(), Some("company"));
        assert_eq!(back.servers["personal"].api_token, "ptok");
        assert_eq!(back.servers["company"].custom_fields["state"], 7);
    }

    #[test]
    fn legacy_toml_becomes_a_default_server() {
        let text = r#"
server_url = "https://old.example.com"
api_token = "otok"

[custom_fields]
state = 7
"#;
        let file = parse_legacy(text, &dummy_path()).unwrap();
        assert_eq!(file.default_server.as_deref(), Some(LEGACY_SERVER_NAME));
        let s = &file.servers[LEGACY_SERVER_NAME];
        assert_eq!(s.url, "https://old.example.com");
        assert_eq!(s.api_token, "otok");
        assert_eq!(s.custom_fields["state"], 7);
    }

    #[test]
    fn legacy_toml_without_credentials_still_keeps_aliases() {
        let text = "[custom_fields]\nstate = 7\n";
        let file = parse_legacy(text, &dummy_path()).unwrap();
        let s = &file.servers[LEGACY_SERVER_NAME];
        assert!(s.url.is_empty() && s.api_token.is_empty());
        assert_eq!(s.custom_fields["state"], 7);
    }

    #[test]
    fn debug_redacts_tokens() {
        let file = two_servers();
        let dumped = format!("{file:?}");
        assert!(!dumped.contains("ctok"), "{dumped}");
        assert!(dumped.contains("REDACTED"), "{dumped}");

        let cfg = resolve_from(&file, &overrides(None), &dummy_path()).unwrap();
        let dumped = format!("{cfg:?}");
        assert!(!dumped.contains("ctok"), "{dumped}");
    }

    #[test]
    fn parse_custom_field_numeric() {
        let aliases = BTreeMap::new();
        assert_eq!(
            parse_custom_field("7=Dev", &aliases).unwrap(),
            (7, "Dev".into())
        );
    }

    #[test]
    fn parse_custom_field_alias() {
        let mut aliases = BTreeMap::new();
        aliases.insert("state".to_string(), 7);
        assert_eq!(
            parse_custom_field("state=Dev", &aliases).unwrap(),
            (7, "Dev".into())
        );
    }

    #[test]
    fn parse_custom_field_unknown_alias() {
        let aliases = BTreeMap::new();
        assert!(parse_custom_field("zz=Dev", &aliases).is_err());
    }

    #[test]
    fn parse_custom_field_missing_equals() {
        let aliases = BTreeMap::new();
        assert!(parse_custom_field("Dev", &aliases).is_err());
    }
}
