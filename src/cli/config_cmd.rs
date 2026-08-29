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

pub fn handle(cmd: ConfigCommand, config_path: Option<PathBuf>, server: Option<String>) {
    let path = match config::resolve_path(config_path) {
        Some(p) => p,
        None => output::print_error("could not determine config path; pass --config"),
    };
    match cmd {
        ConfigCommand::Server(sub) => match sub {
            ServerCommand::List => server_list(&path),
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
