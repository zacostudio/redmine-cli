// categories 서브커맨드 핸들러.
use clap::Args;
use serde::Serialize;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Args, Debug)]
pub struct CategoriesArgs {
    /// Project identifier or numeric id.
    #[arg(long)]
    pub project: String,
}

#[derive(Serialize)]
struct CategoryOut {
    id: u64,
    name: String,
    assigned_to: Option<String>,
}

pub fn handle(args: CategoriesArgs, client: &RedmineClient) {
    let resp = match client.list_categories(&args.project) {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine categories: {e}")),
    };
    let out: Vec<CategoryOut> = resp
        .issue_categories
        .into_iter()
        .map(|c| CategoryOut {
            id: c.id,
            name: c.name,
            assigned_to: c.assigned_to.map(|a| a.name),
        })
        .collect();
    output::print_json(json!({ "categories": out, "total_count": resp.total_count }));
}
