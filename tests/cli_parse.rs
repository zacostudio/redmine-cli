// clap derive 가 의도대로 인자를 해석하는지 확인.
use clap::Parser;
use redmine_cli::cli::{Cli, Command};

fn parse(args: &[&str]) -> Cli {
    let mut full = vec!["redmine"];
    full.extend(args.iter().copied());
    Cli::try_parse_from(full).expect("parse")
}

#[test]
fn parses_projects_with_defaults() {
    let cli = parse(&["projects"]);
    match cli.command {
        Command::Projects(a) => {
            assert_eq!(a.limit, 25);
            assert_eq!(a.offset, 0);
        }
        _ => panic!("expected Projects"),
    }
}

#[test]
fn parses_issues_with_filters() {
    let cli = parse(&[
        "issues",
        "--project",
        "demo",
        "--status",
        "1",
        "--query",
        "bug",
        "--custom-field",
        "7=Dev",
    ]);
    match cli.command {
        Command::Issues(a) => {
            assert_eq!(a.project.as_deref(), Some("demo"));
            assert_eq!(a.status.as_deref(), Some("1"));
            assert_eq!(a.query.as_deref(), Some("bug"));
            assert_eq!(a.custom_field, vec!["7=Dev"]);
        }
        _ => panic!("expected Issues"),
    }
}

#[test]
fn parses_single_issue_by_id() {
    let cli = parse(&["issue", "123"]);
    match cli.command {
        Command::Issue(a) => {
            assert_eq!(a.id, Some(123));
            assert!(a.sub.is_none());
        }
        _ => panic!("expected Issue"),
    }
}

#[test]
fn parses_issue_create_without_id() {
    let cli = parse(&["issue", "create", "--project", "demo", "--subject", "hi"]);
    match cli.command {
        Command::Issue(a) => {
            assert!(a.id.is_none());
            match a.sub {
                Some(redmine_cli::cli::issues::IssueSub::Create(c)) => {
                    // Default for --id-only is false — full JSON output unless explicitly opted in.
                    assert!(!c.id_only);
                }
                _ => panic!("expected Issue Create"),
            }
        }
        _ => panic!("expected Issue"),
    }
}

#[test]
fn parses_issue_create_with_id_only() {
    let cli = parse(&[
        "issue", "create", "--project", "demo", "--subject", "hi", "--id-only",
    ]);
    match cli.command {
        Command::Issue(a) => match a.sub {
            Some(redmine_cli::cli::issues::IssueSub::Create(c)) => {
                assert!(c.id_only);
            }
            _ => panic!("expected Issue Create"),
        },
        _ => panic!("expected Issue"),
    }
}

#[test]
fn parses_time_entry_create() {
    let cli = parse(&["time-entry", "create", "--issue", "10", "--hours", "1.5"]);
    match cli.command {
        Command::TimeEntry(redmine_cli::cli::time_entries::TimeEntryCommand::Create(a)) => {
            assert_eq!(a.issue, 10);
            assert!((a.hours - 1.5).abs() < f64::EPSILON);
        }
        _ => panic!("expected TimeEntry::Create"),
    }
}

#[test]
fn parses_config_alias_list() {
    let cli = parse(&["config", "alias", "list"]);
    match cli.command {
        Command::Config(redmine_cli::cli::config_cmd::ConfigCommand::Alias(
            redmine_cli::cli::config_cmd::AliasCommand::List,
        )) => {}
        _ => panic!("expected Config::Alias::List"),
    }
}

#[test]
fn parses_config_alias_set() {
    let cli = parse(&["config", "alias", "set", "state", "7"]);
    match cli.command {
        Command::Config(redmine_cli::cli::config_cmd::ConfigCommand::Alias(
            redmine_cli::cli::config_cmd::AliasCommand::Set { name, id },
        )) => {
            assert_eq!(name, "state");
            assert_eq!(id, 7);
        }
        _ => panic!("expected Config::Alias::Set"),
    }
}

#[test]
fn parses_config_alias_remove() {
    let cli = parse(&["config", "alias", "remove", "state"]);
    match cli.command {
        Command::Config(redmine_cli::cli::config_cmd::ConfigCommand::Alias(
            redmine_cli::cli::config_cmd::AliasCommand::Remove { name },
        )) => {
            assert_eq!(name, "state");
        }
        _ => panic!("expected Config::Alias::Remove"),
    }
}

#[test]
fn parses_roles() {
    let cli = parse(&["roles"]);
    assert!(matches!(cli.command, Command::Roles));
}

#[test]
fn parses_document_categories() {
    let cli = parse(&["document-categories"]);
    assert!(matches!(cli.command, Command::DocumentCategories));
}

#[test]
fn parses_custom_fields() {
    let cli = parse(&["custom-fields"]);
    assert!(matches!(cli.command, Command::CustomFields));
}

#[test]
fn parses_search() {
    let cli = parse(&["search", "hello", "--scope", "issues", "--limit", "5"]);
    match cli.command {
        Command::Search(a) => {
            assert_eq!(a.query, "hello");
            assert_eq!(a.scope.as_deref(), Some("issues"));
            assert_eq!(a.limit, 5);
        }
        _ => panic!("expected Search"),
    }
}

#[test]
fn parses_version_list() {
    let cli = parse(&["version", "list", "demo"]);
    match cli.command {
        Command::Version(redmine_cli::cli::versions::VersionCommand::List(a)) => {
            assert_eq!(a.project, "demo");
        }
        _ => panic!("expected Version::List"),
    }
}

#[test]
fn parses_version_create() {
    let cli = parse(&["version", "create", "demo", "--name", "v2", "--due-date", "2026-12-31"]);
    match cli.command {
        Command::Version(redmine_cli::cli::versions::VersionCommand::Create(a)) => {
            assert_eq!(a.project, "demo");
            assert_eq!(a.name, "v2");
            assert_eq!(a.due_date.as_deref(), Some("2026-12-31"));
        }
        _ => panic!("expected Version::Create"),
    }
}

#[test]
fn parses_membership_add_with_multi_role() {
    let cli = parse(&[
        "membership", "add", "demo", "--user", "11", "--role", "4,5", "--role", "6",
    ]);
    match cli.command {
        Command::Membership(redmine_cli::cli::memberships::MembershipCommand::Add(a)) => {
            assert_eq!(a.project, "demo");
            assert_eq!(a.user, Some(11));
            assert_eq!(a.role, vec![4, 5, 6]);
        }
        _ => panic!("expected Membership::Add"),
    }
}

#[test]
fn parses_membership_remove() {
    let cli = parse(&["membership", "remove", "42"]);
    match cli.command {
        Command::Membership(redmine_cli::cli::memberships::MembershipCommand::Remove(a)) => {
            assert_eq!(a.id, 42);
        }
        _ => panic!("expected Membership::Remove"),
    }
}
