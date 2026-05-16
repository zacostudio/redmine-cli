// version 서브커맨드 핸들러 (프로젝트 버전/마일스톤 CRUD).
use clap::{Args, Subcommand};
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;
use crate::types::RedmineVersion;

#[derive(Subcommand, Debug)]
pub enum VersionCommand {
    /// List versions of a project.
    List(ListArgs),
    /// Show a single version by id.
    Show(ShowArgs),
    /// Create a version under a project.
    Create(CreateArgs),
    /// Update a version by id.
    Update(UpdateArgs),
    /// Delete a version by id.
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    pub project: String,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    pub id: u64,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    pub project: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    /// open | locked | closed
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long = "due-date")]
    pub due_date: Option<String>,
    /// none | descendants | hierarchy | tree | system
    #[arg(long)]
    pub sharing: Option<String>,
    #[arg(long = "wiki-page-title")]
    pub wiki_page_title: Option<String>,
    /// Print only the new version ID followed by a newline.
    #[arg(long = "id-only", default_value_t = false)]
    pub id_only: bool,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    pub id: u64,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long = "due-date")]
    pub due_date: Option<String>,
    #[arg(long)]
    pub sharing: Option<String>,
    #[arg(long = "wiki-page-title")]
    pub wiki_page_title: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    pub id: u64,
}

pub fn handle(cmd: VersionCommand, client: &RedmineClient) {
    match cmd {
        VersionCommand::List(a) => list(a, client),
        VersionCommand::Show(a) => show(a, client),
        VersionCommand::Create(a) => create(a, client),
        VersionCommand::Update(a) => update(a, client),
        VersionCommand::Delete(a) => delete(a, client),
    }
}

fn version_to_json(v: &RedmineVersion) -> Value {
    json!({
        "id": v.id,
        "project": v.project.as_ref().map(|p| &p.name),
        "name": v.name,
        "description": v.description,
        "status": v.status,
        "due_date": v.due_date,
        "sharing": v.sharing,
        "wiki_page_title": v.wiki_page_title,
        "created_on": v.created_on,
        "updated_on": v.updated_on,
    })
}

fn list(a: ListArgs, client: &RedmineClient) {
    match client.list_versions(&a.project) {
        Ok(r) => {
            let items: Vec<_> = r.versions.iter().map(version_to_json).collect();
            output::print_json(json!({ "versions": items, "total_count": r.total_count }));
        }
        Err(e) => output::print_error(&format!("redmine version list: {e}")),
    }
}

fn show(a: ShowArgs, client: &RedmineClient) {
    match client.get_version(a.id) {
        Ok(r) => output::print_json(version_to_json(&r.version)),
        Err(e) => output::print_error(&format!("redmine version show: {e}")),
    }
}

fn create(a: CreateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    payload.insert("name".into(), json!(a.name));
    if let Some(v) = a.description {
        payload.insert("description".into(), json!(v));
    }
    if let Some(v) = a.status {
        payload.insert("status".into(), json!(v));
    }
    if let Some(v) = a.due_date {
        payload.insert("due_date".into(), json!(v));
    }
    if let Some(v) = a.sharing {
        payload.insert("sharing".into(), json!(v));
    }
    if let Some(v) = a.wiki_page_title {
        payload.insert("wiki_page_title".into(), json!(v));
    }
    match client.create_version(&a.project, Value::Object(payload)) {
        Ok(r) => {
            if a.id_only {
                println!("{}", r.version.id);
            } else {
                output::print_json(version_to_json(&r.version));
            }
        }
        Err(e) => output::print_error(&format!("redmine version create: {e}")),
    }
}

fn update(a: UpdateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    if let Some(v) = a.name {
        payload.insert("name".into(), json!(v));
    }
    if let Some(v) = a.description {
        payload.insert("description".into(), json!(v));
    }
    if let Some(v) = a.status {
        payload.insert("status".into(), json!(v));
    }
    if let Some(v) = a.due_date {
        payload.insert("due_date".into(), json!(v));
    }
    if let Some(v) = a.sharing {
        payload.insert("sharing".into(), json!(v));
    }
    if let Some(v) = a.wiki_page_title {
        payload.insert("wiki_page_title".into(), json!(v));
    }
    if payload.is_empty() {
        output::print_error("version update: no fields to update");
    }
    match client.update_version(a.id, Value::Object(payload)) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine version update: {e}")),
    }
}

fn delete(a: DeleteArgs, client: &RedmineClient) {
    match client.delete_version(a.id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine version delete: {e}")),
    }
}
