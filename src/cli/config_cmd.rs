// config 서브커맨드 핸들러. config.yml 의 서버 목록과 서버별 custom field alias 를 관리한다.
use clap::Subcommand;
use serde_json::json;
use std::path::{Path, PathBuf};

use crate::config::{self, FileConfig};
use crate::output;

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Manage Redmine servers stored in config.yml.
    #[command(subcommand)]
    Server(ServerCommand),
    /// Manage custom field aliases of the selected server.
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
    // 토큰은 절대 싣지 않는다. 이 명령의 출력은 로그·이슈에 그대로 붙는 일이 잦다.
    let servers: Vec<_> = file
        .servers
        .iter()
        .map(|(name, s)| {
            json!({
                "name": name,
                "url": s.url,
                "default": file.default_server.as_deref() == Some(name.as_str()),
                "custom_fields": s.custom_fields,
            })
        })
        .collect();
    output::print_json(json!({
        "path": path.display().to_string(),
        "default_server": file.default_server,
        "servers": servers,
    }));
}

/// add 의 토큰은 --api-token 이 없으면 stdin 에서 읽는다. 셸 히스토리에 토큰을 남기지
/// 않으려는 사용을 위해서다. 터미널이 그대로 붙어 있으면 입력을 기다리며 멈추므로,
/// 파이프가 아닐 때는 기다리지 않고 에러를 낸다.
fn token_for_add(api_token: Option<String>) -> String {
    if let Some(t) = api_token
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
    {
        return t;
    }
    // SAFETY: isatty 는 fd 하나만 보는 순수 조회다.
    let piped = unsafe { libc::isatty(0) } == 0;
    if !piped {
        output::print_error("missing API token; pass --api-token or pipe the token to stdin");
    }
    match output::read_stdin() {
        Ok(t) => {
            let t = t.trim().to_string();
            if t.is_empty() {
                output::print_error("empty API token on stdin");
            }
            t
        }
        Err(e) => output::print_error(&e),
    }
}

fn server_add(path: &Path, name: String, url: String, force: bool, api_token: Option<String>) {
    let url = url.trim().to_string();
    if url.is_empty() {
        output::print_error("--url must not be empty");
    }
    let mut file = load(path);
    if file.servers.contains_key(&name) && !force {
        output::print_error(&format!(
            "Redmine server '{name}' already exists; pass --force to overwrite"
        ));
    }
    let api_token = token_for_add(api_token);
    // 기존 항목을 덮어쓸 때 alias 는 살린다. URL/토큰만 갱신하려는 경우가 대부분이다.
    let custom_fields = file
        .servers
        .get(&name)
        .map(|s| s.custom_fields.clone())
        .unwrap_or_default();
    file.servers.insert(
        name.clone(),
        crate::config::ServerConfig {
            url,
            api_token,
            custom_fields,
        },
    );
    // 첫 서버는 기본 서버가 된다. 서버가 하나뿐이면 --server 를 매번 쓰게 할 이유가 없다.
    if file.default_server.is_none() {
        file.default_server = Some(name);
    }
    store(path, &file);
}

fn server_remove(path: &Path, name: String) {
    let mut file = load(path);
    if file.servers.remove(&name).is_none() {
        output::print_error(&format!(
            "Redmine server '{name}' not found in config (available: {})",
            file.servers.keys().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    // 지운 서버를 가리키던 기본값을 남겨두면 이후 모든 호출이 에러가 된다.
    if file.default_server.as_deref() == Some(name.as_str()) {
        file.default_server = None;
    }
    store(path, &file);
}

fn server_use(path: &Path, name: String) {
    let mut file = load(path);
    if !file.servers.contains_key(&name) {
        output::print_error(&format!(
            "Redmine server '{name}' not found in config (available: {})",
            file.servers.keys().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    file.default_server = Some(name);
    store(path, &file);
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
    let mut file = load(path);
    let name = selected(&file, server, path);
    file.servers
        .get_mut(&name)
        .expect("selected server exists")
        .custom_fields
        .insert(alias, id);
    store(path, &file);
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
