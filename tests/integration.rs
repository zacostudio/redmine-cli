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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args(["issue", "42"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["id"], 42);
    assert_eq!(v["subject"], "answer");
}

/// config.yml 을 만들어 두는 헬퍼. 서버 하나는 mock 서버를, 하나는 쓰지 않는 URL 을 가리킨다.
fn write_config(path: &std::path::Path, company_url: &str) {
    std::fs::write(
        path,
        format!(
            "default_server: company\n\
             servers:\n\
             \x20 company:\n\
             \x20   url: {company_url}\n\
             \x20   api_token: secret\n\
             \x20   custom_fields:\n\
             \x20     state: 7\n\
             \x20 personal:\n\
             \x20   url: https://personal.invalid\n\
             \x20   api_token: ptok\n\
             \x20   custom_fields:\n\
             \x20     state: 3\n"
        ),
    )
    .unwrap();
}

#[test]
fn config_alias_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    write_config(&cfg_path, "https://company.invalid");
    let cfg = cfg_path.to_str().unwrap();

    // 기본 서버(company)의 alias 를 본다.
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "alias", "list"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["server"], "company");
    assert_eq!(v["aliases"]["state"], 7);

    // set — --server 로 지정한 서버에만 들어가야 한다.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config", cfg, "--server", "personal", "config", "alias", "set", "qa", "8",
        ])
        .assert()
        .success();

    // 보안: 토큰을 평문 보관하는 파일이므로 Unix 에서는 반드시 0600 이어야 한다.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(&cfg_path).unwrap();
        assert_eq!(
            meta.permissions().mode() & 0o777,
            0o600,
            "config.yml must be 0600 to protect the stored api_token"
        );
    }

    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config", cfg, "--server", "personal", "config", "alias", "list",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["aliases"]["qa"], 8);
    assert_eq!(v["aliases"]["state"], 3, "서버별 alias 는 섞이지 않는다");

    // company 쪽에는 qa 가 없어야 한다.
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "alias", "list"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(v["aliases"]["qa"].is_null());

    // remove
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config", cfg, "--server", "personal", "config", "alias", "remove", "qa",
        ])
        .assert()
        .success();
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config", cfg, "--server", "personal", "config", "alias", "list",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(v["aliases"]["qa"].is_null());
}

#[test]
fn config_server_list_hides_tokens_and_use_sets_default() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    write_config(&cfg_path, "https://company.invalid");
    let cfg = cfg_path.to_str().unwrap();

    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "list"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("secret") && !stdout.contains("ptok"),
        "server list 는 토큰을 출력하면 안 된다: {stdout}"
    );
    let v: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["default_server"], "company");
    let servers = v["servers"].as_array().unwrap();
    assert_eq!(servers.len(), 2);
    assert_eq!(servers[0]["name"], "company");
    assert_eq!(servers[0]["default"], true);

    // use — 기본 서버를 바꾼다.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "use", "personal"])
        .assert()
        .success();
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "list"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["default_server"], "personal");

    // 없는 이름은 에러.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "use", "nope"])
        .assert()
        .failure();
}

#[tokio::test(flavor = "current_thread")]
async fn server_flag_picks_the_configured_server() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/issues.json"))
        .and(header("X-Redmine-API-Key", "secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [], "total_count": 0
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    write_config(&cfg_path, &server.uri());
    let cfg = cfg_path.to_str().unwrap();

    // 기본 서버(company)로 호출 — mock 서버에 도달한다.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "issues"])
        .assert()
        .success();

    // --server company 로 명시해도 같다.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "--server", "company", "issues"])
        .assert()
        .success();

    // 없는 서버 이름은 사용 가능한 이름을 알려주며 실패한다.
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "--server", "nope", "issues"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("company") && stderr.contains("personal"),
        "{stderr}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn credential_flags_do_not_read_the_config_file() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/issues.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [], "total_count": 0
        })))
        .mount(&server)
        .await;

    // 파싱조차 안 되는 config.yml 이 있어도 flag 만으로 한 호출은 성공해야 한다.
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    std::fs::write(&cfg_path, "servers: [ this is not valid\n").unwrap();

    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg_path.to_str().unwrap()])
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret", "issues"])
        .assert()
        .success();
}

