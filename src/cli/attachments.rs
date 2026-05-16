// attachment 서브커맨드 핸들러.
use clap::{Args, Subcommand};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::client::RedmineClient;
use crate::output;

#[derive(Subcommand, Debug)]
pub enum AttachmentCommand {
    /// List attachments of an issue.
    List(ListArgs),
    /// Download an attachment by id.
    Download(DownloadArgs),
    /// Upload a file and attach to an issue.
    Upload(UploadArgs),
    /// Delete an attachment.
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long)]
    pub issue: u64,
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Attachment ID.
    pub id: u64,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Args, Debug)]
pub struct UploadArgs {
    #[arg(long)]
    pub issue: u64,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Attachment ID.
    pub id: u64,
}

pub fn handle(cmd: AttachmentCommand, client: &RedmineClient) {
    match cmd {
        AttachmentCommand::List(a) => list(a, client),
        AttachmentCommand::Download(a) => download(a, client),
        AttachmentCommand::Upload(a) => upload(a, client),
        AttachmentCommand::Delete(a) => delete(a, client),
    }
}

fn list(a: ListArgs, client: &RedmineClient) {
    match client.get_issue(a.issue) {
        Ok(r) => {
            let attachments = r.issue.attachments.unwrap_or_default();
            let items: Vec<Value> = attachments
                .iter()
                .map(|x| {
                    json!({
                        "id": x.id,
                        "filename": x.filename,
                        "filesize": x.filesize,
                        "content_url": x.content_url,
                        "author": x.author.as_ref().map(|au| &au.name),
                        "created_on": x.created_on,
                    })
                })
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("failed to get attachments: {e}")),
    }
}

fn download(a: DownloadArgs, client: &RedmineClient) {
    let info_path = format!("/attachments/{}.json", a.id);
    let val: Value = match client.get(&info_path, &[]) {
        Ok(v) => v,
        Err(e) => output::print_error(&format!("failed to get attachment info: {e}")),
    };
    let url = val
        .get("attachment")
        .and_then(|x| x.get("content_url"))
        .and_then(|u| u.as_str())
        .unwrap_or_else(|| output::print_error("attachment: content_url not found"));
    match client.download_attachment(url, &a.output) {
        Ok(()) => output::print_json(json!({ "ok": true, "path": a.output.display().to_string() })),
        Err(e) => output::print_error(&format!("download failed: {e}")),
    }
}

fn upload(a: UploadArgs, client: &RedmineClient) {
    if !a.file.exists() {
        output::print_error(&format!("file not found: {}", a.file.display()));
    }
    let filename = a
        .file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload");
    let content_type = "application/octet-stream";
    match client.upload_file(&a.file) {
        Ok(upload_resp) => match client.attach_to_issue(
            a.issue,
            &upload_resp.upload.token,
            filename,
            content_type,
            a.description.as_deref(),
        ) {
            Ok(()) => output::print_json(json!({ "ok": true, "token": upload_resp.upload.token })),
            Err(e) => output::print_error(&format!("failed to attach: {e}")),
        },
        Err(e) => output::print_error(&format!("upload failed: {e}")),
    }
}

fn delete(a: DeleteArgs, client: &RedmineClient) {
    match client.delete_attachment(a.id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("failed to delete attachment: {e}")),
    }
}
