// users 서브커맨드 핸들러.
use clap::Args;
use serde::Serialize;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Args, Debug)]
pub struct UsersArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
}

#[derive(Serialize)]
struct UserOut {
    id: u64,
    login: Option<String>,
    firstname: Option<String>,
    lastname: Option<String>,
    mail: Option<String>,
}

pub fn handle(args: UsersArgs, client: &RedmineClient) {
    let resp = match client.search_users(&args.name, args.limit) {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine users: {e}")),
    };
    let out: Vec<UserOut> = resp
        .users
        .into_iter()
        .map(|u| UserOut {
            id: u.id,
            login: u.login,
            firstname: u.firstname,
            lastname: u.lastname,
            mail: u.mail,
        })
        .collect();
    output::print_json(json!({ "users": out, "total_count": resp.total_count }));
}
