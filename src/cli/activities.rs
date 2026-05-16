// activities 서브커맨드 핸들러.
use serde::Serialize;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Serialize)]
struct ActivityOut {
    id: u64,
    name: String,
    is_default: Option<bool>,
}

pub fn handle(client: &RedmineClient) {
    let resp = match client.list_activities() {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine activities: {e}")),
    };
    let out: Vec<ActivityOut> = resp
        .time_entry_activities
        .into_iter()
        .map(|a| ActivityOut {
            id: a.id,
            name: a.name,
            is_default: a.is_default,
        })
        .collect();
    output::print_json(json!({ "activities": out }));
}
