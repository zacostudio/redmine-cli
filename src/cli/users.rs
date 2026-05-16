// users 서브커맨드 핸들러.
use clap::Args;

use crate::client::RedmineClient;

#[derive(Args, Debug)]
pub struct UsersArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
}

pub fn handle(_args: UsersArgs, _client: &RedmineClient) {
    unimplemented!("users handler — Task 13");
}
