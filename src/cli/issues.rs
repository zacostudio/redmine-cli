// issues / issue 서브커맨드 핸들러.
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::config::{self, Config};
use crate::output;

// ── issues (search) ─────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct IssuesArgs {
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long = "assigned-to")]
    pub assigned_to: Option<String>,
    #[arg(long)]
    pub tracker: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    #[arg(long)]
    pub offset: Option<u64>,
    #[arg(long)]
    pub sort: Option<String>,
    /// Filter by custom field, `<id>=<value>` or `<alias>=<value>`. Repeatable.
    ///
    /// Aliases come from the selected server in config.yml
    /// (`redmine config alias list` shows them). Numeric names are read as
    /// ids, never as aliases.
    /// Custom field as `<id>=<value>` or `<alias>=<value>`. Repeatable.
    ///
    /// Aliases are per server: `redmine config alias set state 7` stores
    /// state -> 7 for the selected server, so `--custom-field state=Dev`
    /// sends custom field 7 there and whatever id `state` has on another
    /// server. Numeric names are always read as ids, never as aliases.
    /// An unknown alias is an error — nothing is sent.
    #[arg(long = "custom-field", value_name = "ID_OR_ALIAS=VALUE")]
    pub custom_field: Vec<String>,
}

#[derive(Serialize)]
struct IssueListOut {
    id: u64,
    subject: String,
    status: Option<String>,
    priority: Option<String>,
    assigned_to: Option<String>,
    project: String,
    updated_on: Option<String>,
}

pub fn handle_search(args: IssuesArgs, client: &RedmineClient, cfg: &Config) {
    let mut params: Vec<(String, String)> = Vec::new();
    if let Some(v) = args.project {
        params.push(("project_id".into(), v));
    }
    if let Some(v) = args.status {
        params.push(("status_id".into(), v));
    }
    if let Some(v) = args.query {
        params.push(("subject".into(), format!("~{v}")));
    }
    if let Some(v) = args.assigned_to {
        params.push(("assigned_to_id".into(), v));
    }
    if let Some(v) = args.tracker {
        params.push(("tracker_id".into(), v));
    }
    if let Some(v) = args.priority {
        params.push(("priority_id".into(), v));
    }
    params.push(("limit".into(), args.limit.to_string()));
    if let Some(o) = args.offset {
        params.push(("offset".into(), o.to_string()));
    }
    if let Some(v) = args.sort {
        params.push(("sort".into(), v));
    }

    // custom-field 옵션. cf_<id>=value 쿼리 파라미터로 변환.
    for spec in args.custom_field {
        let (id, val) = match config::parse_custom_field(&spec, &cfg.cf_aliases) {
            Ok(p) => p,
            Err(e) => output::print_error(&e),
        };
        params.push((format!("cf_{id}"), val));
    }

    let borrowed: Vec<(&str, String)> = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();
    let resp = match client.search_issues(&borrowed) {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine issues: {e}")),
    };
    let out: Vec<IssueListOut> = resp
        .issues
        .into_iter()
        .map(|i| IssueListOut {
            id: i.id,
            subject: i.subject,
            status: i.status.map(|s| s.name),
            priority: i.priority.map(|p| p.name),
            assigned_to: i.assigned_to.map(|a| a.name),
            project: i.project.name,
            updated_on: i.updated_on,
        })
        .collect();
    output::print_json(json!({ "issues": out, "total_count": resp.total_count }));
}

// ── issue (one) ─────────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct IssueArgs {
    /// Issue ID (positional). Required unless using `create` / `remove-relation`.
    pub id: Option<u64>,
    #[command(subcommand)]
    pub sub: Option<IssueSub>,
}

