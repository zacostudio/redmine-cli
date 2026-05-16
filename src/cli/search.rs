// search 서브커맨드 핸들러 (전역 검색).
use clap::Args;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query string.
    pub query: String,
    /// Restrict scope: e.g. issues, news, documents, wiki_pages, projects.
    #[arg(long)]
    pub scope: Option<String>,
    /// Match all words (default: any).
    #[arg(long)]
    pub all_words: bool,
    /// Search in titles only.
    #[arg(long)]
    pub titles_only: bool,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
}

pub fn handle(args: SearchArgs, client: &RedmineClient) {
    let mut params: Vec<(&str, String)> = vec![
        ("q", args.query),
        ("limit", args.limit.to_string()),
        ("offset", args.offset.to_string()),
    ];
    if let Some(scope) = args.scope {
        params.push(("scope", scope));
    }
    if args.all_words {
        params.push(("all_words", "1".to_string()));
    }
    if args.titles_only {
        params.push(("titles_only", "1".to_string()));
    }

    match client.search(&params) {
        Ok(r) => {
            let results: Vec<_> = r
                .results
                .iter()
                .map(|x| {
                    json!({
                        "id": x.id,
                        "title": x.title,
                        "type": x.kind,
                        "url": x.url,
                        "description": x.description,
                        "datetime": x.datetime,
                    })
                })
                .collect();
            output::print_json(json!({
                "results": results,
                "total_count": r.total_count,
                "limit": r.limit,
                "offset": r.offset,
            }));
        }
        Err(e) => output::print_error(&format!("redmine search: {e}")),
    }
}
