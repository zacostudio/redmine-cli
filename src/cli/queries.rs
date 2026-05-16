// query 서브커맨드 핸들러 (저장된 이슈 쿼리, REST 는 read-only).
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

pub fn handle(client: &RedmineClient) {
    match client.list_queries() {
        Ok(r) => {
            let items: Vec<_> = r
                .queries
                .iter()
                .map(|q| {
                    json!({
                        "id": q.id,
                        "name": q.name,
                        "is_public": q.is_public,
                        "project_id": q.project_id,
                    })
                })
                .collect();
            output::print_json(json!({ "queries": items, "total_count": r.total_count }));
        }
        Err(e) => output::print_error(&format!("redmine query: {e}")),
    }
}