#[derive(Subcommand, Debug)]
pub enum IssueSub {
    /// Create a new issue.
    Create(IssueCreateArgs),
    /// Update an existing issue (requires <id>).
    Update(IssueUpdateArgs),
    /// Delete an issue (requires <id>).
    Delete,
    /// List relations of an issue.
    Relations,
    /// Add a relation from <id> to --to.
    AddRelation(IssueAddRelationArgs),
    /// Remove a relation by its relation-id.
    RemoveRelation(IssueRemoveRelationArgs),
    /// Manage watchers on an issue.
    #[command(subcommand)]
    Watcher(IssueWatcherSub),
    /// Post a note (journal entry) on an issue.
    Note(IssueNoteArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum IssueWatcherSub {
    /// Add a user as watcher.
    Add(IssueWatcherArgs),
    /// Remove a user from watchers.
    Remove(IssueWatcherArgs),
}

#[derive(Args, Debug, Clone)]
pub struct IssueWatcherArgs {
    /// User id to add or remove.
    #[arg(long = "user")]
    pub user: u64,
}

#[derive(Args, Debug, Clone)]
pub struct IssueNoteArgs {
    /// Note body. Use "-" to read from stdin.
    #[arg(long)]
    pub message: String,
    /// 비공개 노트로 등록한다. 기존 호환을 위해 `--private` 도 받는다.
    #[arg(
        long = "private-notes",
        visible_alias = "private",
        default_value_t = false
    )]
    pub private_notes: bool,
}

#[derive(Args, Debug, Default, Clone)]
pub struct IssueCreateArgs {
    #[arg(long)]
    pub project: String,
    #[arg(long)]
    pub subject: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub tracker: Option<u64>,
    #[arg(long)]
    pub priority: Option<u64>,
    #[arg(long = "assigned-to")]
    pub assigned_to: Option<u64>,
    #[arg(long)]
    pub category: Option<u64>,
    #[arg(long)]
    pub parent: Option<u64>,
    #[arg(long = "start-date")]
    pub start_date: Option<String>,
    #[arg(long = "due-date")]
    pub due_date: Option<String>,
    #[arg(long = "estimated-hours")]
    pub estimated_hours: Option<f64>,
    #[arg(long = "done-ratio")]
    pub done_ratio: Option<u32>,
    #[arg(long = "target-version")]
    pub target_version: Option<u64>,
    /// Custom field as `<id>=<value>` or `<alias>=<value>`. Repeatable.
    ///
    /// Aliases are per server: `redmine config alias set state 7` stores
    /// state -> 7 for the selected server, so `--custom-field state=Dev`
    /// sends custom field 7 there and whatever id `state` has on another
    /// server. Numeric names are always read as ids, never as aliases.
    /// An unknown alias is an error — nothing is sent.
    #[arg(long = "custom-field", value_name = "ID_OR_ALIAS=VALUE")]
    pub custom_field: Vec<String>,
    /// Print only the new issue ID followed by a newline, instead of the
    /// full JSON. Designed so shell scripts can do `id=$(redmine issue
    /// create ... --id-only)` without piping through `jq` / `python -c`
    /// (whose own parse failures used to be mistaken for create failures,
    /// causing duplicate issues on retry).
    #[arg(long = "id-only", default_value_t = false)]
    pub id_only: bool,
}

