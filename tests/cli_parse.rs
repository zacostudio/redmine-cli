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

#[test]
fn parses_news_list_with_project() {
    let cli = parse(&["news", "list", "--project", "demo", "--limit", "5"]);
    match cli.command {
        Command::News(redmine_cli::cli::news::NewsCommand::List(a)) => {
            assert_eq!(a.project.as_deref(), Some("demo"));
            assert_eq!(a.limit, 5);
        }
        _ => panic!("expected News::List"),
    }
}

#[test]
fn parses_news_create() {
    let cli = parse(&["news", "create", "demo", "--title", "Hi"]);
    match cli.command {
        Command::News(redmine_cli::cli::news::NewsCommand::Create(a)) => {
            assert_eq!(a.project, "demo");
            assert_eq!(a.title, "Hi");
        }
        _ => panic!("expected News::Create"),
    }
}

#[test]
fn parses_file_list() {
    let cli = parse(&["file", "list", "demo"]);
    match cli.command {
        Command::File(redmine_cli::cli::files::FileCommand::List(a)) => {
            assert_eq!(a.project, "demo");
        }
        _ => panic!("expected File::List"),
    }
}

#[test]
fn parses_file_upload() {
    let cli = parse(&[
        "file", "upload", "demo", "--path", "/tmp/x.bin", "--description", "d",
    ]);
    match cli.command {
        Command::File(redmine_cli::cli::files::FileCommand::Upload(a)) => {
            assert_eq!(a.project, "demo");
            assert_eq!(a.path.as_os_str(), "/tmp/x.bin");
            assert_eq!(a.description.as_deref(), Some("d"));
        }
        _ => panic!("expected File::Upload"),
    }
}

#[test]
fn parses_query() {
    let cli = parse(&["query"]);
    assert!(matches!(cli.command, Command::Query));
}

#[test]
fn parses_wiki_show() {
    let cli = parse(&["wiki", "show", "demo", "Roadmap"]);
    match cli.command {
        Command::Wiki(redmine_cli::cli::wiki::WikiCommand::Show(a)) => {
            assert_eq!(a.project, "demo");
            assert_eq!(a.title, "Roadmap");
        }
        _ => panic!("expected Wiki::Show"),
    }
}

#[test]
fn parses_wiki_update_with_text() {
    let cli = parse(&[
        "wiki", "update", "demo", "Roadmap", "--text", "hi", "--comments", "x",
    ]);
    match cli.command {
        Command::Wiki(redmine_cli::cli::wiki::WikiCommand::Update(a)) => {
            assert_eq!(a.project, "demo");
            assert_eq!(a.title, "Roadmap");
            assert_eq!(a.text, "hi");
            assert_eq!(a.comments.as_deref(), Some("x"));
        }
        _ => panic!("expected Wiki::Update"),
    }
}

#[test]
fn parses_group_create() {
    let cli = parse(&["group", "create", "--name", "QA", "--user", "1,2"]);
    match cli.command {
        Command::Group(redmine_cli::cli::groups::GroupCommand::Create(a)) => {
            assert_eq!(a.name, "QA");
            assert_eq!(a.users, vec![1, 2]);
        }
        _ => panic!("expected Group::Create"),
    }
}

#[test]
fn parses_group_add_user() {
    let cli = parse(&["group", "add-user", "10", "--user", "42"]);
    match cli.command {
        Command::Group(redmine_cli::cli::groups::GroupCommand::AddUser(a)) => {
            assert_eq!(a.group, 10);
            assert_eq!(a.user, 42);
        }
        _ => panic!("expected Group::AddUser"),
    }
}

#[test]
fn parses_my_account_update() {
    let cli = parse(&["my-account", "update", "--mail", "x@y"]);
    match cli.command {
        Command::MyAccount(redmine_cli::cli::my_account::MyAccountCommand::Update(a)) => {
            assert_eq!(a.mail.as_deref(), Some("x@y"));
        }
        _ => panic!("expected MyAccount::Update"),
    }
}

