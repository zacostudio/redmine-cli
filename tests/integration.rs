// 가짜 Redmine 서버를 띄우고 실제 바이너리를 호출해 검증.
use assert_cmd::Command;
use serde_json::Value;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
