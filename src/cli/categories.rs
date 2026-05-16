// categories 서브커맨드 핸들러.
use clap::Args;

use crate::client::RedmineClient;

#[derive(Args, Debug)]
pub struct CategoriesArgs {
    #[arg(long)]
    pub project: String,
}

pub fn handle(_args: CategoriesArgs, _client: &RedmineClient) {
    unimplemented!("categories handler — Task 10");
}