#[test]
fn config_server_add_creates_file_and_sets_first_as_default() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    let cfg = cfg_path.to_str().unwrap();

    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg,
            "--api-token",
            "ctok",
            "config",
            "server",
            "add",
            "company",
            "--url",
            "https://company.invalid",
        ])
        .assert()
        .success();

    assert!(cfg_path.exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(&cfg_path).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }

    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "list"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("ctok"),
        "토큰이 출력되면 안 된다: {stdout}"
    );
    let v: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(v["default_server"], "company", "첫 서버는 기본 서버가 된다");
    assert_eq!(v["servers"][0]["url"], "https://company.invalid");

    // 두 번째 서버는 기본값을 바꾸지 않는다.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg,
            "--api-token",
            "ptok",
            "config",
            "server",
            "add",
            "personal",
            "--url",
            "https://personal.invalid",
        ])
        .assert()
        .success();
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "list"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["default_server"], "company");
    assert_eq!(v["servers"].as_array().unwrap().len(), 2);
}

#[test]
fn config_server_add_rejects_duplicate_unless_forced() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    let cfg = cfg_path.to_str().unwrap();
    let add = |url: &str, force: bool| {
        let mut c = AssertCommand::cargo_bin("redmine").unwrap();
        c.args([
            "--config",
            cfg,
            "--api-token",
            "ctok",
            "config",
            "server",
            "add",
            "company",
            "--url",
            url,
        ]);
        if force {
            c.arg("--force");
        }
        c
    };

    add("https://one.invalid", false).assert().success();
    let out = add("https://two.invalid", false).assert().failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("already exists"), "{stderr}");

    add("https://two.invalid", true).assert().success();
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "list"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["servers"][0]["url"], "https://two.invalid");
}

#[test]
fn config_server_add_reads_token_from_stdin() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");

    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg_path.to_str().unwrap(),
            "config",
            "server",
            "add",
            "company",
            "--url",
            "https://company.invalid",
        ])
        .write_stdin("stok\n")
        .assert()
        .success();

    // trim 되지 않았다면 serde 가 따옴표로 감싸 escape 하므로 이 형태가 나오지 않는다.
    let yml = std::fs::read_to_string(&cfg_path).unwrap();
    assert!(
        yml.contains("api_token: stok\n"),
        "stdin 토큰이 그대로 저장돼야 한다: {yml}"
    );
}

#[test]
fn config_server_remove_clears_the_default() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    write_config(&cfg_path, "https://company.invalid");
    let cfg = cfg_path.to_str().unwrap();

    // 없는 서버는 에러.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "remove", "nope"])
        .assert()
        .failure();

    // 기본 서버(company)를 지우면 default_server 도 비워진다.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "remove", "company"])
        .assert()
        .success();

    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg, "config", "server", "list"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(v["default_server"].is_null());
    let servers = v["servers"].as_array().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0]["name"], "personal");
}

#[test]
fn config_server_add_rejects_unusable_input() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = tmp.path().join("config.yml");
    let cfg = cfg.to_str().unwrap();
    let add = |name: &str, url: &str| {
        let mut c = AssertCommand::cargo_bin("redmine").unwrap();
        c.args([
            "--config", cfg, "config", "server", "add", name, "--url", url,
        ]);
        c
    };

    // 여러 줄 토큰은 HTTP 헤더로 만들 수 없다. 저장 시점에 막는다.
    let out = add("a", "https://a.invalid")
        .write_stdin("tok\nextra\n")
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("token"), "{stderr}");

    // URL 이 아닌 문자열
    let out = add("a", "not a url").write_stdin("tok").assert().failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("url"), "{stderr}");

    // http/https 가 아닌 scheme
    add("a", "ftp://a.invalid")
        .write_stdin("tok")
        .assert()
        .failure();

    // 빈 이름
    add("", "https://a.invalid")
        .write_stdin("tok")
        .assert()
        .failure();

    assert!(
        !std::path::Path::new(cfg).exists(),
        "거부된 입력으로 파일이 만들어지면 안 된다"
    );
}

#[test]
fn config_server_add_force_keeps_the_existing_token() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    let cfg = cfg_path.to_str().unwrap();

    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg,
            "config",
            "server",
            "add",
            "a",
            "--url",
            "https://a.invalid",
        ])
        .write_stdin("tok")
        .assert()
        .success();

    // 토큰을 다시 주지 않아도 URL 만 갱신할 수 있어야 한다.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg,
            "config",
            "server",
            "add",
            "a",
            "--url",
            "https://b.invalid",
            "--force",
        ])
        .assert()
        .success();

    let yml = std::fs::read_to_string(&cfg_path).unwrap();
    assert!(yml.contains("https://b.invalid"), "{yml}");
    assert!(
        yml.contains("api_token: tok"),
        "기존 토큰이 유지돼야 한다: {yml}"
    );
}

