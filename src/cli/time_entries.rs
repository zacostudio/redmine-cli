// time-entry 서브커맨드 핸들러.
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;

#[derive(Subcommand, Debug)]
pub enum TimeEntryCommand {
    /// Create a new time entry.
    Create(CreateArgs),
    /// List time entries.
    List(ListArgs),
    /// Update a time entry.
    Update(UpdateArgs),
    /// Delete a time entry.
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[arg(long)]
    pub issue: u64,
    #[arg(long)]
    pub hours: f64,
    #[arg(long)]
    pub activity: Option<u64>,
    #[arg(long = "spent-on")]
    pub spent_on: Option<String>,
    #[arg(long)]
    pub comment: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long)]
    pub user: Option<String>,
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long)]
    pub issue: Option<String>,
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    pub id: u64,
    #[arg(long)]
    pub hours: Option<f64>,
    #[arg(long)]
    pub activity: Option<u64>,
    #[arg(long)]
    pub comment: Option<String>,
    #[arg(long = "spent-on")]
    pub spent_on: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    pub id: u64,
}

#[derive(Serialize)]
struct TimeEntryOut {
    id: u64,
    issue_id: Option<u64>,
    hours: f64,
    activity: Option<String>,
    spent_on: Option<String>,
    comments: Option<String>,
}

pub fn handle(cmd: TimeEntryCommand, client: &RedmineClient) {
    match cmd {
        TimeEntryCommand::Create(a) => create(a, client),
        TimeEntryCommand::List(a) => list(a, client),
        TimeEntryCommand::Update(a) => update(a, client),
        TimeEntryCommand::Delete(a) => delete(a, client),
    }
}

fn create(a: CreateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    payload.insert("issue_id".into(), json!(a.issue));
    payload.insert("hours".into(), json!(a.hours));
    if let Some(v) = a.activity {
        payload.insert("activity_id".into(), json!(v));
    }
    if let Some(v) = a.spent_on {
        payload.insert("spent_on".into(), json!(v));
    }
    if let Some(v) = a.comment {
        payload.insert("comments".into(), json!(v));
    }
    match client.create_time_entry(Value::Object(payload)) {
        Ok(r) => {
            let out = TimeEntryOut {
                id: r.time_entry.id,
                issue_id: r.time_entry.issue.map(|i| i.id),
                hours: r.time_entry.hours,
                activity: r.time_entry.activity.map(|x| x.name),
                spent_on: r.time_entry.spent_on,
                comments: r.time_entry.comments,
            };
            output::print_json(serde_json::to_value(&out).unwrap_or(json!({})));
        }
        Err(e) => output::print_error(&format!("redmine time-entry create: {e}")),
    }
}

fn list(a: ListArgs, client: &RedmineClient) {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(v) = a.user {
        params.push(("user_id", v));
    }
    if let Some(v) = a.project {
        params.push(("project_id", v));
    }
    if let Some(v) = a.issue {
        params.push(("issue_id", v));
    }
    if let Some(v) = a.from {
        params.push(("from", v));
    }
    if let Some(v) = a.to {
        params.push(("to", v));
    }
    params.push(("limit", a.limit.to_string()));
    match client.list_time_entries(&params) {
        Ok(r) => {
            let entries: Vec<Value> = r
                .time_entries
                .iter()
                .map(|te| {
                    json!({
                        "id": te.id,
                        "issue_id": te.issue.as_ref().map(|i| i.id),
                        "project": te.project.as_ref().map(|p| &p.name),
                        "user": te.user.as_ref().map(|u| &u.name),
                        "activity": te.activity.as_ref().map(|x| &x.name),
                        "hours": te.hours,
                        "comments": te.comments,
                        "spent_on": te.spent_on,
                    })
                })
                .collect();
            output::print_json(json!({ "time_entries": entries, "total_count": r.total_count }));
        }
        Err(e) => output::print_error(&format!("failed to list time entries: {e}")),
    }
}

fn update(a: UpdateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    if let Some(v) = a.hours {
        payload.insert("hours".into(), json!(v));
    }
    if let Some(v) = a.activity {
        payload.insert("activity_id".into(), json!(v));
    }
    if let Some(v) = a.comment {
        payload.insert("comments".into(), json!(v));
    }
    if let Some(v) = a.spent_on {
        payload.insert("spent_on".into(), json!(v));
    }
    if payload.is_empty() {
        output::print_error("time-entry update: no fields to update");
    }
    match client.update_time_entry(a.id, Value::Object(payload)) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("failed to update time entry: {e}")),
    }
}

fn delete(a: DeleteArgs, client: &RedmineClient) {
    match client.delete_time_entry(a.id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("failed to delete time entry: {e}")),
    }
}
