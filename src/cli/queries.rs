// query 서브커맨드 핸들러 (저장된 이슈 쿼리, REST 는 read-only).
use clap::Args;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Maximum rows to return. Redmine caps this at 100.
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    /// Rows to skip, for paging past the first `--limit` rows.
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
}

pub fn handle(args: QueryArgs, client: &RedmineClient) {
    match client.list_queries(args.limit, args.offset) {
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