#[test]
fn config_alias_set_rejects_names_that_can_never_resolve() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    write_config(&cfg_path, "https://company.invalid");
    let cfg = cfg_path.to_str().unwrap();
    let set = |name: &str| {
        let mut c = AssertCommand::cargo_bin("redmine").unwrap();
        c.args(["--config", cfg, "config", "alias", "set", name, "99"]);
        c
    };

    // 숫자 이름은 --custom-field 에서 항상 id 로 먼저 해석돼 도달할 수 없다.
    set("7").assert().failure();
    // '=' 가 들어가면 id=value 분해가 불가능하다.
    set("a=b").assert().failure();
    set("").assert().failure();
    set("ok").assert().success();
}

#[test]
fn config_server_list_reflects_the_actual_resolution() {
    let tmp = tempfile::tempdir().unwrap();

    // default_server 가 없어도 서버가 하나면 그 서버가 실제로 쓰인다.
    let single = tmp.path().join("single.yml");
    std::fs::write(
        &single,
        "servers:\n  only:\n    url: https://o.invalid\n    api_token: t\n",
    )
    .unwrap();
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            single.to_str().unwrap(),
            "config",
            "server",
            "list",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(
        v["servers"][0]["default"], true,
        "단일 서버 폴백이 list 에도 보여야 한다"
    );

    // 가리키는 대상이 없는 default_server 는 경고로 드러난다.
    let dangling = tmp.path().join("dangling.yml");
    std::fs::write(
        &dangling,
        "default_server: gone\nservers:\n  a:\n    url: https://a.invalid\n    api_token: t\n",
    )
    .unwrap();
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            dangling.to_str().unwrap(),
            "config",
            "server",
            "list",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(
        v["warning"].as_str().unwrap_or_default().contains("gone"),
        "dangling default 를 알려야 한다: {v}"
    );
}

#[test]
fn config_server_use_on_empty_config_points_at_the_file() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = tmp.path().join("config.yml");
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "config",
            "server",
            "use",
            "company",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("no Redmine server configured"),
        "빈 설정에서는 'available: ' 가 아니라 파일 경로를 알려야 한다: {stderr}"
    );
}

#[test]
fn config_rejects_misspelled_keys() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = tmp.path().join("config.yml");
    // api-token (하이픈) 은 조용히 무시되면 안 된다.
    std::fs::write(
        &cfg,
        "servers:\n  a:\n    url: https://a.invalid\n    api-token: abc\n",
    )
    .unwrap();
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "config",
            "server",
            "list",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("api-token"), "{stderr}");
}

#[test]
fn config_server_remove_reports_the_new_default() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    write_config(&cfg_path, "https://company.invalid");

    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg_path.to_str().unwrap(),
            "config",
            "server",
            "remove",
            "company",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(
        v["default_server"].is_null(),
        "기본 서버가 비워진 사실이 출력에 드러나야 한다: {v}"
    );

    // 저장 도중 임시 파일이 남으면 안 된다.
    let leftovers: Vec<_> = std::fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n != "config.yml")
        .collect();
    assert!(leftovers.is_empty(), "임시 파일 잔여물: {leftovers:?}");
}

#[tokio::test(flavor = "current_thread")]
async fn api_token_file_keeps_the_token_off_argv() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/issues.json"))
        .and(header("X-Redmine-API-Key", "filetok"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [], "total_count": 0
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let token_file = tmp.path().join("token");
    // 끝의 개행은 흔한 실수라 자동으로 잘라낸다.
    std::fs::write(&token_file, "filetok\n").unwrap();
    let cfg = tmp.path().join("config.yml");

    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg.to_str().unwrap()])
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token-file", token_file.to_str().unwrap(), "issues"])
        .assert()
        .success();

    // --api-token 과 동시에 주면 어느 쪽이 이겼는지 알 수 없으므로 거부한다.
    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg.to_str().unwrap()])
        .arg("--server-url")
        .arg(server.uri())
        .args([
            "--api-token",
            "x",
            "--api-token-file",
            token_file.to_str().unwrap(),
            "issues",
        ])
        .assert()
        .failure();

    // 여러 줄이 든 파일은 헤더로 만들 수 없다.
    let bad = tmp.path().join("bad");
    std::fs::write(&bad, "tok\nextra\n").unwrap();
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg.to_str().unwrap()])
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token-file", bad.to_str().unwrap(), "issues"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("token"), "{stderr}");

    // 없는 파일은 경로를 알려준다.
    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", cfg.to_str().unwrap()])
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token-file", "/nonexistent/token", "issues"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("/nonexistent/token"), "{stderr}");
}

