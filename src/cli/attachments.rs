// attachment 서브커맨드 핸들러.
use clap::Subcommand;

use crate::client::RedmineClient;

#[derive(Subcommand, Debug)]
pub enum AttachmentCommand {
    List,
    Download,
    Upload,
    Delete,
}

pub fn handle(_cmd: AttachmentCommand, _client: &RedmineClient) {
    unimplemented!("attachments handler — Task 15");
}
