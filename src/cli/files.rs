// file 서브커맨드 핸들러 (프로젝트 파일 보관함).
use clap::{Args, Subcommand};
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;
use crate::types::RedmineFile;

#[derive(Subcommand, Debug)]
pub enum FileCommand {
    /// List files attached to a project.
    List(ListArgs),
    /// Upload a local file to a project.
    Upload(UploadArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    pub project: String,
}

#[derive(Args, Debug)]
pub struct UploadArgs {
    pub project: String,
    /// Local path to upload.
    #[arg(long)]
    pub path: std::path::PathBuf,
    /// Optional description shown in the file list.
    #[arg(long)]
    pub description: Option<String>,
    /// Optional version id to associate the file with.
    #[arg(long = "version-id")]
    pub version_id: Option<u64>,
}

pub fn handle(cmd: FileCommand, client: &RedmineClient) {
    match cmd {
        FileCommand::List(a) => list(a, client),
        FileCommand::Upload(a) => upload(a, client),
    }
}

fn file_to_json(f: &RedmineFile) -> Value {
    json!({
        "id": f.id,
        "filename": f.filename,
        "filesize": f.filesize,
        "content_type": f.content_type,
        "description": f.description,
        "content_url": f.content_url,
        "author": f.author.as_ref().map(|a| &a.name),
        "created_on": f.created_on,
        "version": f.version.as_ref().map(|v| json!({ "id": v.id, "name": v.name })),
        "digest": f.digest,
        "downloads": f.downloads,
    })
}

fn list(a: ListArgs, client: &RedmineClient) {
    match client.list_files(&a.project) {
        Ok(r) => {
            let items: Vec<_> = r.files.iter().map(file_to_json).collect();
            output::print_json(json!({ "files": items }));
        }
        Err(e) => output::print_error(&format!("redmine file list: {e}")),
    }
}

fn upload(a: UploadArgs, client: &RedmineClient) {
    let filename = a
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload")
        .to_string();

    let token_resp = match client.upload_file(&a.path) {
        Ok(t) => t,
        Err(e) => output::print_error(&format!("redmine file upload (token): {e}")),
    };

    let mut payload = serde_json::Map::new();
    payload.insert("token".into(), json!(token_resp.upload.token));
    payload.insert("filename".into(), json!(filename));
    if let Some(v) = a.description {
        payload.insert("description".into(), json!(v));
    }
    if let Some(v) = a.version_id {
        payload.insert("version_id".into(), json!(v));
    }
    match client.attach_file_to_project(&a.project, Value::Object(payload)) {
        Ok(()) => output::print_json(json!({ "ok": true, "filename": filename })),
        Err(e) => output::print_error(&format!("redmine file upload (attach): {e}")),
    }
}