#[test]
fn parses_issue_watcher_add() {
    let cli = parse(&["issue", "10", "watcher", "add", "--user", "7"]);
    match cli.command {
        Command::Issue(a) => {
            assert_eq!(a.id, Some(10));
            match a.sub {
                Some(redmine_cli::cli::issues::IssueSub::Watcher(
                    redmine_cli::cli::issues::IssueWatcherSub::Add(w),
                )) => assert_eq!(w.user, 7),
                _ => panic!("expected Watcher::Add"),
            }
        }
        _ => panic!("expected Issue"),
    }
}

#[test]
fn parses_issue_note() {
    // --private 은 --private-notes 의 alias 로 유지되어야 한다 (호환).
    for flag in ["--private", "--private-notes"] {
        let cli = parse(&["issue", "10", "note", "--message", "hi", flag]);
        match cli.command {
            Command::Issue(a) => {
                assert_eq!(a.id, Some(10));
                match a.sub {
                    Some(redmine_cli::cli::issues::IssueSub::Note(n)) => {
                        assert_eq!(n.message, "hi");
                        assert!(n.private_notes, "flag {} should set private_notes", flag);
                    }
                    _ => panic!("expected Note"),
                }
            }
            _ => panic!("expected Issue"),
        }
    }
}

#[test]
fn parses_time_entry_create_with_id_only() {
    let cli = parse(&[
        "time-entry",
        "create",
        "--issue",
        "10",
        "--hours",
        "1.5",
        "--id-only",
    ]);
    match cli.command {
        Command::TimeEntry(redmine_cli::cli::time_entries::TimeEntryCommand::Create(a)) => {
            assert!(a.id_only);
        }
        _ => panic!("expected TimeEntry::Create"),
    }
}

#[test]
fn parses_issue_add_relation_with_id_only() {
    let cli = parse(&["issue", "10", "add-relation", "--to", "11", "--id-only"]);
    match cli.command {
        Command::Issue(a) => match a.sub {
            Some(redmine_cli::cli::issues::IssueSub::AddRelation(r)) => {
                assert_eq!(r.to, 11);
                assert!(r.id_only);
            }
            _ => panic!("expected AddRelation"),
        },
        _ => panic!("expected Issue"),
    }
}

#[test]
fn parses_attachment_upload_with_token_only() {
    let cli = parse(&[
        "attachment",
        "upload",
        "--issue",
        "10",
        "--file",
        "/tmp/x.bin",
        "--token-only",
    ]);
    match cli.command {
        Command::Attachment(redmine_cli::cli::attachments::AttachmentCommand::Upload(a)) => {
            assert_eq!(a.issue, 10);
            assert_eq!(a.file.as_os_str(), "/tmp/x.bin");
            assert!(a.token_only);
        }
        _ => panic!("expected Attachment::Upload"),
    }
}

#[test]
fn parses_version_create_with_id_only() {
    let cli = parse(&["version", "create", "demo", "--name", "v2", "--id-only"]);
    match cli.command {
        Command::Version(redmine_cli::cli::versions::VersionCommand::Create(a)) => {
            assert!(a.id_only);
        }
        _ => panic!("expected Version::Create"),
    }
}

#[test]
fn parses_news_create_with_id_only() {
    let cli = parse(&["news", "create", "demo", "--title", "hi", "--id-only"]);
    match cli.command {
        Command::News(redmine_cli::cli::news::NewsCommand::Create(a)) => {
            assert!(a.id_only);
        }
        _ => panic!("expected News::Create"),
    }
}

#[test]
fn parses_group_create_with_id_only() {
    let cli = parse(&["group", "create", "--name", "QA", "--id-only"]);
    match cli.command {
        Command::Group(redmine_cli::cli::groups::GroupCommand::Create(a)) => {
            assert!(a.id_only);
        }
        _ => panic!("expected Group::Create"),
    }
}

#[test]
fn parses_membership_add_with_id_only() {
    let cli = parse(&[
        "membership",
        "add",
        "demo",
        "--user",
        "11",
        "--role",
        "4",
        "--id-only",
    ]);
    match cli.command {
        Command::Membership(redmine_cli::cli::memberships::MembershipCommand::Add(a)) => {
            assert!(a.id_only);
        }
        _ => panic!("expected Membership::Add"),
    }
}
