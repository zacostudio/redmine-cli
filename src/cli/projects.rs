// projects 서브커맨드 핸들러.
use clap::Args;
use serde::Serialize;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Args, Debug)]
pub struct ProjectsArgs {
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
}

#[derive(Serialize)]
struct ProjectOut {
    id: u64,
    name: String,
    identifier: String,
    description: Option<String>,
    status: Option<u32>,
    created_on: Option<String>,
}

pub fn handle(args: ProjectsArgs, client: &RedmineClient) {
    let resp = match client.list_projects(args.limit, args.offset) {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine projects: {e}")),
    };
    let out: Vec<ProjectOut> = resp
        .projects
        .into_iter()
        .map(|p| ProjectOut {
            id: p.id,
            name: p.name,
            identifier: p.identifier,
            description: p.description,
            status: p.status,
            created_on: p.created_on,
        })
        .collect();
    output::print_json(json!({ "projects": out, "total_count": resp.total_count }));
}
