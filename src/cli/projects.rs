// projects 서브커맨드 핸들러.
use clap::Args;

use crate::client::RedmineClient;

#[derive(Args, Debug)]
pub struct ProjectsArgs {
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
}

pub fn handle(_args: ProjectsArgs, _client: &RedmineClient) {
    unimplemented!("projects handler — Task 9");
}
