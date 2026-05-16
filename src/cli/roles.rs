// roles 서브커맨드 핸들러 (단순 enum 목록).
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

pub fn handle(client: &RedmineClient) {
    match client.list_roles() {
        Ok(r) => {
            let items: Vec<_> = r
                .roles
                .iter()
                .map(|x| json!({ "id": x.id, "name": x.name }))
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine roles: {e}")),
    }
}
