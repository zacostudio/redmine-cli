// wiki 서브커맨드 핸들러 (프로젝트 위키 페이지 CRUD).
use clap::{Args, Subcommand};
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;
use crate::types::RedmineWikiPage;

#[derive(Subcommand, Debug)]
pub enum WikiCommand {
    /// List wiki pages in a project.
    List(ListArgs),
    /// Show a single wiki page (full text + metadata).
    Show(ShowArgs),
    /// Create a new wiki page (PUT, 4xx if it already exists).
    Create(WriteArgs),
    /// Update an existing wiki page (same PUT endpoint; convenient alias).
    Update(WriteArgs),
    /// Delete a wiki page.
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    pub project: String,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    pub project: String,
    pub title: String,
}

#[derive(Args, Debug)]
pub struct WriteArgs {
    pub project: String,
    pub title: String,
    /// Page text. Use "-" to read from stdin.
    #[arg(long)]
    pub text: String,
    /// Optional change comment.
    #[arg(long)]
    pub comments: Option<String>,
    /// Optional parent page title.
    #[arg(long = "parent-title")]
    pub parent_title: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    pub project: String,
    pub title: String,
}

pub fn handle(cmd: WikiCommand, client: &RedmineClient) {
    match cmd {
        WikiCommand::List(a) => list(a, client),
        WikiCommand::Show(a) => show(a, client),
        WikiCommand::Create(a) => put(a, client),
        WikiCommand::Update(a) => put(a, client),
        WikiCommand::Delete(a) => delete(a, client),
    }
}

fn page_summary_json(p: &RedmineWikiPage) -> Value {
    json!({
        "title": p.title,
        "parent": p.parent.as_ref().map(|x| &x.title),
        "version": p.version,
        "created_on": p.created_on,
        "updated_on": p.updated_on,
    })
}

fn page_full_json(p: &RedmineWikiPage) -> Value {
    json!({
        "title": p.title,
        "parent": p.parent.as_ref().map(|x| &x.title),
        "text": p.text,
        "version": p.version,
        "author": p.author.as_ref().map(|a| &a.name),
        "comments": p.comments,
        "created_on": p.created_on,
        "updated_on": p.updated_on,
        "attachments": p.attachments,
    })
}

fn list(a: ListArgs, client: &RedmineClient) {
    match client.list_wiki_pages(&a.project) {
        Ok(r) => {
            let items: Vec<_> = r.wiki_pages.iter().map(page_summary_json).collect();
            output::print_json(json!({ "wiki_pages": items }));
        }
        Err(e) => output::print_error(&format!("redmine wiki list: {e}")),
    }
}

fn show(a: ShowArgs, client: &RedmineClient) {
    match client.get_wiki_page(&a.project, &a.title) {
        Ok(r) => output::print_json(page_full_json(&r.wiki_page)),
        Err(e) => output::print_error(&format!("redmine wiki show: {e}")),
    }
}

fn put(a: WriteArgs, client: &RedmineClient) {
    let text = if a.text == "-" {
        match output::read_stdin() {
            Ok(s) => s,
            Err(e) => output::print_error(&format!("redmine wiki: {e}")),
        }
    } else {
        a.text
    };

    let mut payload = serde_json::Map::new();
    payload.insert("text".into(), json!(text));
    if let Some(v) = a.comments {
        payload.insert("comments".into(), json!(v));
    }
    if let Some(v) = a.parent_title {
        payload.insert("parent_title".into(), json!(v));
    }

    match client.put_wiki_page(&a.project, &a.title, Value::Object(payload)) {
        Ok(()) => output::print_json(json!({ "ok": true, "title": a.title })),
        Err(e) => output::print_error(&format!("redmine wiki put: {e}")),
    }
}

fn delete(a: DeleteArgs, client: &RedmineClient) {
    match client.delete_wiki_page(&a.project, &a.title) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine wiki delete: {e}")),
    }
}
