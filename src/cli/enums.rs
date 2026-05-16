// statuses / trackers / priorities 서브커맨드 핸들러.
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

pub fn statuses(client: &RedmineClient) {
    match client.list_statuses() {
        Ok(r) => {
            let items: Vec<_> = r
                .issue_statuses
                .iter()
                .map(|s| json!({ "id": s.id, "name": s.name, "is_closed": s.is_closed }))
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine statuses: {e}")),
    }
}

pub fn trackers(client: &RedmineClient) {
    match client.list_trackers() {
        Ok(r) => {
            let items: Vec<_> = r
                .trackers
                .iter()
                .map(|t| json!({ "id": t.id, "name": t.name }))
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine trackers: {e}")),
    }
}

pub fn priorities(client: &RedmineClient) {
    match client.list_priorities() {
        Ok(r) => {
            let items: Vec<_> = r
                .issue_priorities
                .iter()
                .map(|p| json!({ "id": p.id, "name": p.name, "is_default": p.is_default }))
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine priorities: {e}")),
    }
}

pub fn document_categories(client: &RedmineClient) {
    match client.list_document_categories() {
        Ok(r) => {
            let items: Vec<_> = r
                .document_categories
                .iter()
                .map(|c| json!({ "id": c.id, "name": c.name, "is_default": c.is_default }))
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine document-categories: {e}")),
    }
}
