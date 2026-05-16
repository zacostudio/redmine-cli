// news 서브커맨드 핸들러 (프로젝트 공지사항).
use clap::{Args, Subcommand};
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;
use crate::types::RedmineNews;

#[derive(Subcommand, Debug)]
pub enum NewsCommand {
    /// List news (global or by project).
    List(ListArgs),
    /// Show a single news item by id.
    Show(ShowArgs),
    /// Create a news item under a project (Redmine 5+ only).
    Create(CreateArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Restrict to a project identifier or id.
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    pub id: u64,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    pub project: String,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub summary: Option<String>,
    /// Print only the new news ID followed by a newline.
    #[arg(long = "id-only", default_value_t = false)]
    pub id_only: bool,
}

pub fn handle(cmd: NewsCommand, client: &RedmineClient) {
    match cmd {
        NewsCommand::List(a) => list(a, client),
        NewsCommand::Show(a) => show(a, client),
        NewsCommand::Create(a) => create(a, client),
    }
}

fn news_to_json(n: &RedmineNews) -> Value {
    json!({
        "id": n.id,
        "project": n.project.as_ref().map(|p| &p.name),
        "author": n.author.as_ref().map(|a| &a.name),
        "title": n.title,
        "summary": n.summary,
        "description": n.description,
        "created_on": n.created_on,
    })
}

fn list(a: ListArgs, client: &RedmineClient) {
    let params = vec![
        ("limit", a.limit.to_string()),
        ("offset", a.offset.to_string()),
    ];
    match client.list_news(a.project.as_deref(), &params) {
        Ok(r) => {
            let items: Vec<_> = r.news.iter().map(news_to_json).collect();
            output::print_json(json!({ "news": items, "total_count": r.total_count }));
        }
        Err(e) => output::print_error(&format!("redmine news list: {e}")),
    }
}

fn show(a: ShowArgs, client: &RedmineClient) {
    match client.get_news(a.id) {
        Ok(r) => output::print_json(news_to_json(&r.news)),
        Err(e) => output::print_error(&format!("redmine news show: {e}")),
    }
}

fn create(a: CreateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    payload.insert("title".into(), json!(a.title));
    if let Some(v) = a.description {
        payload.insert("description".into(), json!(v));
    }
    if let Some(v) = a.summary {
        payload.insert("summary".into(), json!(v));
    }
    match client.create_news(&a.project, Value::Object(payload)) {
        Ok(r) => {
            if a.id_only {
                println!("{}", r.news.id);
            } else {
                output::print_json(news_to_json(&r.news));
            }
        }
        Err(e) => output::print_error(&format!("redmine news create: {e}")),
    }
}
