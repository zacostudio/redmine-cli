// config 서브커맨드 핸들러. config.yml 의 서버 목록과 서버별 custom field alias 를 관리한다.
use clap::Subcommand;
use serde_json::json;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::config::{self, FileConfig};
use crate::output;

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Manage Redmine servers stored in config.yml.
    #[command(subcommand)]
    Server(ServerCommand),
    /// Manage custom field aliases of the selected server.
    ///
    /// A Redmine custom field is addressed by a numeric id, and the same
    /// field usually has a different id on each server. An alias is a name
    /// for one id on one server, stored under that server in config.yml, and
    /// used as `--custom-field <alias>=<value>`. Aliases never cross servers.
    #[command(subcommand)]
    Alias(AliasCommand),
}

#[derive(Subcommand, Debug)]
pub enum ServerCommand {
    /// List configured servers. API tokens are never printed.
    List,
    /// Add a server. The token comes from --api-token, or from stdin when omitted.
    Add {
        /// Name to store it under (used as --server <name>).
        name: String,
        /// Redmine base URL.
        #[arg(long)]
        url: String,
        /// Overwrite an existing entry with the same name.
        #[arg(long)]
        force: bool,
    },
    /// Remove a server. Clears default_server when it pointed at that server.
    Remove {
        /// Server name to remove.
        name: String,
    },
    /// Set the default server used when --server is omitted.
    Use {
        /// Server name as it appears under `servers:` in config.yml.
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AliasCommand {
    /// List custom field aliases of the selected server.
    List,
    /// Set or update an alias on the selected server.
    Set {
        /// Alias name (used as --custom-field <name>=value).
        name: String,
        /// Numeric custom field ID.
        id: u64,
    },
    /// Remove an alias from the selected server.
    Remove {
        /// Alias name to remove.
        name: String,
    },
}

pub fn handle(
    cmd: ConfigCommand,
    config_path: Option<PathBuf>,
    server: Option<String>,
    api_token: Option<String>,
) {
    let path = match config::resolve_path(config_path) {
        Some(p) => p,
        None => output::print_error("could not determine config path; pass --config"),
    };
    match cmd {
        ConfigCommand::Server(sub) => match sub {
            ServerCommand::List => server_list(&path),
            ServerCommand::Add { name, url, force } => {
                server_add(&path, name, url, force, api_token)
            }
            ServerCommand::Remove { name } => server_remove(&path, name),
            ServerCommand::Use { name } => server_use(&path, name),
        },
        ConfigCommand::Alias(sub) => match sub {
            AliasCommand::List => alias_list(&path, server),
            AliasCommand::Set { name, id } => alias_set(&path, server, name, id),
            AliasCommand::Remove { name } => alias_remove(&path, server, name),
        },
    }
}

fn load(path: &Path) -> FileConfig {
    match config::load_or_empty(Some(path.to_path_buf())) {
        Ok(f) => f,
        Err(e) => output::print_error(&e.to_string()),
    }
}

fn store(path: &Path, file: &FileConfig) {
    if let Err(e) = config::save(path, file) {
        output::print_error(&e.to_string());
    }
    output::print_json(json!({ "ok": true, "path": path.display().to_string() }));
}

/// alias 명령이 대상으로 삼을 서버를 고른다. 없는 서버를 자동 생성하지는 않는다.
fn selected(file: &FileConfig, server: Option<String>, path: &Path) -> String {
    match config::select_server_name(file, server.as_deref(), path) {
        Ok(name) => name,
        Err(e) => output::print_error(&e.to_string()),
    }
}

fn server_list(path: &Path) {
    let file = load(path);
    // `default` 는 default_server 필드가 아니라 실제 해석 결과를 따른다. 둘이 갈라지면
    // 사용자가 여기서 본 것과 다른 서버로 호출이 나간다.
    let effective = config::select_server_name(&file, None, path);
    let selected = effective.as_ref().ok().cloned();
    // 토큰은 절대 싣지 않는다. 이 명령의 출력은 로그·이슈에 그대로 붙는 일이 잦다.
    let servers: Vec<_> = file
        .servers
        .iter()
        .map(|(name, s)| {
            json!({
                "name": name,
                "url": s.url,
                "default": selected.as_deref() == Some(name.as_str()),
                "custom_fields": s.custom_fields,
            })
        })
        .collect();
    let mut out = json!({
        "path": path.display().to_string(),
        "default_server": file.default_server,
        "servers": servers,
    });
    // 서버가 하나도 없는 경우까지 경고로 만들면 시끄럽다. 설정이 모순된 경우만 알린다.
    if let Err(e) = effective {
        if !file.servers.is_empty() {
            out["warning"] = json!(e.to_string());
        }
    }
    output::print_json(out);
}

/// add 의 토큰은 --api-token 이 없으면 stdin 에서 읽는다. 셸 히스토리에 토큰을 남기지
/// 않으려는 사용을 위해서다. 터미널이 그대로 붙어 있으면 입력을 기다리며 멈추므로,
/// 파이프가 아닐 때는 기다리지 않는다. `existing` 은 --force 로 덮어쓰는 경우의 기존 토큰이다.
fn token_for_add(api_token: Option<String>, existing: Option<&str>) -> String {
    if let Some(t) = api_token
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
    {
        return validated_token(t);
    }
    if std::io::stdin().is_terminal() {
        // URL 만 바꾸려는 --force 재실행에서 토큰을 다시 요구하지 않는다.
        if let Some(t) = existing.filter(|t| !t.is_empty()) {
            return t.to_string();
        }
        output::print_error("missing API token; pass --api-token or pipe the token to stdin");
    }
    match output::read_stdin() {
        Ok(t) => {
            let t = t.trim().to_string();
            if t.is_empty() {
                match existing.filter(|t| !t.is_empty()) {
                    Some(t) => return t.to_string(),
                    None => output::print_error("empty API token on stdin"),
                }
            }
            validated_token(t)
        }
        Err(e) => output::print_error(&e),
    }
}

/// 저장 시점에 막지 않으면 이후 모든 명령이 "invalid API key" 로 실패한다. 원인에서 먼 에러다.
fn validated_token(token: String) -> String {
    if let Err(e) = config::validate_token(&token) {
        output::print_error(&e);
    }
    token
}

fn server_add(path: &Path, name: String, url: String, force: bool, api_token: Option<String>) {
    let name = name.trim().to_string();
    if name.is_empty() {
        output::print_error("server name must not be empty");
    }
    let url = url.trim().to_string();
    validate_url(&url);
    let mut file = load(path);
    let existing = file.servers.get(&name);
    if existing.is_some() && !force {
        output::print_error(&format!(
            "Redmine server '{name}' already exists; pass --force to overwrite"
        ));
    }
    // 기존 항목을 덮어쓸 때 alias 와 토큰은 살린다. URL 만 갱신하려는 경우가 대부분이다.
    let custom_fields = existing
        .map(|s| s.custom_fields.clone())
        .unwrap_or_default();
    let previous_token = existing.map(|s| s.api_token.clone());
    let api_token = token_for_add(api_token, previous_token.as_deref());
    file.servers.insert(
        name.clone(),
        crate::config::ServerConfig {
            url,
            api_token,
            custom_fields,
        },
    );
    // 서버가 이것 하나뿐일 때만 기본으로 삼는다. default_server 가 없다는 이유로 승격시키면,
    // 손으로 서버 둘을 적어 둔 설정에 하나 더 추가했을 때 이후 호출이 조용히 새 서버로 간다.
    if file.default_server.is_none() && file.servers.len() == 1 {
        file.default_server = Some(name.clone());
    }
    if let Err(e) = config::save(path, &file) {
        output::print_error(&e.to_string());
    }
    // 어느 서버가 기본인지 매번 드러내야 --server 를 빠뜨렸을 때 놀라지 않는다.
    output::print_json(json!({
        "ok": true,
        "path": path.display().to_string(),
        "added": name,
        "default_server": file.default_server,
    }));
}

/// 잘못된 URL 은 저장된 뒤 "builder error" 라는 알아볼 수 없는 에러로만 드러난다.
fn validate_url(url: &str) {
    if url.is_empty() {
        output::print_error("--url must not be empty");
    }
    match reqwest::Url::parse(url) {
        Ok(u) if matches!(u.scheme(), "http" | "https") => {}
        Ok(u) => output::print_error(&format!(
            "--url must be http or https, got scheme '{}'",
            u.scheme()
        )),
        Err(e) => output::print_error(&format!("--url is not a valid URL: {e}")),
    }
}

fn server_remove(path: &Path, name: String) {
    let mut file = load(path);
    exists_or_exit(&file, &name, path);
    file.servers.remove(&name);
    // 지운 서버를 가리키던 기본값을 남겨두면 이후 모든 호출이 에러가 된다.
    if file.default_server.as_deref() == Some(name.as_str()) {
        file.default_server = None;
    }
    if let Err(e) = config::save(path, &file) {
        output::print_error(&e.to_string());
    }
    // 기본 서버가 비워졌다는 사실이 드러나야 다음 호출에서 놀라지 않는다.
    output::print_json(json!({
        "ok": true,
        "path": path.display().to_string(),
        "removed": name,
        "default_server": file.default_server,
    }));
}

fn server_use(path: &Path, name: String) {
    let mut file = load(path);
    exists_or_exit(&file, &name, path);
    file.default_server = Some(name);
    store(path, &file);
}

fn exists_or_exit(file: &FileConfig, name: &str, path: &Path) {
    if let Err(e) = config::ensure_server_exists(file, name, path) {
        output::print_error(&e.to_string());
    }
}

fn alias_list(path: &Path, server: Option<String>) {
    let file = load(path);
    let name = selected(&file, server, path);
    output::print_json(json!({
        "server": name,
        "aliases": file.servers[&name].custom_fields,
    }));
}

fn alias_set(path: &Path, server: Option<String>, alias: String, id: u64) {
    validate_alias(&alias);
    let mut file = load(path);
    let name = selected(&file, server, path);
    file.servers
        .get_mut(&name)
        .expect("selected server exists")
        .custom_fields
        .insert(alias, id);
    store(path, &file);
}

/// `--custom-field <k>=<v>` 는 k 를 먼저 숫자로 파싱하고 '=' 로 자른다. 그래서 숫자 이름과
/// '=' 를 담은 이름은 저장돼도 영영 해석되지 않는다. 조용히 다른 cf id 로 나가는 것보다
/// 저장을 막는 편이 낫다.
fn validate_alias(alias: &str) {
    if alias.trim().is_empty() {
        output::print_error("alias name must not be empty");
    }
    if alias.parse::<u64>().is_ok() {
        output::print_error(&format!(
            "alias name '{alias}' is numeric; it would always resolve as a custom field id"
        ));
    }
    if alias.contains('=') {
        output::print_error("alias name must not contain '='");
    }
}

fn alias_remove(path: &Path, server: Option<String>, alias: String) {
    let mut file = load(path);
    let name = selected(&file, server, path);
    let removed = file
        .servers
        .get_mut(&name)
        .expect("selected server exists")
        .custom_fields
        .remove(&alias);
    if removed.is_none() {
        output::print_error(&format!("alias not found on server '{name}': {alias}"));
    }
    store(path, &file);
}
