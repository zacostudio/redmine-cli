// 가짜 Redmine 서버를 띄우고 실제 바이너리를 호출해 검증.
use assert_cmd::Command;
use serde_json::Value;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use assert_cmd::Command as AssertCommand;

#[tokio::test(flavor = "current_thread")]
async fn issues_search_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/issues.json"))
        .and(query_param("limit", "1"))
        .and(header("X-Redmine-API-Key", "secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [{
                "id": 7,
                "project": {"id": 1, "name": "demo"},
                "subject": "hello",
                "status": {"id": 1, "name": "New"}
            }],
            "total_count": 1
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["issues", "--limit", "1"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["total_count"], 1);
    assert_eq!(v["issues"][0]["id"], 7);
    assert_eq!(v["issues"][0]["subject"], "hello");
}

#[tokio::test(flavor = "current_thread")]
async fn issue_get_by_id_returns_full() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/issues/42.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issue": {
                "id": 42,
                "project": {"id": 1, "name": "demo"},
                "subject": "answer"
            }
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["issue", "42"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["id"], 42);
    assert_eq!(v["subject"], "answer");
}

#[test]
fn config_alias_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.toml");

    // initial list — empty
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg_path.to_str().unwrap(),
            "config",
            "alias",
            "list",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(v["aliases"].as_object().unwrap().is_empty());

    // set
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg_path.to_str().unwrap(),
            "config",
            "alias",
            "set",
            "state",
            "7",
        ])
        .assert()
        .success();

    // list — now has state=7
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg_path.to_str().unwrap(),
            "config",
            "alias",
            "list",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["aliases"]["state"], 7);

    // remove
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg_path.to_str().unwrap(),
            "config",
            "alias",
            "remove",
            "state",
        ])
        .assert()
        .success();

    // list — empty again
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg_path.to_str().unwrap(),
            "config",
            "alias",
            "list",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(v["aliases"].as_object().unwrap().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn roles_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/roles.json"))
        .and(header("X-Redmine-API-Key", "secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "roles": [
                {"id": 3, "name": "Manager"},
                {"id": 4, "name": "Developer"}
            ]
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["roles"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v[0]["id"], 3);
    assert_eq!(v[0]["name"], "Manager");
    assert_eq!(v[1]["id"], 4);
}

#[tokio::test(flavor = "current_thread")]
async fn document_categories_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/enumerations/document_categories.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "document_categories": [
                {"id": 1, "name": "Uncategorized", "is_default": true},
                {"id": 2, "name": "Technical"}
            ]
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["document-categories"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v[0]["id"], 1);
    assert_eq!(v[0]["name"], "Uncategorized");
    assert_eq!(v[0]["is_default"], true);
}

#[tokio::test(flavor = "current_thread")]
async fn custom_fields_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/custom_fields.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "custom_fields": [
                {
                    "id": 7,
                    "name": "Severity",
                    "customized_type": "issue",
                    "field_format": "list",
                    "is_required": true,
                    "is_filter": true,
                    "multiple": false,
                    "default_value": "Low",
                    "visible": true,
                    "possible_values": [{"value": "Low"}, {"value": "High"}]
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["custom-fields"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v[0]["id"], 7);
    assert_eq!(v[0]["name"], "Severity");
    assert_eq!(v[0]["customized_type"], "issue");
    assert_eq!(v[0]["field_format"], "list");
}

#[tokio::test(flavor = "current_thread")]
async fn search_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("q", "bug"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {"id": 9, "title": "bug: x", "type": "issue", "url": "http://x/9", "datetime": "2026-05-16T00:00:00Z"}
            ],
            "total_count": 1,
            "offset": 0,
            "limit": 10
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["search", "bug", "--limit", "10"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["total_count"], 1);
    assert_eq!(v["results"][0]["id"], 9);
    assert_eq!(v["results"][0]["type"], "issue");
}

#[tokio::test(flavor = "current_thread")]
async fn version_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/projects/demo/versions.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "versions": [
                {"id": 11, "project": {"id": 1, "name": "demo"}, "name": "v1.0", "status": "open"}
            ],
            "total_count": 1
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["version", "list", "demo"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["total_count"], 1);
    assert_eq!(v["versions"][0]["id"], 11);
    assert_eq!(v["versions"][0]["name"], "v1.0");
}

#[tokio::test(flavor = "current_thread")]
async fn version_create_posts_and_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/projects/demo/versions.json"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "version": {"id": 22, "project": {"id": 1, "name": "demo"}, "name": "v2.0", "status": "open"}
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["version", "create", "demo", "--name", "v2.0"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["id"], 22);
    assert_eq!(v["name"], "v2.0");
}

