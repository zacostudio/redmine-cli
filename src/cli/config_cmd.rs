// config 서브커맨드 핸들러. config.toml 의 custom field alias 를 관리한다.
use clap::Subcommand;
use serde_json::json;
use std::path::PathBuf;

use crate::config;
use crate::output;

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Manage custom field aliases stored in config.toml.
    #[command(subcommand)]
    Alias(AliasCommand),
}

#[derive(Subcommand, Debug)]
pub enum AliasCommand {
    /// List all custom field aliases.
    List,
    /// Set or update an alias.
    Set {
        /// Alias name (used as --custom-field <name>=value).
        name: String,
        /// Numeric custom field ID.
        id: u64,
    },
    /// Remove an alias.
    Remove {
        /// Alias name to remove.
        name: String,
    },
}

pub fn handle(cmd: ConfigCommand, config_path: Option<PathBuf>) {
    match cmd {
        ConfigCommand::Alias(sub) => match sub {
            AliasCommand::List => list(config_path),
            AliasCommand::Set { name, id } => set(name, id, config_path),
            AliasCommand::Remove { name } => remove(name, config_path),
        },
    }
}

fn list(config_path: Option<PathBuf>) {
    let file = match config::load_or_empty(config_path) {
        Ok(f) => f,
        Err(e) => output::print_error(&e.to_string()),
    };
    output::print_json(json!({ "aliases": file.custom_fields }));
}

fn set(name: String, id: u64, config_path: Option<PathBuf>) {
    let path = match config::resolve_path(config_path) {
        Some(p) => p,
        None => output::print_error("could not determine config path; pass --config"),
    };
    let mut file = match config::load_or_empty(Some(path.clone())) {
        Ok(f) => f,
        Err(e) => output::print_error(&e.to_string()),
    };
    file.custom_fields.insert(name, id);
    if let Err(e) = config::save(&path, &file) {
        output::print_error(&e.to_string());
    }
    output::print_json(json!({ "ok": true, "path": path.display().to_string() }));
}

fn remove(name: String, config_path: Option<PathBuf>) {
    let path = match config::resolve_path(config_path) {
        Some(p) => p,
        None => output::print_error("could not determine config path; pass --config"),
    };
    let mut file = match config::load_or_empty(Some(path.clone())) {
        Ok(f) => f,
        Err(e) => output::print_error(&e.to_string()),
    };
    if file.custom_fields.remove(&name).is_none() {
        output::print_error(&format!("alias not found: {name}"));
    }
    if let Err(e) = config::save(&path, &file) {
        output::print_error(&e.to_string());
    }
    output::print_json(json!({ "ok": true, "path": path.display().to_string() }));
}
