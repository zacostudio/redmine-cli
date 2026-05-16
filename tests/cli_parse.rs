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
        "--project", "demo",
        "--status", "1",
        "--query", "bug",
        "--custom-field", "7=Dev",
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
    let cli = parse(&[
        "issue", "create",
        "--project", "demo",
        "--subject", "hi",
    ]);
    match cli.command {
        Command::Issue(a) => {
            assert!(a.id.is_none());
            assert!(matches!(a.sub, Some(redmine_cli::cli::issues::IssueSub::Create(_))));
        }
        _ => panic!("expected Issue"),
    }
}

#[test]
fn parses_time_entry_create() {
    let cli = parse(&[
        "time-entry", "create",
        "--issue", "10",
        "--hours", "1.5",
    ]);
    match cli.command {
        Command::TimeEntry(redmine_cli::cli::time_entries::TimeEntryCommand::Create(a)) => {
            assert_eq!(a.issue, 10);
            assert!((a.hours - 1.5).abs() < f64::EPSILON);
        }
        _ => panic!("expected TimeEntry::Create"),
    }
}
