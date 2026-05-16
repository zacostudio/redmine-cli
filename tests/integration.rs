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
