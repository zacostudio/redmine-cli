// membership 서브커맨드 핸들러 (프로젝트 멤버 관리).
use clap::{Args, Subcommand};
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;
use crate::types::RedmineMembership;

#[derive(Subcommand, Debug)]
pub enum MembershipCommand {
    /// List memberships of a project.
    List(ListArgs),
    /// Show a single membership by id.
    Show(ShowArgs),
    /// Add a user or group to a project with one or more roles.
    Add(AddArgs),
    /// Update roles on an existing membership.
    Update(UpdateArgs),
    /// Remove a membership by id.
    Remove(RemoveArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    pub project: String,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    pub id: u64,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    pub project: String,
    /// User id (mutually exclusive with --group).
    #[arg(long, conflicts_with = "group")]
    pub user: Option<u64>,
    /// Group id (mutually exclusive with --user).
    #[arg(long)]
    pub group: Option<u64>,
    /// Role ids. Repeat or comma-separate.
    #[arg(long, value_delimiter = ',', required = true)]
    pub role: Vec<u64>,
    /// Print only the new membership ID followed by a newline.
    #[arg(long = "id-only", default_value_t = false)]
    pub id_only: bool,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    pub id: u64,
    /// Role ids. Repeat or comma-separate.
    #[arg(long, value_delimiter = ',', required = true)]
    pub role: Vec<u64>,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    pub id: u64,
}

pub fn handle(cmd: MembershipCommand, client: &RedmineClient) {
    match cmd {
        MembershipCommand::List(a) => list(a, client),
        MembershipCommand::Show(a) => show(a, client),
        MembershipCommand::Add(a) => add(a, client),
        MembershipCommand::Update(a) => update(a, client),
        MembershipCommand::Remove(a) => remove(a, client),
    }
}

fn membership_to_json(m: &RedmineMembership) -> Value {
    json!({
        "id": m.id,
        "project": m.project.as_ref().map(|p| &p.name),
        "user": m.user.as_ref().map(|u| json!({ "id": u.id, "name": u.name })),
        "group": m.group.as_ref().map(|g| json!({ "id": g.id, "name": g.name })),
        "roles": m.roles.as_ref().map(|rs| {
            rs.iter().map(|r| json!({ "id": r.id, "name": r.name })).collect::<Vec<_>>()
        }),
    })
}

fn list(a: ListArgs, client: &RedmineClient) {
    match client.list_memberships(&a.project) {
        Ok(r) => {
            let items: Vec<_> = r.memberships.iter().map(membership_to_json).collect();
            output::print_json(json!({ "memberships": items, "total_count": r.total_count }));
        }
        Err(e) => output::print_error(&format!("redmine membership list: {e}")),
    }
}

fn show(a: ShowArgs, client: &RedmineClient) {
    match client.get_membership(a.id) {
        Ok(r) => output::print_json(membership_to_json(&r.membership)),
        Err(e) => output::print_error(&format!("redmine membership show: {e}")),
    }
}

fn add(a: AddArgs, client: &RedmineClient) {
    if a.user.is_none() && a.group.is_none() {
        output::print_error("membership add: --user or --group is required");
    }
    let mut payload = serde_json::Map::new();
    if let Some(uid) = a.user {
        payload.insert("user_id".into(), json!(uid));
    }
    if let Some(gid) = a.group {
        // Redmine API: group memberships also use user_id field with the group's id.
        payload.insert("user_id".into(), json!(gid));
    }
    payload.insert("role_ids".into(), json!(a.role));

    match client.create_membership(&a.project, Value::Object(payload)) {
        Ok(r) => {
            if a.id_only {
                println!("{}", r.membership.id);
            } else {
                output::print_json(membership_to_json(&r.membership));
            }
        }
        Err(e) => output::print_error(&format!("redmine membership add: {e}")),
    }
}

fn update(a: UpdateArgs, client: &RedmineClient) {
    let payload = json!({ "role_ids": a.role });
    match client.update_membership(a.id, payload) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine membership update: {e}")),
    }
}

fn remove(a: RemoveArgs, client: &RedmineClient) {
    match client.delete_membership(a.id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("redmine membership remove: {e}")),
    }
}
