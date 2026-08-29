// CLI 진입점. clap derive 로 정의된 Command 를 적절한 핸들러로 디스패치한다.
use clap::{Parser, Subcommand};

pub mod activities;
pub mod attachments;
pub mod categories;
pub mod config_cmd;
pub mod custom_fields;
pub mod enums;
pub mod files;
pub mod groups;
pub mod issues;
pub mod memberships;
pub mod my_account;
pub mod news;
pub mod projects;
pub mod queries;
pub mod roles;
pub mod search;
pub mod time_entries;
pub mod users;
pub mod versions;
pub mod wiki;

use crate::client::RedmineClient;
use crate::config::{self, CliOverrides, Config};
use crate::output;

#[derive(Parser)]
#[command(name = "redmine", version, about = "Standalone CLI for Redmine")]
pub struct Cli {
    /// Redmine server to use, by name from config.yml (defaults to default_server).
    #[arg(long, global = true)]
    pub server: Option<String>,

    /// Override the URL of the selected server.
    #[arg(long, global = true)]
    pub server_url: Option<String>,

    /// Override the API token of the selected server (also the token stored by
    /// `config server add`). Visible in ps and shell history — prefer
    /// --api-token-file or config.yml for regular use.
    #[arg(long, global = true)]
    pub api_token: Option<String>,

    /// Read the API token from a file instead of the command line.
    #[arg(long, global = true, conflicts_with = "api_token")]
    pub api_token_file: Option<std::path::PathBuf>,

    /// Path to config.yml (defaults to the per-user config dir).
    #[arg(long, global = true)]
    pub config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

// 토큰이 우연히라도 `{:?}` 로 출력되지 않도록 Debug 를 직접 구현해서 redact 한다.
impl std::fmt::Debug for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cli")
            .field("server", &self.server)
            .field("server_url", &self.server_url)
            .field("api_token", &self.api_token.as_ref().map(|_| "<REDACTED>"))
            .field("api_token_file", &self.api_token_file)
            .field("config", &self.config)
            .field("command", &self.command)
            .finish()
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List projects.
    Projects(projects::ProjectsArgs),
    /// List issue categories for a project.
    Categories(categories::CategoriesArgs),
    /// Search issues.
    Issues(issues::IssuesArgs),
    /// Operate on a single issue (get/create/update/delete/relations).
    Issue(issues::IssueArgs),
    /// Time entries: create/list/update/delete.
    #[command(name = "time-entry", subcommand)]
    TimeEntry(time_entries::TimeEntryCommand),
    /// Search users by name.
    Users(users::UsersArgs),
    /// List time-entry activities.
    Activities,
    /// List issue statuses.
    Statuses,
    /// List trackers.
    Trackers,
    /// List issue priorities.
    Priorities,
    /// List roles (admin only).
    Roles,
    /// List document categories.
    #[command(name = "document-categories")]
    DocumentCategories,
    /// List custom field definitions (admin only).
    #[command(name = "custom-fields")]
    CustomFields,
    /// Global search across issues, news, wiki, etc.
    Search(search::SearchArgs),
    /// Project versions (milestones): list/show/create/update/delete.
    #[command(subcommand)]
    Version(versions::VersionCommand),
    /// Project memberships: list/show/add/update/remove.
    #[command(subcommand)]
    Membership(memberships::MembershipCommand),
    /// News: list/show/create.
    #[command(subcommand)]
    News(news::NewsCommand),
    /// Project files: list/upload.
    #[command(subcommand)]
    File(files::FileCommand),
    /// List saved issue queries (read-only via REST API).
    Query,
    /// Wiki pages: list/show/create/update/delete.
    #[command(subcommand)]
    Wiki(wiki::WikiCommand),
    /// Groups (admin only): list/show/create/update/delete/add-user/remove-user.
    #[command(subcommand)]
    Group(groups::GroupCommand),
    /// Current user account: show/update.
    #[command(name = "my-account", subcommand)]
    MyAccount(my_account::MyAccountCommand),
    /// Attachments: list/download/upload/delete.
    #[command(subcommand)]
    Attachment(attachments::AttachmentCommand),
    /// Manage CLI configuration (servers, custom field aliases).
    #[command(subcommand)]
    Config(config_cmd::ConfigCommand),
}

/// --api-token-file 을 읽어 --api-token 과 같은 자리로 합친다. 토큰을 argv 에 올리지 않으려는
/// 경로이므로, 파일이 없거나 내용이 헤더로 쓸 수 없으면 여기서 끝낸다.
fn token_from_file(path: &std::path::Path) -> String {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => output::print_error(&format!(
            "failed to read API token file at {}: {e}",
            path.display()
        )),
    };
    let token = text.trim().to_string();
    if let Err(e) = config::validate_token(&token) {
        output::print_error(&format!("{} (from {})", e, path.display()));
    }
    token
}

pub fn run(cli: Cli) {
    let config_path_override = cli.config.clone();
    let api_token = match &cli.api_token_file {
        Some(p) => Some(token_from_file(p)),
        None => cli.api_token,
    };

    // config 서브커맨드는 Redmine 자격증명 없이 동작한다.
    if let Command::Config(sub) = cli.command {
        return config_cmd::handle(sub, config_path_override, cli.server, api_token);
    }

    let overrides = CliOverrides {
        server: cli.server,
        server_url: cli.server_url,
        api_token,
        config_path: cli.config,
    };
    let cfg: Config = match config::resolve(&overrides) {
        Ok(c) => c,
        Err(e) => output::print_error(&e.to_string()),
    };
    let client = match RedmineClient::new(&cfg.server_url, &cfg.api_token) {
        Ok(c) => c,
        Err(e) => output::print_error(&e.to_string()),
    };
    dispatch(cli.command, &client, &cfg);
}

fn dispatch(cmd: Command, client: &RedmineClient, cfg: &Config) {
    match cmd {
        Command::Projects(a) => projects::handle(a, client),
        Command::Categories(a) => categories::handle(a, client),
        Command::Issues(a) => issues::handle_search(a, client, cfg),
        Command::Issue(a) => issues::handle_one(a, client, cfg),
        Command::TimeEntry(sub) => time_entries::handle(sub, client),
        Command::Users(a) => users::handle(a, client),
        Command::Activities => activities::handle(client),
        Command::Statuses => enums::statuses(client),
        Command::Trackers => enums::trackers(client),
        Command::Priorities => enums::priorities(client),
        Command::Roles => roles::handle(client),
        Command::DocumentCategories => enums::document_categories(client),
        Command::CustomFields => custom_fields::handle(client),
        Command::Search(a) => search::handle(a, client),
        Command::Version(sub) => versions::handle(sub, client),
        Command::Membership(sub) => memberships::handle(sub, client),
        Command::News(sub) => news::handle(sub, client),
        Command::File(sub) => files::handle(sub, client),
        Command::Query => queries::handle(client),
        Command::Wiki(sub) => wiki::handle(sub, client),
        Command::Group(sub) => groups::handle(sub, client),
        Command::MyAccount(sub) => my_account::handle(sub, client),
        Command::Attachment(sub) => attachments::handle(sub, client),
        Command::Config(_) => unreachable!("handled in run() before client setup"),
    }
}
