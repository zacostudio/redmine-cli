// my-account 서브커맨드 핸들러 (현재 사용자 프로필 조회/수정).
use clap::{Args, Subcommand};
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;
use crate::types::RedmineUser;

#[derive(Subcommand, Debug)]
pub enum MyAccountCommand {
    /// Show the currently authenticated user.
    Show,
    /// Update profile fields (firstname/lastname/mail).
    Update(UpdateArgs),
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    #[arg(long)]
    pub firstname: Option<String>,
    #[arg(long)]
    pub lastname: Option<String>,
    #[arg(long)]
    pub mail: Option<String>,
}

pub fn handle(cmd: MyAccountCommand, client: &RedmineClient) {
    match cmd {
        MyAccountCommand::Show => show(client),
        MyAccountCommand::Update(a) => update(a, client),
    }
}

fn user_to_json(u: &RedmineUser) -> Value {
    json!({
        "id": u.id,
        "login": u.login,
        "firstname": u.firstname,
        "lastname": u.lastname,
        "mail": u.mail,
        "admin": u.admin,
        "created_on": u.created_on,
        "last_login_on": u.last_login_on,
    })
}

fn show(client: &RedmineClient) {
    match client.get_my_account() {
        Ok(r) => output::print_json(user_to_json(&r.user)),
        Err(e) => output::print_error(&format!("redmine my-account show: {e}")),
    }
}

fn update(a: UpdateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    if let Some(v) = a.firstname {
        payload.insert("firstname".into(), json!(v));
    }
    if let Some(v) = a.lastname {
        payload.insert("lastname".into(), json!(v));
    }
    if let Some(v) = a.mail {
        payload.insert("mail".into(), json!(v));
    }
    if payload.is_empty() {
        output::print_error("my-account update: no fields to update");
    }
    match client.update_my_account(Value::Object(payload)) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine my-account update: {e}")),
    }
}
