// group 서브커맨드 핸들러 (admin 전용 그룹 관리).
use clap::{Args, Subcommand};
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;
use crate::types::RedmineGroup;

#[derive(Subcommand, Debug)]
pub enum GroupCommand {
    /// List all groups (admin only).
    List,
    /// Show a single group (users + memberships included).
    Show(IdArgs),
    /// Create a new group.
    Create(CreateArgs),
    /// Update a group's name.
    Update(UpdateArgs),
    /// Delete a group.
    Delete(IdArgs),
    /// Add a user to a group.
    #[command(name = "add-user")]
    AddUser(UserOpArgs),
    /// Remove a user from a group.
    #[command(name = "remove-user")]
    RemoveUser(UserOpArgs),
}

#[derive(Args, Debug)]
pub struct IdArgs {
    pub id: u64,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[arg(long)]
    pub name: String,
    /// Initial user ids. Repeat or comma-separate.
    #[arg(long = "user", value_delimiter = ',')]
    pub users: Vec<u64>,
    /// Print only the new group ID followed by a newline.
    #[arg(long = "id-only", default_value_t = false)]
    pub id_only: bool,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    pub id: u64,
    #[arg(long)]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct UserOpArgs {
    /// Group id.
    pub group: u64,
    #[arg(long = "user")]
    pub user: u64,
}

pub fn handle(cmd: GroupCommand, client: &RedmineClient) {
    match cmd {
        GroupCommand::List => list(client),
        GroupCommand::Show(a) => show(a, client),
        GroupCommand::Create(a) => create(a, client),
        GroupCommand::Update(a) => update(a, client),
        GroupCommand::Delete(a) => delete(a, client),
        GroupCommand::AddUser(a) => add_user(a, client),
        GroupCommand::RemoveUser(a) => remove_user(a, client),
    }
}

fn group_to_json(g: &RedmineGroup) -> Value {
    json!({
        "id": g.id,
        "name": g.name,
        "users": g.users.as_ref().map(|us| {
            us.iter().map(|u| json!({ "id": u.id, "name": u.name })).collect::<Vec<_>>()
        }),
        "memberships": g.memberships,
    })
}

fn list(client: &RedmineClient) {
    match client.list_groups() {
        Ok(r) => {
            let items: Vec<_> = r.groups.iter().map(group_to_json).collect();
            output::print_json(json!({ "groups": items }));
        }
        Err(e) => output::print_error(&format!("redmine group list: {e}")),
    }
}

fn show(a: IdArgs, client: &RedmineClient) {
    match client.get_group(a.id) {
        Ok(r) => output::print_json(group_to_json(&r.group)),
        Err(e) => output::print_error(&format!("redmine group show: {e}")),
    }
}

fn create(a: CreateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    payload.insert("name".into(), json!(a.name));
    if !a.users.is_empty() {
        payload.insert("user_ids".into(), json!(a.users));
    }
    match client.create_group(Value::Object(payload)) {
        Ok(r) => {
            if a.id_only {
                println!("{}", r.group.id);
            } else {
                output::print_json(group_to_json(&r.group));
            }
        }
        Err(e) => output::print_error(&format!("redmine group create: {e}")),
    }
}

fn update(a: UpdateArgs, client: &RedmineClient) {
    match client.update_group(a.id, json!({ "name": a.name })) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine group update: {e}")),
    }
}

fn delete(a: IdArgs, client: &RedmineClient) {
    match client.delete_group(a.id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine group delete: {e}")),
    }
}

fn add_user(a: UserOpArgs, client: &RedmineClient) {
    match client.add_user_to_group(a.group, a.user) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine group add-user: {e}")),
    }
}

fn remove_user(a: UserOpArgs, client: &RedmineClient) {
    match client.remove_user_from_group(a.group, a.user) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine group remove-user: {e}")),
    }
}