#[tokio::test(flavor = "current_thread")]
async fn membership_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/projects/demo/memberships.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "memberships": [
                {
                    "id": 5,
                    "project": {"id": 1, "name": "demo"},
                    "user": {"id": 10, "name": "alice"},
                    "roles": [{"id": 4, "name": "Developer"}]
                }
            ],
            "total_count": 1
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["membership", "list", "demo"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["memberships"][0]["id"], 5);
    assert_eq!(v["memberships"][0]["user"]["name"], "alice");
    assert_eq!(v["memberships"][0]["roles"][0]["name"], "Developer");
}

#[tokio::test(flavor = "current_thread")]
async fn membership_add_posts_and_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/projects/demo/memberships.json"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "membership": {
                "id": 9,
                "project": {"id": 1, "name": "demo"},
                "user": {"id": 11, "name": "bob"},
                "roles": [{"id": 4, "name": "Developer"}, {"id": 5, "name": "Reporter"}]
            }
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args([
            "membership", "add", "demo", "--user", "11", "--role", "4,5",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["id"], 9);
    assert_eq!(v["roles"].as_array().unwrap().len(), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn news_list_global_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/news.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "news": [
                {"id": 3, "project": {"id": 1, "name": "demo"}, "title": "Release", "summary": "v1"}
            ],
            "total_count": 1
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["news", "list"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["news"][0]["title"], "Release");
}

#[tokio::test(flavor = "current_thread")]
async fn news_create_posts_and_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/projects/demo/news.json"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "news": {"id": 8, "project": {"id": 1, "name": "demo"}, "title": "Hello", "description": "world"}
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["news", "create", "demo", "--title", "Hello", "--description", "world"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["id"], 8);
    assert_eq!(v["description"], "world");
}

#[tokio::test(flavor = "current_thread")]
async fn file_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/projects/demo/files.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "files": [
                {
                    "id": 17,
                    "filename": "spec.pdf",
                    "filesize": 1024,
                    "content_type": "application/pdf"
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["file", "list", "demo"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["files"][0]["id"], 17);
    assert_eq!(v["files"][0]["filename"], "spec.pdf");
}

#[tokio::test(flavor = "current_thread")]
async fn query_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/queries.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "queries": [
                {"id": 1, "name": "Open bugs", "is_public": true},
                {"id": 2, "name": "My tasks", "is_public": false, "project_id": 5}
            ],
            "total_count": 2
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["query"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["queries"][0]["name"], "Open bugs");
    assert_eq!(v["queries"][1]["project_id"], 5);
}

#[tokio::test(flavor = "current_thread")]
async fn wiki_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/projects/demo/wiki/index.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "wiki_pages": [
                {"title": "Wiki", "version": 1, "created_on": "2026-01-01T00:00:00Z"},
                {"title": "Roadmap", "parent": {"title": "Wiki"}, "version": 4}
            ]
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["wiki", "list", "demo"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["wiki_pages"][0]["title"], "Wiki");
    assert_eq!(v["wiki_pages"][1]["parent"], "Wiki");
}

#[tokio::test(flavor = "current_thread")]
async fn wiki_show_returns_full_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/projects/demo/wiki/Roadmap.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "wiki_page": {
                "title": "Roadmap",
                "text": "# plan",
                "version": 4,
                "author": {"id": 1, "name": "admin"},
                "comments": "fix typo",
                "created_on": "2026-01-01T00:00:00Z",
                "updated_on": "2026-05-16T00:00:00Z"
            }
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["wiki", "show", "demo", "Roadmap"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["title"], "Roadmap");
    assert_eq!(v["text"], "# plan");
    assert_eq!(v["version"], 4);
    assert_eq!(v["author"], "admin");
}

#[tokio::test(flavor = "current_thread")]
async fn wiki_put_creates_or_updates() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/projects/demo/wiki/Roadmap.json"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args([
            "wiki", "update", "demo", "Roadmap", "--text", "hello", "--comments", "init",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["ok"], true);
    assert_eq!(v["title"], "Roadmap");
}

#[tokio::test(flavor = "current_thread")]
async fn wiki_show_404_surfaces_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/projects/demo/wiki/Missing.json"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    Command::cargo_bin("redmine")
        .unwrap()
        .env("REDMINE_URL", server.uri())
        .env("REDMINE_API_TOKEN", "secret")
        .args(["wiki", "show", "demo", "Missing"])
        .assert()
        .failure();
}
