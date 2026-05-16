// custom-fields 서브커맨드 핸들러 (admin 전용 메타데이터 조회).
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

pub fn handle(client: &RedmineClient) {
    match client.list_custom_fields() {
        Ok(r) => {
            let items: Vec<_> = r
                .custom_fields
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "name": c.name,
                        "customized_type": c.customized_type,
                        "field_format": c.field_format,
                        "is_required": c.is_required,
                        "is_filter": c.is_filter,
                        "multiple": c.multiple,
                        "default_value": c.default_value,
                        "visible": c.visible,
                        "possible_values": c.possible_values,
                    })
                })
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine custom-fields: {e}")),
    }
}