#[derive(Args, Debug, Default, Clone)]
pub struct IssueUpdateArgs {
    #[arg(long)]
    pub subject: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub status: Option<u64>,
    #[arg(long)]
    pub tracker: Option<u64>,
    #[arg(long)]
    pub priority: Option<u64>,
    #[arg(long = "assigned-to")]
    pub assigned_to: Option<u64>,
    #[arg(long)]
    pub category: Option<u64>,
    #[arg(long)]
    pub parent: Option<u64>,
    #[arg(long = "start-date")]
    pub start_date: Option<String>,
    #[arg(long = "due-date")]
    pub due_date: Option<String>,
    #[arg(long = "estimated-hours")]
    pub estimated_hours: Option<f64>,
    #[arg(long = "done-ratio")]
    pub done_ratio: Option<u32>,
    #[arg(long = "target-version")]
    pub target_version: Option<u64>,
    #[arg(long)]
    pub notes: Option<String>,
    #[arg(long = "private-notes", default_value_t = false)]
    pub private_notes: bool,
    /// Custom field as `<id>=<value>` or `<alias>=<value>`. Repeatable.
    ///
    /// Aliases are per server: `redmine config alias set state 7` stores
    /// state -> 7 for the selected server, so `--custom-field state=Dev`
    /// sends custom field 7 there and whatever id `state` has on another
    /// server. Numeric names are always read as ids, never as aliases.
    /// An unknown alias is an error — nothing is sent.
    #[arg(long = "custom-field", value_name = "ID_OR_ALIAS=VALUE")]
    pub custom_field: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub struct IssueAddRelationArgs {
    #[arg(long)]
    pub to: u64,
    #[arg(long, default_value = "relates")]
    pub r#type: String,
    /// Print only the new relation ID followed by a newline.
    #[arg(long = "id-only", default_value_t = false)]
    pub id_only: bool,
}

#[derive(Args, Debug, Clone)]
pub struct IssueRemoveRelationArgs {
    /// Relation ID (positional).
    pub relation_id: u64,
}

pub fn handle_one(args: IssueArgs, client: &RedmineClient, cfg: &Config) {
    // 1) id 없이도 가능한 케이스
    match &args.sub {
        Some(IssueSub::Create(c)) => return create(c.clone(), client, cfg),
        Some(IssueSub::RemoveRelation(r)) => return remove_relation(r.relation_id, client),
        _ => {}
    }

    // 2) id 필수
    let id = match args.id {
        Some(n) => n,
        None => output::print_error("redmine issue: <id> is required"),
    };

    match args.sub {
        Some(IssueSub::Update(u)) => update(id, u, client, cfg),
        Some(IssueSub::Delete) => match client.delete_issue(id) {
            Ok(()) => output::print_json(json!({ "ok": true })),
            Err(e) => output::print_error(&format!("failed to delete issue: {e}")),
        },
        Some(IssueSub::Relations) => match client.get_issue(id, &["relations"]) {
            Ok(resp) => {
                let rels = resp.issue.relations.unwrap_or_default();
                let items: Vec<Value> = rels
                    .iter()
                    .map(|r| {
                        json!({
                            "id": r.id,
                            "issue_id": r.issue_id,
                            "issue_to_id": r.issue_to_id,
                            "relation_type": r.relation_type,
                        })
                    })
                    .collect();
                output::print_json(json!(items));
            }
            Err(e) => output::print_error(&format!("failed to get relations: {e}")),
        },
        Some(IssueSub::AddRelation(a)) => match client.create_relation(id, a.to, &a.r#type) {
            Ok(resp) => {
                if a.id_only {
                    println!("{}", resp.relation.id);
                } else {
                    output::print_json(json!({
                        "id": resp.relation.id,
                        "issue_id": resp.relation.issue_id,
                        "issue_to_id": resp.relation.issue_to_id,
                        "relation_type": resp.relation.relation_type,
                    }));
                }
            }
            Err(e) => output::print_error(&format!("failed to add relation: {e}")),
        },
        Some(IssueSub::Watcher(IssueWatcherSub::Add(w))) => {
            match client.add_issue_watcher(id, w.user) {
                Ok(()) => output::print_json(json!({ "ok": true })),
                Err(e) => output::print_error(&format!("failed to add watcher: {e}")),
            }
        }
        Some(IssueSub::Watcher(IssueWatcherSub::Remove(w))) => {
            match client.remove_issue_watcher(id, w.user) {
                Ok(()) => output::print_json(json!({ "ok": true })),
                Err(e) => output::print_error(&format!("failed to remove watcher: {e}")),
            }
        }
        Some(IssueSub::Note(n)) => post_note(id, n, client),
        None => match client.get_issue(id, &["journals", "attachments", "children", "relations"]) {
            Ok(r) => output::print_json(serde_json::to_value(&r.issue).unwrap_or(json!({}))),
            Err(e) => output::print_error(&format!("redmine issue: {e}")),
        },
        Some(IssueSub::Create(_)) | Some(IssueSub::RemoveRelation(_)) => unreachable!(),
    }
}

fn post_note(id: u64, a: IssueNoteArgs, client: &RedmineClient) {
    let text = if a.message == "-" {
        match output::read_stdin() {
            Ok(s) => s,
            Err(e) => output::print_error(&format!("redmine issue note: {e}")),
        }
    } else {
        a.message
    };
    let mut payload = serde_json::Map::new();
    payload.insert("notes".into(), json!(text));
    if a.private_notes {
        payload.insert("private_notes".into(), json!(true));
    }
    match client.update_issue(id, Value::Object(payload)) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine issue note: {e}")),
    }
}

fn cf_array(specs: &[String], cfg: &Config) -> Vec<Value> {
    specs
        .iter()
        .map(
            |spec| match config::parse_custom_field(spec, &cfg.cf_aliases) {
                Ok((id, value)) => json!({ "id": id, "value": value }),
                Err(e) => output::print_error(&e),
            },
        )
        .collect()
}