#[test]
fn config_server_add_accepts_a_token_file() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
    let token_file = tmp.path().join("token");
    std::fs::write(&token_file, "filetok\n").unwrap();

    AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args([
            "--config",
            cfg_path.to_str().unwrap(),
            "--api-token-file",
            token_file.to_str().unwrap(),
            "config",
            "server",
            "add",
            "a",
            "--url",
            "https://a.invalid",
        ])
        .assert()
        .success();
    let yml = std::fs::read_to_string(&cfg_path).unwrap();
    assert!(yml.contains("api_token: filetok\n"), "{yml}");
}

#[test]
fn leftover_config_toml_is_named_in_the_error() {
    let tmp = tempfile::tempdir().unwrap();
    let toml_path = tmp.path().join("config.toml");
    let yml_path = tmp.path().join("config.yml");
    std::fs::write(&toml_path, "server_url = \"https://old.invalid\"\n").unwrap();

    let out = AssertCommand::cargo_bin("redmine")
        .unwrap()
        .args(["--config", yml_path.to_str().unwrap(), "issues"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("config.toml"),
        "옆에 있는 옛 파일을 알려야 한다: {stderr}"
    );
    assert!(
        stderr.contains("config server add"),
        "옮기는 방법까지 알려야 한다: {stderr}"
    );
    // 읽지 않고 존재만 본다. 파일은 그대로 남는다.
    assert!(toml_path.exists());
    assert!(!yml_path.exists());
}

#[test]
fn empty_config_reports_no_server_configured() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.yml");
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
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("no Redmine server"), "{stderr}");
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args(["membership", "add", "demo", "--user", "11", "--role", "4,5"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args([
            "news",
            "create",
            "demo",
            "--title",
            "Hello",
            "--description",
            "world",
        ])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args([
            "wiki",
            "update",
            "demo",
            "Roadmap",
            "--text",
            "hello",
            "--comments",
            "init",
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
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args(["wiki", "show", "demo", "Missing"])
        .assert()
        .failure();
}

#[tokio::test(flavor = "current_thread")]
async fn group_list_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/groups.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "groups": [
                {"id": 10, "name": "QA"},
                {"id": 11, "name": "Devs"}
            ]
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args(["group", "list"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["groups"][0]["id"], 10);
    assert_eq!(v["groups"][1]["name"], "Devs");
}

#[tokio::test(flavor = "current_thread")]
async fn group_create_posts_and_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/groups.json"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "group": {"id": 99, "name": "Reviewers", "users": [{"id": 1, "name": "alice"}]}
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args(["group", "create", "--name", "Reviewers", "--user", "1"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["id"], 99);
    assert_eq!(v["users"][0]["name"], "alice");
}

#[tokio::test(flavor = "current_thread")]
async fn my_account_show_returns_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/my/account.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "user": {
                "id": 7,
                "login": "alice",
                "firstname": "Alice",
                "lastname": "Park",
                "mail": "a@x",
                "admin": false,
                "created_on": "2024-01-01T00:00:00Z",
                "last_login_on": "2026-05-16T00:00:00Z"
            }
        })))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args(["my-account", "show"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["id"], 7);
    assert_eq!(v["login"], "alice");
    assert_eq!(v["admin"], false);
}

#[tokio::test(flavor = "current_thread")]
async fn issue_watcher_add_posts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/issues/42/watchers.json"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args(["issue", "42", "watcher", "add", "--user", "7"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["ok"], true);
}

#[tokio::test(flavor = "current_thread")]
async fn issue_note_puts_journal() {
    let server = MockServer::start().await;
    // issue note 는 PUT 만 수행하고 즉시 ok 를 출력한다 (이전에는 PUT 후 GET 까지 했었음).
    Mock::given(method("PUT"))
        .and(path("/issues/42.json"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args([
            "issue",
            "42",
            "note",
            "--message",
            "looks good",
            "--private",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["ok"], true);
}

#[tokio::test(flavor = "current_thread")]
async fn attachment_download_rejects_foreign_host() {
    // 서버 응답의 content_url 이 외부 호스트를 가리키는 경우 다운로드를 거부해야 한다.
    // 거부하지 않으면 X-Redmine-API-Key 가 외부 호스트로 따라간다.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/attachments/1.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "attachment": {
                "id": 1,
                "filename": "evil.bin",
                "content_url": "http://attacker.example.invalid/leak"
            }
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("out.bin");
    let assert = Command::cargo_bin("redmine")
        .unwrap()
        .arg("--server-url")
        .arg(server.uri())
        .args(["--api-token", "secret"])
        .args([
            "attachment",
            "download",
            "1",
            "--output",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("does not match server"),
        "stderr should mention host mismatch, got: {stderr}"
    );
    assert!(
        !out_path.exists(),
        "output file must not be created when host check fails"
    );
}
