// issues / issue 서브커맨드 핸들러.
use clap::{Args, Subcommand};

use crate::client::RedmineClient;
use crate::config::Config;

#[derive(Args, Debug)]
pub struct IssuesArgs {}

#[derive(Args, Debug)]
pub struct IssueArgs {
    pub id: Option<u64>,
    #[command(subcommand)]
    pub sub: Option<IssueSub>,
}

#[derive(Subcommand, Debug)]
pub enum IssueSub {
    Create,
    Update,
    Delete,
    Relations,
    AddRelation,
    RemoveRelation,
}

pub fn handle_search(_args: IssuesArgs, _client: &RedmineClient, _cfg: &Config) {
    unimplemented!("issues search — Task 11");
}

pub fn handle_one(_args: IssueArgs, _client: &RedmineClient, _cfg: &Config) {
    unimplemented!("issue handler — Task 11");
}