fn create(a: IssueCreateArgs, client: &RedmineClient, cfg: &Config) {
    let project_id_val: Value = match a.project.parse::<u64>() {
        Ok(n) => json!(n),
        Err(_) => json!(a.project),
    };
    let mut payload = serde_json::Map::new();
    payload.insert("project_id".into(), project_id_val);
    payload.insert("subject".into(), json!(a.subject));
    if let Some(v) = a.description {
        payload.insert("description".into(), json!(v));
    }
    if let Some(v) = a.tracker {
        payload.insert("tracker_id".into(), json!(v));
    }
    if let Some(v) = a.priority {
        payload.insert("priority_id".into(), json!(v));
    }
    if let Some(v) = a.assigned_to {
        payload.insert("assigned_to_id".into(), json!(v));
    }
    if let Some(v) = a.category {
        payload.insert("category_id".into(), json!(v));
    }
    if let Some(v) = a.parent {
        payload.insert("parent_issue_id".into(), json!(v));
    }
    if let Some(v) = a.start_date {
        payload.insert("start_date".into(), json!(v));
    }
    if let Some(v) = a.due_date {
        payload.insert("due_date".into(), json!(v));
    }
    if let Some(v) = a.estimated_hours {
        payload.insert("estimated_hours".into(), json!(v));
    }
    if let Some(v) = a.done_ratio {
        payload.insert("done_ratio".into(), json!(v));
    }
    if let Some(v) = a.target_version {
        payload.insert("fixed_version_id".into(), json!(v));
    }
    let cf = cf_array(&a.custom_field, cfg);
    if !cf.is_empty() {
        payload.insert("custom_fields".into(), json!(cf));
    }

    match client.create_issue(Value::Object(payload)) {
        Ok(r) => {
            if a.id_only {
                println!("{}", r.issue.id);
            } else {
                output::print_json(serde_json::to_value(&r.issue).unwrap_or(json!({})));
            }
        }
        Err(e) => output::print_error(&format!("redmine issue create: {e}")),
    }
}

fn update(id: u64, a: IssueUpdateArgs, client: &RedmineClient, cfg: &Config) {
    let mut payload = serde_json::Map::new();
    if let Some(v) = a.subject {
        payload.insert("subject".into(), json!(v));
    }
    if let Some(v) = a.description {
        payload.insert("description".into(), json!(v));
    }
    if let Some(v) = a.status {
        payload.insert("status_id".into(), json!(v));
    }
    if let Some(v) = a.tracker {
        payload.insert("tracker_id".into(), json!(v));
    }
    if let Some(v) = a.priority {
        payload.insert("priority_id".into(), json!(v));
    }
    if let Some(v) = a.assigned_to {
        payload.insert("assigned_to_id".into(), json!(v));
    }
    if let Some(v) = a.category {
        payload.insert("category_id".into(), json!(v));
    }
    if let Some(v) = a.parent {
        payload.insert("parent_issue_id".into(), json!(v));
    }
    if let Some(v) = a.start_date {
        payload.insert("start_date".into(), json!(v));
    }
    if let Some(v) = a.due_date {
        payload.insert("due_date".into(), json!(v));
    }
    if let Some(v) = a.estimated_hours {
        payload.insert("estimated_hours".into(), json!(v));
    }
    if let Some(v) = a.done_ratio {
        payload.insert("done_ratio".into(), json!(v));
    }
    if let Some(v) = a.target_version {
        payload.insert("fixed_version_id".into(), json!(v));
    }
    if let Some(v) = a.notes {
        payload.insert("notes".into(), json!(v));
    }
    if a.private_notes {
        payload.insert("private_notes".into(), json!(true));
    }
    let cf = cf_array(&a.custom_field, cfg);
    if !cf.is_empty() {
        payload.insert("custom_fields".into(), json!(cf));
    }
    if payload.is_empty() {
        output::print_error("redmine issue update: at least one field is required");
    }
    if let Err(e) = client.update_issue(id, Value::Object(payload)) {
        output::print_error(&format!("redmine issue update: {e}"));
    }
    // 갱신 본문 출력은 사용자 경험상 유지한다. PUT 성공 이후의 GET 실패는 별도 메시지로 구분.
    match client.get_issue(id, &["journals", "attachments", "children", "relations"]) {
        Ok(r) => output::print_json(serde_json::to_value(&r.issue).unwrap_or(json!({}))),
        Err(e) => output::print_error(&format!("update succeeded but fetch failed: {e}")),
    }
}

fn remove_relation(id: u64, client: &RedmineClient) {
    match client.delete_relation(id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("failed to remove relation: {e}")),
    }
}
