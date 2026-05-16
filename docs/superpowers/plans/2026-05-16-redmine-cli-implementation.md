# Redmine CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** tome 의 Redmine 모듈을 독립 Rust CLI 바이너리(`redmine`)로 이식하여 Homebrew tap 으로 배포한다.

**Architecture:** 단일 `redmine` 바이너리. `clap` derive 로 인자 파싱, `reqwest` blocking 으로 Redmine REST API 호출, JSON stdout 출력. 설정은 CLI 플래그 > 환경변수 > `~/.config/redmine-cli/config.toml` 우선순위로 머지.

**Tech Stack:** Rust stable (1.83+), clap 4.5, reqwest 0.12 (blocking, rustls-tls), serde 1, serde_json 1, toml 0.9, directories 6, anyhow 1, urlencoding 2, libc 0.2. 테스트: wiremock 0.6, assert_cmd 2.

**참고 원본 경로.**
- `/Volumes/Projects/zacostudio/apps/tome/src-tauri/src/services/redmine/client.rs`
- `/Volumes/Projects/zacostudio/apps/tome/src-tauri/src/services/redmine/types.rs`
- `/Volumes/Projects/zacostudio/apps/tome/src-tauri/src/cli/redmine.rs`
- `/Volumes/Projects/zacostudio/apps/tome/src-tauri/src/cli/output.rs`

**프로젝트 루트:** `/Volumes/Projects/zacostudio/cli/issues`

---

## Phase 1 — 프로젝트 스캐폴드

### Task 1: Cargo.toml / rust-toolchain.toml / .gitignore

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`

- [ ] **Step 1: `Cargo.toml` 작성**

```toml
[package]
name = "redmine-cli"
version = "0.1.0"
edition = "2021"
rust-version = "1.83"
description = "Standalone CLI for Redmine"
license = "MIT"
repository = "https://github.com/zacostudio/redmine-cli"
readme = "README.md"

[[bin]]
name = "redmine"
path = "src/main.rs"

[lib]
name = "redmine_cli"
path = "src/lib.rs"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.9"
directories = "6"
anyhow = "1"
urlencoding = "2"
libc = "0.2"

[dev-dependencies]
assert_cmd = "2"
wiremock = "0.6"
predicates = "3"
tokio = { version = "1", features = ["rt", "macros"] }

[profile.release]
opt-level = 3
```

- [ ] **Step 2: `rust-toolchain.toml` 작성**

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

- [ ] **Step 3: `.gitignore` 작성**

```gitignore
/target
**/*.rs.bk
Cargo.lock.bak
.env
.env.local
.idea/
.vscode/
.DS_Store
```

- [ ] **Step 4: 의존성 해석 확인 (네트워크 필요)**

Run: `cargo fetch`
Expected: 모든 crate 다운로드 완료, 에러 없음.

- [ ] **Step 5: 커밋**

```bash
git add Cargo.toml rust-toolchain.toml .gitignore
git commit -m "chore: bootstrap Cargo project for redmine CLI"
```

---

### Task 2: 최소 main.rs / lib.rs 로 빌드 통과

**Files:**
- Create: `src/main.rs`
- Create: `src/lib.rs`

- [ ] **Step 1: `src/lib.rs` 작성**

```rust
// Redmine CLI 의 공개 모듈 진입점.
pub mod client;
pub mod cli;
pub mod config;
pub mod output;
pub mod types;
```

이 시점엔 각 모듈이 아직 없으니 컴파일이 실패한다. 다음 step 에서 stub 만 채워 통과시킨다.

- [ ] **Step 2: 빈 모듈 stub 5개 작성**

각각 다음 한 줄짜리 파일을 만든다.

`src/client.rs`:
```rust
// Redmine REST API HTTP 클라이언트.
```

`src/cli.rs`:
```rust
// CLI 서브커맨드 디스패치 진입점.
```

`src/config.rs`:
```rust
// flag > env > toml 우선순위로 설정을 머지한다.
```

`src/output.rs`:
```rust
// JSON 출력 및 에러 종료 헬퍼.
```

`src/types.rs`:
```rust
// Redmine API 응답 타입 정의.
```

주의: `src/cli/mod.rs` 가 아닌 `src/cli.rs` 로 잠시 두고, Phase 3 에서 디렉터리로 승격한다.

- [ ] **Step 3: `src/main.rs` 작성**

```rust
// redmine 바이너리 진입점. clap 파싱 후 cli::dispatch 호출.
fn main() {
    println!("{{\"hello\": \"redmine\"}}");
}
```

- [ ] **Step 4: 빌드**

Run: `cargo build`
Expected: 경고는 가능하지만 에러 없음, `target/debug/redmine` 생성.

- [ ] **Step 5: 실행 확인**

Run: `./target/debug/redmine`
Expected stdout: `{"hello": "redmine"}`

- [ ] **Step 6: 커밋**

```bash
git add src/
git commit -m "feat: scaffold binary and library entrypoints"
```

---

### Task 3: README 초기본

**Files:**
- Create: `README.md`

- [ ] **Step 1: `README.md` 작성**

````markdown
# redmine-cli

Standalone CLI for Redmine. Ported from the [tome](https://github.com/zacostudio/tome) Tauri app.

## Install

```bash
brew tap zacostudio/redmine
brew install redmine
```

Or from source:

```bash
cargo install --git https://github.com/zacostudio/redmine-cli
```

## Configure

Set environment variables:

```bash
export REDMINE_URL=https://redmine.example.com
export REDMINE_API_TOKEN=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

Or create `~/.config/redmine-cli/config.toml`:

```toml
server_url = "https://redmine.example.com"
api_token  = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

[custom_fields]
state = 7
qa    = 8
```

CLI flags `--server-url` and `--api-token` override env and file.

## Usage

```bash
redmine projects
redmine issues --project myproj --status 1
redmine issue 1234
redmine issue create --project myproj --subject "..." --description "..."
redmine time-entry create --issue 1234 --hours 2.5 --comment "..."
```

See `redmine --help`.

## License

MIT.
````

- [ ] **Step 2: 커밋**

```bash
git add README.md
git commit -m "docs: add initial README"
```

---

## Phase 2 — 공통 유틸리티

### Task 4: `output` 모듈 (JSON / 에러)

**Files:**
- Modify: `src/output.rs`
- Test: `src/output.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: 실패 테스트 작성**

`src/output.rs` 를 다음으로 교체.

```rust
// JSON 출력 및 에러 종료 헬퍼.
use serde_json::Value;

/// JSON 을 stdout 으로 한 줄 출력한다. broken-pipe 는 무시한다.
pub fn print_json(value: Value) {
    use std::io::Write;
    let _ = writeln!(std::io::stdout(), "{value}");
}

/// 에러 JSON 을 stderr 로 출력하고 1 로 종료한다.
pub fn print_error(message: &str) -> ! {
    let obj = serde_json::json!({ "error": message });
    eprintln!("{obj}");
    // FFI 없음. stdout flush 안전을 위해 _exit 사용.
    unsafe { libc::_exit(1) }
}

/// stdin 전체를 String 으로 읽는다.
pub fn read_stdin() -> Result<String, String> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("failed to read stdin: {e}"))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_json_serializes_value() {
        // 실제 stdout 캡처는 통합 테스트에서. 여기서는 직렬화만 검증.
        let v = serde_json::json!({"a": 1});
        assert_eq!(v.to_string(), r#"{"a":1}"#);
        // 실행해도 패닉 없어야 한다.
        print_json(v);
    }
}
```

- [ ] **Step 2: 테스트 실행**

Run: `cargo test --lib output::tests`
Expected: PASS.

- [ ] **Step 3: 커밋**

```bash
git add src/output.rs
git commit -m "feat(output): port JSON/error helpers from tome"
```

---

### Task 5: `types` 모듈 (Redmine API 응답 타입)

**Files:**
- Modify: `src/types.rs`

- [ ] **Step 1: tome 의 `types.rs` 를 이식**

원본: `/Volumes/Projects/zacostudio/apps/tome/src-tauri/src/services/redmine/types.rs` 전체를 다음과 같이 그대로 복사. 첫 줄에 한국어 헤더만 추가한다.

```rust
// Redmine API 응답 JSON 을 역직렬화하기 위한 데이터 타입 모음.
use serde::{Deserialize, Serialize};

// ── Shared field types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdName {
    pub id: u64,
    pub name: String,
}

// ── Issues ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineIssue {
    pub id: u64,
    pub project: IdName,
    pub tracker: Option<IdName>,
    pub status: Option<IdName>,
    pub priority: Option<IdName>,
    pub author: Option<IdName>,
    pub assigned_to: Option<IdName>,
    pub category: Option<IdName>,
    pub fixed_version: Option<IdName>,
    pub parent: Option<IssueParent>,
    pub subject: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub done_ratio: Option<u32>,
    pub estimated_hours: Option<f64>,
    pub spent_hours: Option<f64>,
    pub created_on: Option<String>,
    pub updated_on: Option<String>,
    pub closed_on: Option<String>,
    pub journals: Option<Vec<Journal>>,
    pub attachments: Option<Vec<Attachment>>,
    pub children: Option<Vec<ChildIssue>>,
    pub relations: Option<Vec<Relation>>,
    pub custom_fields: Option<Vec<CustomField>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueParent {
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub id: u64,
    pub user: Option<IdName>,
    pub notes: Option<String>,
    pub created_on: Option<String>,
    pub private_notes: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: u64,
    pub filename: String,
    pub filesize: Option<u64>,
    pub content_url: Option<String>,
    pub created_on: Option<String>,
    pub author: Option<IdName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildIssue {
    pub id: u64,
    pub tracker: Option<IdName>,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: u64,
    pub issue_id: u64,
    pub issue_to_id: u64,
    pub relation_type: String,
}

// ── Projects ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineProject {
    pub id: u64,
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub status: Option<u32>,
    pub created_on: Option<String>,
    pub updated_on: Option<String>,
}

// ── Categories ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineCategory {
    pub id: u64,
    pub name: String,
    pub project: Option<IdName>,
    pub assigned_to: Option<IdName>,
}

// ── Users ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineUser {
    pub id: u64,
    pub login: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub mail: Option<String>,
}

// ── Time entries ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineTimeEntry {
    pub id: u64,
    pub project: Option<IdName>,
    pub issue: Option<IssueParent>,
    pub user: Option<IdName>,
    pub activity: Option<IdName>,
    pub hours: f64,
    pub comments: Option<String>,
    pub spent_on: Option<String>,
    pub created_on: Option<String>,
}

// ── Activities ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineActivity {
    pub id: u64,
    pub name: String,
    pub is_default: Option<bool>,
}

// ── API response wrappers ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct IssuesResponse {
    pub issues: Vec<RedmineIssue>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct IssueResponse {
    pub issue: RedmineIssue,
}

#[derive(Debug, Deserialize)]
pub struct ProjectsResponse {
    pub projects: Vec<RedmineProject>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CategoriesResponse {
    pub issue_categories: Vec<RedmineCategory>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct UsersResponse {
    pub users: Vec<RedmineUser>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TimeEntryResponse {
    pub time_entry: RedmineTimeEntry,
}

#[derive(Debug, Deserialize)]
pub struct ActivitiesResponse {
    pub time_entry_activities: Vec<RedmineActivity>,
}

#[derive(Debug, Deserialize)]
pub struct TimeEntriesResponse {
    pub time_entries: Vec<RedmineTimeEntry>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    pub upload: UploadToken,
}

#[derive(Debug, Deserialize)]
pub struct UploadToken {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusesResponse {
    pub issue_statuses: Vec<RedmineStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineStatus {
    pub id: u64,
    pub name: String,
    pub is_closed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TrackersResponse {
    pub trackers: Vec<IdName>,
}

#[derive(Debug, Deserialize)]
pub struct PrioritiesResponse {
    pub issue_priorities: Vec<RedminePriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedminePriority {
    pub id: u64,
    pub name: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RelationResponse {
    pub relation: Relation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub id: u64,
    pub name: Option<String>,
    pub value: Option<String>,
}
```

- [ ] **Step 2: 빌드 확인**

Run: `cargo build`
Expected: 에러 없음, dead_code 경고는 있을 수 있음.

- [ ] **Step 3: 커밋**

```bash
git add src/types.rs
git commit -m "feat(types): port Redmine API types from tome"
```

---

### Task 6: `client` 모듈 (HTTP)

**Files:**
- Modify: `src/client.rs`

- [ ] **Step 1: `RedmineClient` 이식**

`src/client.rs` 를 다음으로 교체. 원본 `/Volumes/Projects/zacostudio/apps/tome/src-tauri/src/services/redmine/client.rs` 와 동일하되 (1) `use super::types::*;` 가 `use crate::types::*;` 로 바뀌고 (2) 첫 줄 한국어 헤더 추가.

```rust
// Redmine REST API 호출용 reqwest blocking 클라이언트.
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::types::*;

pub struct RedmineClient {
    client: Client,
    base_url: String,
}

impl RedmineClient {
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Redmine-API-Key",
            HeaderValue::from_str(api_key).map_err(|e| format!("invalid API key: {e}"))?,
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("failed to create HTTP client: {e}"))?;

        let base_url = base_url.trim_end_matches('/').to_string();
        Ok(Self { client, base_url })
    }

    pub fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .query(params)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(format!("HTTP {} — {}", status.as_u16(), body));
        }
        resp.json::<T>()
            .map_err(|e| format!("failed to parse response: {e}"))
    }

    fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().unwrap_or_default();
            return Err(format!("HTTP {} — {}", status.as_u16(), body_text));
        }
        resp.json::<T>()
            .map_err(|e| format!("failed to parse response: {e}"))
    }

    fn put_no_content(&self, path: &str, body: &serde_json::Value) -> Result<(), String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .put(&url)
            .json(body)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().unwrap_or_default();
            return Err(format!("HTTP {} — {}", status.as_u16(), body_text));
        }
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .delete(&url)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(format!("HTTP {} — {}", status.as_u16(), body));
        }
        Ok(())
    }

    // ── API methods ─────────────────────────────────────────────────

    pub fn search_issues(&self, params: &[(&str, String)]) -> Result<IssuesResponse, String> {
        self.get("/issues.json", params)
    }

    pub fn get_issue(&self, id: u64) -> Result<IssueResponse, String> {
        let path = format!("/issues/{}.json", id);
        self.get(
            &path,
            &[("include", "journals,attachments,children,relations".to_string())],
        )
    }

    pub fn create_issue(&self, payload: serde_json::Value) -> Result<IssueResponse, String> {
        self.post("/issues.json", &serde_json::json!({ "issue": payload }))
    }

    pub fn update_issue(
        &self,
        id: u64,
        payload: serde_json::Value,
    ) -> Result<IssueResponse, String> {
        let body = serde_json::json!({ "issue": payload });
        self.put_no_content(&format!("/issues/{}.json", id), &body)?;
        self.get_issue(id)
    }

    pub fn list_projects(&self, limit: u64, offset: u64) -> Result<ProjectsResponse, String> {
        self.get(
            "/projects.json",
            &[("limit", limit.to_string()), ("offset", offset.to_string())],
        )
    }

    pub fn list_categories(&self, project_id: &str) -> Result<CategoriesResponse, String> {
        let path = format!("/projects/{}/issue_categories.json", project_id);
        self.get(&path, &[])
    }

    pub fn create_time_entry(
        &self,
        payload: serde_json::Value,
    ) -> Result<TimeEntryResponse, String> {
        self.post(
            "/time_entries.json",
            &serde_json::json!({ "time_entry": payload }),
        )
    }

    pub fn search_users(&self, name: &str, limit: u64) -> Result<UsersResponse, String> {
        self.get(
            "/users.json",
            &[("name", name.to_string()), ("limit", limit.to_string())],
        )
    }

    pub fn list_activities(&self) -> Result<ActivitiesResponse, String> {
        self.get("/enumerations/time_entry_activities.json", &[])
    }

    pub fn delete_issue(&self, id: u64) -> Result<(), String> {
        self.delete(&format!("/issues/{}.json", id))
    }

    pub fn delete_attachment(&self, id: u64) -> Result<(), String> {
        self.delete(&format!("/attachments/{}.json", id))
    }

    pub fn upload_file(&self, file_path: &std::path::Path) -> Result<UploadResponse, String> {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload");
        let data = std::fs::read(file_path).map_err(|e| format!("failed to read file: {e}"))?;
        let url = format!(
            "{}/uploads.json?filename={}",
            self.base_url,
            urlencoding::encode(filename)
        );
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(data)
            .send()
            .map_err(|e| format!("upload failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(format!("HTTP {} — {}", status.as_u16(), body));
        }
        resp.json::<UploadResponse>()
            .map_err(|e| format!("failed to parse upload response: {e}"))
    }

    pub fn attach_to_issue(
        &self,
        issue_id: u64,
        token: &str,
        filename: &str,
        content_type: &str,
        description: Option<&str>,
    ) -> Result<(), String> {
        let mut upload = serde_json::json!({
            "token": token,
            "filename": filename,
            "content_type": content_type,
        });
        if let Some(desc) = description {
            upload["description"] = serde_json::json!(desc);
        }
        let body = serde_json::json!({ "issue": { "uploads": [upload] } });
        self.put_no_content(&format!("/issues/{}.json", issue_id), &body)
    }

    pub fn download_attachment(
        &self,
        url: &str,
        output_path: &std::path::Path,
    ) -> Result<(), String> {
        let resp = self
            .client
            .get(url)
            .send()
            .map_err(|e| format!("download failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("HTTP {}", status.as_u16()));
        }
        let bytes = resp.bytes().map_err(|e| format!("read failed: {e}"))?;
        std::fs::write(output_path, &bytes).map_err(|e| format!("write failed: {e}"))?;
        Ok(())
    }

    pub fn list_time_entries(
        &self,
        params: &[(&str, String)],
    ) -> Result<TimeEntriesResponse, String> {
        self.get("/time_entries.json", params)
    }

    pub fn update_time_entry(&self, id: u64, payload: serde_json::Value) -> Result<(), String> {
        let body = serde_json::json!({ "time_entry": payload });
        self.put_no_content(&format!("/time_entries/{}.json", id), &body)
    }

    pub fn delete_time_entry(&self, id: u64) -> Result<(), String> {
        self.delete(&format!("/time_entries/{}.json", id))
    }

    pub fn create_relation(
        &self,
        issue_id: u64,
        target_id: u64,
        relation_type: &str,
    ) -> Result<RelationResponse, String> {
        let body = serde_json::json!({
            "relation": {
                "issue_to_id": target_id,
                "relation_type": relation_type,
            }
        });
        let path = format!("/issues/{}/relations.json", issue_id);
        self.post(&path, &body)
    }

    pub fn delete_relation(&self, id: u64) -> Result<(), String> {
        self.delete(&format!("/relations/{}.json", id))
    }

    pub fn list_statuses(&self) -> Result<StatusesResponse, String> {
        self.get("/issue_statuses.json", &[])
    }

    pub fn list_trackers(&self) -> Result<TrackersResponse, String> {
        self.get("/trackers.json", &[])
    }

    pub fn list_priorities(&self) -> Result<PrioritiesResponse, String> {
        self.get("/enumerations/issue_priorities.json", &[])
    }
}
```

- [ ] **Step 2: 빌드 확인**

Run: `cargo build`
Expected: 에러 없음.

- [ ] **Step 3: 커밋**

```bash
git add src/client.rs
git commit -m "feat(client): port RedmineClient from tome"
```

---

### Task 7: `config` 모듈

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: 실패 테스트 먼저**

`src/config.rs` 를 다음으로 교체.

```rust
// flag > env > toml 우선순위로 Redmine 설정을 머지한다.
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
struct FileConfig {
    server_url: Option<String>,
    api_token: Option<String>,
    #[serde(default)]
    custom_fields: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server_url: String,
    pub api_token: String,
    pub cf_aliases: HashMap<String, u64>,
}

pub struct CliOverrides {
    pub server_url: Option<String>,
    pub api_token: Option<String>,
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing server URL (set --server-url, REDMINE_URL, or server_url in config.toml)")]
    MissingServer,
    #[error("missing API token (set --api-token, REDMINE_API_TOKEN, or api_token in config.toml)")]
    MissingToken,
    #[error("failed to read config file at {0}: {1}")]
    Io(PathBuf, String),
    #[error("failed to parse config file at {0}: {1}")]
    Parse(PathBuf, String),
}

pub fn resolve(overrides: &CliOverrides) -> Result<Config, ConfigError> {
    let file = load_file(overrides.config_path.clone())?;
    let server_url = overrides
        .server_url
        .clone()
        .or_else(|| std::env::var("REDMINE_URL").ok())
        .or(file.server_url)
        .filter(|s| !s.is_empty())
        .ok_or(ConfigError::MissingServer)?;
    let api_token = overrides
        .api_token
        .clone()
        .or_else(|| std::env::var("REDMINE_API_TOKEN").ok())
        .or(file.api_token)
        .filter(|s| !s.is_empty())
        .ok_or(ConfigError::MissingToken)?;
    Ok(Config {
        server_url,
        api_token,
        cf_aliases: file.custom_fields,
    })
}

fn default_config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "redmine-cli")
        .map(|p| p.config_dir().join("config.toml"))
}

fn load_file(explicit: Option<PathBuf>) -> Result<FileConfig, ConfigError> {
    let path = explicit.or_else(default_config_path);
    let Some(path) = path else {
        return Ok(FileConfig::default());
    };
    if !path.exists() {
        return Ok(FileConfig::default());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| ConfigError::Io(path.clone(), e.to_string()))?;
    toml::from_str::<FileConfig>(&text).map_err(|e| ConfigError::Parse(path, e.to_string()))
}

/// `--custom-field` 입력(`id=value` 또는 `alias=value`)을 (cf_id, value) 로 분해한다.
pub fn parse_custom_field(
    spec: &str,
    aliases: &HashMap<String, u64>,
) -> Result<(u64, String), String> {
    let (k, v) = spec
        .split_once('=')
        .ok_or_else(|| format!("--custom-field expects id=value, got: {spec}"))?;
    let id = match k.parse::<u64>() {
        Ok(n) => n,
        Err(_) => *aliases
            .get(k)
            .ok_or_else(|| format!("unknown custom field alias: {k}"))?,
    };
    Ok((id, v.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_custom_field_numeric() {
        let aliases = HashMap::new();
        assert_eq!(parse_custom_field("7=Dev", &aliases).unwrap(), (7, "Dev".into()));
    }

    #[test]
    fn parse_custom_field_alias() {
        let mut aliases = HashMap::new();
        aliases.insert("state".to_string(), 7);
        assert_eq!(parse_custom_field("state=Dev", &aliases).unwrap(), (7, "Dev".into()));
    }

    #[test]
    fn parse_custom_field_unknown_alias() {
        let aliases = HashMap::new();
        assert!(parse_custom_field("zz=Dev", &aliases).is_err());
    }

    #[test]
    fn parse_custom_field_missing_equals() {
        let aliases = HashMap::new();
        assert!(parse_custom_field("Dev", &aliases).is_err());
    }
}
```

- [ ] **Step 2: `thiserror` 의존성 추가**

`Cargo.toml` 의 `[dependencies]` 끝에 추가.

```toml
thiserror = "1"
```

- [ ] **Step 3: 테스트 실행**

Run: `cargo test --lib config::tests`
Expected: 4 PASS.

- [ ] **Step 4: 커밋**

```bash
git add src/config.rs Cargo.toml
git commit -m "feat(config): resolve flag>env>toml with cf alias parser"
```

---

## Phase 3 — CLI 골격

### Task 8: `cli` 디렉터리 + Command enum

**Files:**
- Delete: `src/cli.rs`
- Create: `src/cli/mod.rs`
- Create: `src/cli/projects.rs` (stub)
- Create: `src/cli/categories.rs` (stub)
- Create: `src/cli/issues.rs` (stub)
- Create: `src/cli/time_entries.rs` (stub)
- Create: `src/cli/users.rs` (stub)
- Create: `src/cli/activities.rs` (stub)
- Create: `src/cli/enums.rs` (stub)
- Create: `src/cli/attachments.rs` (stub)
- Modify: `src/main.rs`

- [ ] **Step 1: `src/cli.rs` 삭제**

```bash
rm src/cli.rs
```

- [ ] **Step 2: `src/cli/mod.rs` 작성**

```rust
// CLI 진입점. clap derive 로 정의된 Command 를 적절한 핸들러로 디스패치한다.
use clap::{Parser, Subcommand};

pub mod activities;
pub mod attachments;
pub mod categories;
pub mod enums;
pub mod issues;
pub mod projects;
pub mod time_entries;
pub mod users;

use crate::client::RedmineClient;
use crate::config::{self, CliOverrides, Config};
use crate::output;

#[derive(Parser, Debug)]
#[command(name = "redmine", version, about = "Standalone CLI for Redmine")]
pub struct Cli {
    /// Override server URL (defaults to env REDMINE_URL or config file).
    #[arg(long, global = true)]
    pub server_url: Option<String>,

    /// Override API token (defaults to env REDMINE_API_TOKEN or config file).
    #[arg(long, global = true)]
    pub api_token: Option<String>,

    /// Path to config.toml (defaults to ~/.config/redmine-cli/config.toml).
    #[arg(long, global = true)]
    pub config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
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
    /// Attachments: list/download/upload/delete.
    #[command(subcommand)]
    Attachment(attachments::AttachmentCommand),
}

pub fn run(cli: Cli) {
    let overrides = CliOverrides {
        server_url: cli.server_url,
        api_token: cli.api_token,
        config_path: cli.config,
    };
    let cfg: Config = match config::resolve(&overrides) {
        Ok(c) => c,
        Err(e) => output::print_error(&e.to_string()),
    };
    let client = match RedmineClient::new(&cfg.server_url, &cfg.api_token) {
        Ok(c) => c,
        Err(e) => output::print_error(&e),
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
        Command::Attachment(sub) => attachments::handle(sub, client),
    }
}
```

- [ ] **Step 3: 8개 도메인 스텁 작성**

각 파일을 다음 내용으로 만든다.

`src/cli/projects.rs`:
```rust
// projects 서브커맨드 핸들러.
use clap::Args;

use crate::client::RedmineClient;

#[derive(Args, Debug)]
pub struct ProjectsArgs {
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
}

pub fn handle(_args: ProjectsArgs, _client: &RedmineClient) {
    unimplemented!("projects handler — Task 9");
}
```

`src/cli/categories.rs`:
```rust
// categories 서브커맨드 핸들러.
use clap::Args;

use crate::client::RedmineClient;

#[derive(Args, Debug)]
pub struct CategoriesArgs {
    #[arg(long)]
    pub project: String,
}

pub fn handle(_args: CategoriesArgs, _client: &RedmineClient) {
    unimplemented!("categories handler — Task 10");
}
```

`src/cli/issues.rs`:
```rust
// issues / issue 서브커맨드 핸들러.
use clap::{Args, Subcommand};

use crate::client::RedmineClient;
use crate::config::Config;

#[derive(Args, Debug)]
pub struct IssuesArgs {}

#[derive(Args, Debug)]
pub struct IssueArgs {
    pub id: Option<u64>,
    #[command(subcommand)]
    pub sub: Option<IssueSub>,
}

#[derive(Subcommand, Debug)]
pub enum IssueSub {
    Create,
    Update,
    Delete,
    Relations,
    AddRelation,
    RemoveRelation,
}

pub fn handle_search(_args: IssuesArgs, _client: &RedmineClient, _cfg: &Config) {
    unimplemented!("issues search — Task 11");
}

pub fn handle_one(_args: IssueArgs, _client: &RedmineClient, _cfg: &Config) {
    unimplemented!("issue handler — Task 11");
}
```

`src/cli/time_entries.rs`:
```rust
// time-entry 서브커맨드 핸들러.
use clap::Subcommand;

use crate::client::RedmineClient;

#[derive(Subcommand, Debug)]
pub enum TimeEntryCommand {
    Create,
    List,
    Update,
    Delete,
}

pub fn handle(_cmd: TimeEntryCommand, _client: &RedmineClient) {
    unimplemented!("time-entry handler — Task 12");
}
```

`src/cli/users.rs`:
```rust
// users 서브커맨드 핸들러.
use clap::Args;

use crate::client::RedmineClient;

#[derive(Args, Debug)]
pub struct UsersArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
}

pub fn handle(_args: UsersArgs, _client: &RedmineClient) {
    unimplemented!("users handler — Task 13");
}
```

`src/cli/activities.rs`:
```rust
// activities 서브커맨드 핸들러.
use crate::client::RedmineClient;

pub fn handle(_client: &RedmineClient) {
    unimplemented!("activities handler — Task 14");
}
```

`src/cli/enums.rs`:
```rust
// statuses / trackers / priorities 서브커맨드 핸들러.
use crate::client::RedmineClient;

pub fn statuses(_client: &RedmineClient) {
    unimplemented!("statuses — Task 14");
}
pub fn trackers(_client: &RedmineClient) {
    unimplemented!("trackers — Task 14");
}
pub fn priorities(_client: &RedmineClient) {
    unimplemented!("priorities — Task 14");
}
```

`src/cli/attachments.rs`:
```rust
// attachment 서브커맨드 핸들러.
use clap::Subcommand;

use crate::client::RedmineClient;

#[derive(Subcommand, Debug)]
pub enum AttachmentCommand {
    List,
    Download,
    Upload,
    Delete,
}

pub fn handle(_cmd: AttachmentCommand, _client: &RedmineClient) {
    unimplemented!("attachments handler — Task 15");
}
```

- [ ] **Step 4: `src/main.rs` 갱신**

```rust
// redmine 바이너리 진입점. clap 파싱 후 cli::run 호출.
use clap::Parser;

use redmine_cli::cli::Cli;

fn main() {
    let cli = Cli::parse();
    redmine_cli::cli::run(cli);
}
```

- [ ] **Step 5: 빌드**

Run: `cargo build`
Expected: 에러 없음. 일부 dead_code 경고는 허용.

- [ ] **Step 6: `--help` 동작 확인**

Run: `./target/debug/redmine --help`
Expected stdout: `Standalone CLI for Redmine` 포함, 11개 서브커맨드 (projects/categories/issues/issue/time-entry/users/activities/statuses/trackers/priorities/attachment) 노출.

- [ ] **Step 7: 커밋**

```bash
git add src/
git commit -m "feat(cli): scaffold clap subcommand tree with stubs"
```

---

## Phase 4 — 도메인 핸들러

이후 모든 핸들러는 동일 패턴을 따른다.

1. `Args` 구조체에 플래그 정의 (clap derive).
2. `handle` 함수가 `client.<method>` 호출.
3. 성공이면 `output::print_json`, 실패면 `output::print_error`.
4. tome 원본 `cli/redmine.rs` 의 해당 함수를 참조해 변환.

### Task 9: `projects` 핸들러

**Files:**
- Modify: `src/cli/projects.rs`

- [ ] **Step 1: 구현**

```rust
// projects 서브커맨드 핸들러.
use clap::Args;
use serde::Serialize;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Args, Debug)]
pub struct ProjectsArgs {
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
}

#[derive(Serialize)]
struct ProjectOut {
    id: u64,
    name: String,
    identifier: String,
    description: Option<String>,
    status: Option<u32>,
    created_on: Option<String>,
}

pub fn handle(args: ProjectsArgs, client: &RedmineClient) {
    let resp = match client.list_projects(args.limit, args.offset) {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine projects: {e}")),
    };
    let out: Vec<ProjectOut> = resp
        .projects
        .into_iter()
        .map(|p| ProjectOut {
            id: p.id,
            name: p.name,
            identifier: p.identifier,
            description: p.description,
            status: p.status,
            created_on: p.created_on,
        })
        .collect();
    output::print_json(json!({ "projects": out, "total_count": resp.total_count }));
}
```

- [ ] **Step 2: 빌드**

Run: `cargo build`
Expected: 에러 없음.

- [ ] **Step 3: 커밋**

```bash
git add src/cli/projects.rs
git commit -m "feat(cli): implement projects handler"
```

---

### Task 10: `categories` 핸들러

**Files:**
- Modify: `src/cli/categories.rs`

- [ ] **Step 1: 구현**

```rust
// categories 서브커맨드 핸들러.
use clap::Args;
use serde::Serialize;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Args, Debug)]
pub struct CategoriesArgs {
    /// Project identifier or numeric id.
    #[arg(long)]
    pub project: String,
}

#[derive(Serialize)]
struct CategoryOut {
    id: u64,
    name: String,
    assigned_to: Option<String>,
}

pub fn handle(args: CategoriesArgs, client: &RedmineClient) {
    let resp = match client.list_categories(&args.project) {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine categories: {e}")),
    };
    let out: Vec<CategoryOut> = resp
        .issue_categories
        .into_iter()
        .map(|c| CategoryOut {
            id: c.id,
            name: c.name,
            assigned_to: c.assigned_to.map(|a| a.name),
        })
        .collect();
    output::print_json(json!({ "categories": out, "total_count": resp.total_count }));
}
```

- [ ] **Step 2: 빌드 + 커밋**

```bash
cargo build && git add src/cli/categories.rs && git commit -m "feat(cli): implement categories handler"
```

---

### Task 11: `issues` 핸들러 (검색/조회/CRUD/관계)

**Files:**
- Modify: `src/cli/issues.rs`

이 Task 는 분량이 커서 step 을 명세별로 쪼갠다.

- [ ] **Step 1: 검색(`issues`) 인자 구조체 + 핸들러**

`src/cli/issues.rs` 를 다음으로 교체.

```rust
// issues / issue 서브커맨드 핸들러.
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::config::{self, Config};
use crate::output;

// ── issues (search) ─────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct IssuesArgs {
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long = "assigned-to")]
    pub assigned_to: Option<String>,
    #[arg(long)]
    pub tracker: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
    #[arg(long)]
    pub offset: Option<u64>,
    #[arg(long)]
    pub sort: Option<String>,
    /// Repeatable: --custom-field 7=Dev (or alias from config).
    #[arg(long = "custom-field", value_name = "ID_OR_ALIAS=VALUE")]
    pub custom_field: Vec<String>,
}

#[derive(Serialize)]
struct IssueListOut {
    id: u64,
    subject: String,
    status: Option<String>,
    priority: Option<String>,
    assigned_to: Option<String>,
    project: String,
    updated_on: Option<String>,
}

pub fn handle_search(args: IssuesArgs, client: &RedmineClient, cfg: &Config) {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(v) = args.project {
        params.push(("project_id", v));
    }
    if let Some(v) = args.status {
        params.push(("status_id", v));
    }
    if let Some(v) = args.query {
        params.push(("subject", format!("~{v}")));
    }
    if let Some(v) = args.assigned_to {
        params.push(("assigned_to_id", v));
    }
    if let Some(v) = args.tracker {
        params.push(("tracker_id", v));
    }
    if let Some(v) = args.priority {
        params.push(("priority_id", v));
    }
    params.push(("limit", args.limit.to_string()));
    if let Some(o) = args.offset {
        params.push(("offset", o.to_string()));
    }
    if let Some(v) = args.sort {
        params.push(("sort", v));
    }

    // custom-field 옵션. cf_<id>=value 쿼리 파라미터 변환.
    // (&str, String) 형태 유지 위해 key 를 leak 처리한다.
    for spec in args.custom_field {
        let (id, val) = match config::parse_custom_field(&spec, &cfg.cf_aliases) {
            Ok(p) => p,
            Err(e) => output::print_error(&e),
        };
        let key: &'static str = Box::leak(format!("cf_{id}").into_boxed_str());
        params.push((key, val));
    }

    let resp = match client.search_issues(&params) {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine issues: {e}")),
    };
    let out: Vec<IssueListOut> = resp
        .issues
        .into_iter()
        .map(|i| IssueListOut {
            id: i.id,
            subject: i.subject,
            status: i.status.map(|s| s.name),
            priority: i.priority.map(|p| p.name),
            assigned_to: i.assigned_to.map(|a| a.name),
            project: i.project.name,
            updated_on: i.updated_on,
        })
        .collect();
    output::print_json(json!({ "issues": out, "total_count": resp.total_count }));
}

// ── issue (one) ─────────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct IssueArgs {
    /// Issue ID (positional). Required unless using `create` / `remove-relation`.
    pub id: Option<u64>,
    #[command(subcommand)]
    pub sub: Option<IssueSub>,
}

#[derive(Subcommand, Debug)]
pub enum IssueSub {
    /// Create a new issue.
    Create(IssueCreateArgs),
    /// Update an existing issue (requires <id>).
    Update(IssueUpdateArgs),
    /// Delete an issue (requires <id>).
    Delete,
    /// List relations of an issue.
    Relations,
    /// Add a relation from <id> to --to.
    AddRelation(IssueAddRelationArgs),
    /// Remove a relation by its relation-id.
    RemoveRelation(IssueRemoveRelationArgs),
}

#[derive(Args, Debug, Default)]
pub struct IssueCreateArgs {
    #[arg(long)]
    pub project: String,
    #[arg(long)]
    pub subject: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub tracker: Option<u64>,
    #[arg(long)]
    pub priority: Option<u64>,
    #[arg(long = "assigned-to")]
    pub assigned_to: Option<u64>,
    #[arg(long)]
    pub category: Option<u64>,
    #[arg(long)]
    pub parent: Option<u64>,
    #[arg(long = "start-date")]
    pub start_date: Option<String>,
    #[arg(long = "due-date")]
    pub due_date: Option<String>,
    #[arg(long = "estimated-hours")]
    pub estimated_hours: Option<f64>,
    #[arg(long = "done-ratio")]
    pub done_ratio: Option<u32>,
    #[arg(long = "target-version")]
    pub target_version: Option<u64>,
    #[arg(long = "custom-field", value_name = "ID_OR_ALIAS=VALUE")]
    pub custom_field: Vec<String>,
}

#[derive(Args, Debug, Default)]
pub struct IssueUpdateArgs {
    #[arg(long)]
    pub subject: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub status: Option<u64>,
    #[arg(long)]
    pub tracker: Option<u64>,
    #[arg(long)]
    pub priority: Option<u64>,
    #[arg(long = "assigned-to")]
    pub assigned_to: Option<u64>,
    #[arg(long)]
    pub category: Option<u64>,
    #[arg(long)]
    pub parent: Option<u64>,
    #[arg(long = "start-date")]
    pub start_date: Option<String>,
    #[arg(long = "due-date")]
    pub due_date: Option<String>,
    #[arg(long = "estimated-hours")]
    pub estimated_hours: Option<f64>,
    #[arg(long = "done-ratio")]
    pub done_ratio: Option<u32>,
    #[arg(long = "target-version")]
    pub target_version: Option<u64>,
    #[arg(long)]
    pub notes: Option<String>,
    #[arg(long = "private-notes", default_value_t = false)]
    pub private_notes: bool,
    #[arg(long = "custom-field", value_name = "ID_OR_ALIAS=VALUE")]
    pub custom_field: Vec<String>,
}

#[derive(Args, Debug)]
pub struct IssueAddRelationArgs {
    #[arg(long)]
    pub to: u64,
    #[arg(long, default_value = "relates")]
    pub r#type: String,
}

#[derive(Args, Debug)]
pub struct IssueRemoveRelationArgs {
    /// Relation ID (positional).
    pub relation_id: u64,
}

pub fn handle_one(args: IssueArgs, client: &RedmineClient, cfg: &Config) {
    // 1) id 없이도 가능한 케이스
    match &args.sub {
        Some(IssueSub::Create(c)) => return create(c.clone(), client, cfg),
        Some(IssueSub::RemoveRelation(r)) => return remove_relation(r.relation_id, client),
        _ => {}
    }

    // 2) id 필수
    let id = match args.id {
        Some(n) => n,
        None => output::print_error("redmine issue: <id> is required"),
    };

    match args.sub {
        Some(IssueSub::Update(u)) => update(id, u, client, cfg),
        Some(IssueSub::Delete) => match client.delete_issue(id) {
            Ok(()) => output::print_json(json!({ "ok": true })),
            Err(e) => output::print_error(&format!("failed to delete issue: {e}")),
        },
        Some(IssueSub::Relations) => match client.get_issue(id) {
            Ok(resp) => {
                let rels = resp.issue.relations.unwrap_or_default();
                let items: Vec<Value> = rels
                    .iter()
                    .map(|r| {
                        json!({
                            "id": r.id,
                            "issue_id": r.issue_id,
                            "issue_to_id": r.issue_to_id,
                            "relation_type": r.relation_type,
                        })
                    })
                    .collect();
                output::print_json(json!(items));
            }
            Err(e) => output::print_error(&format!("failed to get relations: {e}")),
        },
        Some(IssueSub::AddRelation(a)) => match client.create_relation(id, a.to, &a.r#type) {
            Ok(resp) => output::print_json(json!({
                "id": resp.relation.id,
                "issue_id": resp.relation.issue_id,
                "issue_to_id": resp.relation.issue_to_id,
                "relation_type": resp.relation.relation_type,
            })),
            Err(e) => output::print_error(&format!("failed to add relation: {e}")),
        },
        None => match client.get_issue(id) {
            Ok(r) => output::print_json(serde_json::to_value(&r.issue).unwrap_or(json!({}))),
            Err(e) => output::print_error(&format!("redmine issue: {e}")),
        },
        Some(IssueSub::Create(_)) | Some(IssueSub::RemoveRelation(_)) => unreachable!(),
    }
}

// (편의를 위해 Args 가 Clone 이 필요)
impl Clone for IssueCreateArgs {
    fn clone(&self) -> Self {
        Self {
            project: self.project.clone(),
            subject: self.subject.clone(),
            description: self.description.clone(),
            tracker: self.tracker,
            priority: self.priority,
            assigned_to: self.assigned_to,
            category: self.category,
            parent: self.parent,
            start_date: self.start_date.clone(),
            due_date: self.due_date.clone(),
            estimated_hours: self.estimated_hours,
            done_ratio: self.done_ratio,
            target_version: self.target_version,
            custom_field: self.custom_field.clone(),
        }
    }
}

fn cf_array(specs: &[String], cfg: &Config) -> Vec<Value> {
    specs
        .iter()
        .map(|spec| match config::parse_custom_field(spec, &cfg.cf_aliases) {
            Ok((id, value)) => json!({ "id": id, "value": value }),
            Err(e) => output::print_error(&e),
        })
        .collect()
}

fn create(a: IssueCreateArgs, client: &RedmineClient, cfg: &Config) {
    let project_id_val: Value = match a.project.parse::<u64>() {
        Ok(n) => json!(n),
        Err(_) => json!(a.project),
    };
    let mut payload = serde_json::Map::new();
    payload.insert("project_id".into(), project_id_val);
    payload.insert("subject".into(), json!(a.subject));
    if let Some(v) = a.description {
        payload.insert("description".into(), json!(v));
    }
    if let Some(v) = a.tracker {
        payload.insert("tracker_id".into(), json!(v));
    }
    if let Some(v) = a.priority {
        payload.insert("priority_id".into(), json!(v));
    }
    if let Some(v) = a.assigned_to {
        payload.insert("assigned_to_id".into(), json!(v));
    }
    if let Some(v) = a.category {
        payload.insert("category_id".into(), json!(v));
    }
    if let Some(v) = a.parent {
        payload.insert("parent_issue_id".into(), json!(v));
    }
    if let Some(v) = a.start_date {
        payload.insert("start_date".into(), json!(v));
    }
    if let Some(v) = a.due_date {
        payload.insert("due_date".into(), json!(v));
    }
    if let Some(v) = a.estimated_hours {
        payload.insert("estimated_hours".into(), json!(v));
    }
    if let Some(v) = a.done_ratio {
        payload.insert("done_ratio".into(), json!(v));
    }
    if let Some(v) = a.target_version {
        payload.insert("fixed_version_id".into(), json!(v));
    }
    let cf = cf_array(&a.custom_field, cfg);
    if !cf.is_empty() {
        payload.insert("custom_fields".into(), json!(cf));
    }

    match client.create_issue(Value::Object(payload)) {
        Ok(r) => output::print_json(serde_json::to_value(&r.issue).unwrap_or(json!({}))),
        Err(e) => output::print_error(&format!("redmine issue create: {e}")),
    }
}

fn update(id: u64, a: IssueUpdateArgs, client: &RedmineClient, cfg: &Config) {
    let mut payload = serde_json::Map::new();
    if let Some(v) = a.subject {
        payload.insert("subject".into(), json!(v));
    }
    if let Some(v) = a.description {
        payload.insert("description".into(), json!(v));
    }
    if let Some(v) = a.status {
        payload.insert("status_id".into(), json!(v));
    }
    if let Some(v) = a.tracker {
        payload.insert("tracker_id".into(), json!(v));
    }
    if let Some(v) = a.priority {
        payload.insert("priority_id".into(), json!(v));
    }
    if let Some(v) = a.assigned_to {
        payload.insert("assigned_to_id".into(), json!(v));
    }
    if let Some(v) = a.category {
        payload.insert("category_id".into(), json!(v));
    }
    if let Some(v) = a.parent {
        payload.insert("parent_issue_id".into(), json!(v));
    }
    if let Some(v) = a.start_date {
        payload.insert("start_date".into(), json!(v));
    }
    if let Some(v) = a.due_date {
        payload.insert("due_date".into(), json!(v));
    }
    if let Some(v) = a.estimated_hours {
        payload.insert("estimated_hours".into(), json!(v));
    }
    if let Some(v) = a.done_ratio {
        payload.insert("done_ratio".into(), json!(v));
    }
    if let Some(v) = a.target_version {
        payload.insert("fixed_version_id".into(), json!(v));
    }
    if let Some(v) = a.notes {
        payload.insert("notes".into(), json!(v));
    }
    if a.private_notes {
        payload.insert("private_notes".into(), json!(true));
    }
    let cf = cf_array(&a.custom_field, cfg);
    if !cf.is_empty() {
        payload.insert("custom_fields".into(), json!(cf));
    }
    if payload.is_empty() {
        output::print_error("redmine issue update: at least one field is required");
    }
    match client.update_issue(id, Value::Object(payload)) {
        Ok(r) => output::print_json(serde_json::to_value(&r.issue).unwrap_or(json!({}))),
        Err(e) => output::print_error(&format!("redmine issue update: {e}")),
    }
}

fn remove_relation(id: u64, client: &RedmineClient) {
    match client.delete_relation(id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("failed to remove relation: {e}")),
    }
}
```

- [ ] **Step 2: 빌드**

Run: `cargo build`
Expected: 에러 없음. (`Box::leak` 사용에 대한 경고 없음.)

- [ ] **Step 3: 커밋**

```bash
git add src/cli/issues.rs
git commit -m "feat(cli): implement issues search and issue CRUD/relations"
```

---

### Task 12: `time-entry` 핸들러

**Files:**
- Modify: `src/cli/time_entries.rs`

- [ ] **Step 1: 구현**

```rust
// time-entry 서브커맨드 핸들러.
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::{json, Value};

use crate::client::RedmineClient;
use crate::output;

#[derive(Subcommand, Debug)]
pub enum TimeEntryCommand {
    /// Create a new time entry.
    Create(CreateArgs),
    /// List time entries.
    List(ListArgs),
    /// Update a time entry.
    Update(UpdateArgs),
    /// Delete a time entry.
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[arg(long)]
    pub issue: u64,
    #[arg(long)]
    pub hours: f64,
    #[arg(long)]
    pub activity: Option<u64>,
    #[arg(long = "spent-on")]
    pub spent_on: Option<String>,
    #[arg(long)]
    pub comment: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long)]
    pub user: Option<String>,
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long)]
    pub issue: Option<String>,
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    pub id: u64,
    #[arg(long)]
    pub hours: Option<f64>,
    #[arg(long)]
    pub activity: Option<u64>,
    #[arg(long)]
    pub comment: Option<String>,
    #[arg(long = "spent-on")]
    pub spent_on: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    pub id: u64,
}

#[derive(Serialize)]
struct TimeEntryOut {
    id: u64,
    issue_id: Option<u64>,
    hours: f64,
    activity: Option<String>,
    spent_on: Option<String>,
    comments: Option<String>,
}

pub fn handle(cmd: TimeEntryCommand, client: &RedmineClient) {
    match cmd {
        TimeEntryCommand::Create(a) => create(a, client),
        TimeEntryCommand::List(a) => list(a, client),
        TimeEntryCommand::Update(a) => update(a, client),
        TimeEntryCommand::Delete(a) => delete(a, client),
    }
}

fn create(a: CreateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    payload.insert("issue_id".into(), json!(a.issue));
    payload.insert("hours".into(), json!(a.hours));
    if let Some(v) = a.activity {
        payload.insert("activity_id".into(), json!(v));
    }
    if let Some(v) = a.spent_on {
        payload.insert("spent_on".into(), json!(v));
    }
    if let Some(v) = a.comment {
        payload.insert("comments".into(), json!(v));
    }
    match client.create_time_entry(Value::Object(payload)) {
        Ok(r) => {
            let out = TimeEntryOut {
                id: r.time_entry.id,
                issue_id: r.time_entry.issue.map(|i| i.id),
                hours: r.time_entry.hours,
                activity: r.time_entry.activity.map(|x| x.name),
                spent_on: r.time_entry.spent_on,
                comments: r.time_entry.comments,
            };
            output::print_json(serde_json::to_value(&out).unwrap_or(json!({})));
        }
        Err(e) => output::print_error(&format!("redmine time-entry create: {e}")),
    }
}

fn list(a: ListArgs, client: &RedmineClient) {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(v) = a.user { params.push(("user_id", v)); }
    if let Some(v) = a.project { params.push(("project_id", v)); }
    if let Some(v) = a.issue { params.push(("issue_id", v)); }
    if let Some(v) = a.from { params.push(("from", v)); }
    if let Some(v) = a.to { params.push(("to", v)); }
    params.push(("limit", a.limit.to_string()));
    match client.list_time_entries(&params) {
        Ok(r) => {
            let entries: Vec<Value> = r
                .time_entries
                .iter()
                .map(|te| {
                    json!({
                        "id": te.id,
                        "issue_id": te.issue.as_ref().map(|i| i.id),
                        "project": te.project.as_ref().map(|p| &p.name),
                        "user": te.user.as_ref().map(|u| &u.name),
                        "activity": te.activity.as_ref().map(|x| &x.name),
                        "hours": te.hours,
                        "comments": te.comments,
                        "spent_on": te.spent_on,
                    })
                })
                .collect();
            output::print_json(json!({ "time_entries": entries, "total_count": r.total_count }));
        }
        Err(e) => output::print_error(&format!("failed to list time entries: {e}")),
    }
}

fn update(a: UpdateArgs, client: &RedmineClient) {
    let mut payload = serde_json::Map::new();
    if let Some(v) = a.hours { payload.insert("hours".into(), json!(v)); }
    if let Some(v) = a.activity { payload.insert("activity_id".into(), json!(v)); }
    if let Some(v) = a.comment { payload.insert("comments".into(), json!(v)); }
    if let Some(v) = a.spent_on { payload.insert("spent_on".into(), json!(v)); }
    if payload.is_empty() {
        output::print_error("time-entry update: no fields to update");
    }
    match client.update_time_entry(a.id, Value::Object(payload)) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("failed to update time entry: {e}")),
    }
}

fn delete(a: DeleteArgs, client: &RedmineClient) {
    match client.delete_time_entry(a.id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("failed to delete time entry: {e}")),
    }
}
```

- [ ] **Step 2: 빌드 + 커밋**

```bash
cargo build && git add src/cli/time_entries.rs && git commit -m "feat(cli): implement time-entry create/list/update/delete"
```

---

### Task 13: `users` 핸들러

**Files:**
- Modify: `src/cli/users.rs`

- [ ] **Step 1: 구현**

```rust
// users 서브커맨드 핸들러.
use clap::Args;
use serde::Serialize;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Args, Debug)]
pub struct UsersArgs {
    #[arg(long)]
    pub name: String,
    #[arg(long, default_value_t = 25)]
    pub limit: u64,
}

#[derive(Serialize)]
struct UserOut {
    id: u64,
    login: Option<String>,
    firstname: Option<String>,
    lastname: Option<String>,
    mail: Option<String>,
}

pub fn handle(args: UsersArgs, client: &RedmineClient) {
    let resp = match client.search_users(&args.name, args.limit) {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine users: {e}")),
    };
    let out: Vec<UserOut> = resp
        .users
        .into_iter()
        .map(|u| UserOut {
            id: u.id,
            login: u.login,
            firstname: u.firstname,
            lastname: u.lastname,
            mail: u.mail,
        })
        .collect();
    output::print_json(json!({ "users": out, "total_count": resp.total_count }));
}
```

- [ ] **Step 2: 빌드 + 커밋**

```bash
cargo build && git add src/cli/users.rs && git commit -m "feat(cli): implement users handler"
```

---

### Task 14: `activities` + `enums`(statuses/trackers/priorities)

**Files:**
- Modify: `src/cli/activities.rs`
- Modify: `src/cli/enums.rs`

- [ ] **Step 1: `activities` 구현**

```rust
// activities 서브커맨드 핸들러.
use serde::Serialize;
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

#[derive(Serialize)]
struct ActivityOut {
    id: u64,
    name: String,
    is_default: Option<bool>,
}

pub fn handle(client: &RedmineClient) {
    let resp = match client.list_activities() {
        Ok(r) => r,
        Err(e) => output::print_error(&format!("redmine activities: {e}")),
    };
    let out: Vec<ActivityOut> = resp
        .time_entry_activities
        .into_iter()
        .map(|a| ActivityOut { id: a.id, name: a.name, is_default: a.is_default })
        .collect();
    output::print_json(json!({ "activities": out }));
}
```

- [ ] **Step 2: `enums` 구현**

```rust
// statuses / trackers / priorities 서브커맨드 핸들러.
use serde_json::json;

use crate::client::RedmineClient;
use crate::output;

pub fn statuses(client: &RedmineClient) {
    match client.list_statuses() {
        Ok(r) => {
            let items: Vec<_> = r
                .issue_statuses
                .iter()
                .map(|s| json!({ "id": s.id, "name": s.name, "is_closed": s.is_closed }))
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine statuses: {e}")),
    }
}

pub fn trackers(client: &RedmineClient) {
    match client.list_trackers() {
        Ok(r) => {
            let items: Vec<_> = r
                .trackers
                .iter()
                .map(|t| json!({ "id": t.id, "name": t.name }))
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine trackers: {e}")),
    }
}

pub fn priorities(client: &RedmineClient) {
    match client.list_priorities() {
        Ok(r) => {
            let items: Vec<_> = r
                .issue_priorities
                .iter()
                .map(|p| json!({ "id": p.id, "name": p.name, "is_default": p.is_default }))
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("redmine priorities: {e}")),
    }
}
```

- [ ] **Step 3: 빌드 + 커밋**

```bash
cargo build && git add src/cli/activities.rs src/cli/enums.rs && git commit -m "feat(cli): implement activities/statuses/trackers/priorities"
```

---

### Task 15: `attachment` 핸들러

**Files:**
- Modify: `src/cli/attachments.rs`

- [ ] **Step 1: 구현**

```rust
// attachment 서브커맨드 핸들러.
use clap::{Args, Subcommand};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::client::RedmineClient;
use crate::output;

#[derive(Subcommand, Debug)]
pub enum AttachmentCommand {
    /// List attachments of an issue.
    List(ListArgs),
    /// Download an attachment by id.
    Download(DownloadArgs),
    /// Upload a file and attach to an issue.
    Upload(UploadArgs),
    /// Delete an attachment.
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long)]
    pub issue: u64,
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Attachment ID.
    pub id: u64,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Args, Debug)]
pub struct UploadArgs {
    #[arg(long)]
    pub issue: u64,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long)]
    pub description: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Attachment ID.
    pub id: u64,
}

pub fn handle(cmd: AttachmentCommand, client: &RedmineClient) {
    match cmd {
        AttachmentCommand::List(a) => list(a, client),
        AttachmentCommand::Download(a) => download(a, client),
        AttachmentCommand::Upload(a) => upload(a, client),
        AttachmentCommand::Delete(a) => delete(a, client),
    }
}

fn list(a: ListArgs, client: &RedmineClient) {
    match client.get_issue(a.issue) {
        Ok(r) => {
            let attachments = r.issue.attachments.unwrap_or_default();
            let items: Vec<Value> = attachments
                .iter()
                .map(|x| {
                    json!({
                        "id": x.id,
                        "filename": x.filename,
                        "filesize": x.filesize,
                        "content_url": x.content_url,
                        "author": x.author.as_ref().map(|au| &au.name),
                        "created_on": x.created_on,
                    })
                })
                .collect();
            output::print_json(json!(items));
        }
        Err(e) => output::print_error(&format!("failed to get attachments: {e}")),
    }
}

fn download(a: DownloadArgs, client: &RedmineClient) {
    let info_path = format!("/attachments/{}.json", a.id);
    let val: Value = match client.get(&info_path, &[]) {
        Ok(v) => v,
        Err(e) => output::print_error(&format!("failed to get attachment info: {e}")),
    };
    let url = val
        .get("attachment")
        .and_then(|x| x.get("content_url"))
        .and_then(|u| u.as_str())
        .unwrap_or_else(|| output::print_error("attachment: content_url not found"));
    match client.download_attachment(url, &a.output) {
        Ok(()) => output::print_json(json!({ "ok": true, "path": a.output.display().to_string() })),
        Err(e) => output::print_error(&format!("download failed: {e}")),
    }
}

fn upload(a: UploadArgs, client: &RedmineClient) {
    if !a.file.exists() {
        output::print_error(&format!("file not found: {}", a.file.display()));
    }
    let filename = a
        .file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload");
    let content_type = "application/octet-stream";
    match client.upload_file(&a.file) {
        Ok(upload_resp) => match client.attach_to_issue(
            a.issue,
            &upload_resp.upload.token,
            filename,
            content_type,
            a.description.as_deref(),
        ) {
            Ok(()) => output::print_json(json!({ "ok": true, "token": upload_resp.upload.token })),
            Err(e) => output::print_error(&format!("failed to attach: {e}")),
        },
        Err(e) => output::print_error(&format!("upload failed: {e}")),
    }
}

fn delete(a: DeleteArgs, client: &RedmineClient) {
    match client.delete_attachment(a.id) {
        Ok(()) => output::print_json(json!({ "ok": true })),
        Err(e) => output::print_error(&format!("failed to delete attachment: {e}")),
    }
}
```

- [ ] **Step 2: 빌드 + 커밋**

```bash
cargo build && git add src/cli/attachments.rs && git commit -m "feat(cli): implement attachment list/download/upload/delete"
```

---

## Phase 5 — 테스트

### Task 16: clap 파싱 테스트

**Files:**
- Create: `tests/cli_parse.rs`

- [ ] **Step 1: 파싱 테스트 작성**

```rust
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
```

- [ ] **Step 2: `Cli` 와 서브 모듈을 lib 에서 노출 확인**

`src/cli/mod.rs` 의 `pub use` 가 필요하지 않다면 `redmine_cli::cli::issues::IssueSub` 처럼 풀 경로로 접근한다. 위 테스트가 그대로 동작해야 한다.

- [ ] **Step 3: 테스트 실행**

Run: `cargo test --test cli_parse`
Expected: 5 PASS.

- [ ] **Step 4: 커밋**

```bash
git add tests/cli_parse.rs
git commit -m "test(cli): add clap derive parsing assertions"
```

---

### Task 17: wiremock 통합 테스트 (golden path)

**Files:**
- Create: `tests/integration.rs`

- [ ] **Step 1: 통합 테스트 작성**

```rust
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
```

- [ ] **Step 2: 테스트 실행**

Run: `cargo test --test integration`
Expected: 2 PASS.

- [ ] **Step 3: 커밋**

```bash
git add tests/integration.rs
git commit -m "test: add wiremock integration coverage for issues"
```

---

## Phase 6 — 측정 + 최적화

### Task 18: 베이스라인 측정 + release profile 튜닝

**Files:**
- Modify: `Cargo.toml`
- Create: `docs/superpowers/notes/perf-baseline.md`

- [ ] **Step 1: 측정 도구 설치 (호스트)**

```bash
brew install hyperfine
cargo install cargo-bloat
```

- [ ] **Step 2: 베이스라인 측정**

```bash
cargo build --release
ls -lh target/release/redmine
hyperfine --warmup 2 'target/release/redmine --help'
cargo bloat --release --crates -n 20
```

결과를 `docs/superpowers/notes/perf-baseline.md` 에 저장한다.

```markdown
# 성능 베이스라인 (Phase 6 시작 시점)

| 측정 | 값 |
| --- | --- |
| 바이너리 크기 |  |
| `redmine --help` 평균 (hyperfine) |  |
| top 5 crate (cargo bloat) |  |

`<측정 결과 붙여넣기>`
```

- [ ] **Step 3: release profile 튜닝**

`Cargo.toml` 의 `[profile.release]` 를 교체.

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = "symbols"
panic = "abort"
```

- [ ] **Step 4: 재측정 + 기록**

같은 측정 3개를 다시 수행하고 `perf-baseline.md` 에 비교 표로 추가한다.

- [ ] **Step 5: 회귀 없음 확인**

```bash
cargo test
```

Expected: 모든 테스트 PASS. panic=abort 라도 테스트는 별도 프로파일이라 무관.

- [ ] **Step 6: 커밋**

```bash
git add Cargo.toml docs/
git commit -m "perf: tune release profile (LTO, codegen-units=1, strip, panic=abort)"
```

---

### Task 19: 의존성 다이어트 + 할당/panic 정리

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/cli/issues.rs` (필요시)
- Modify: `docs/superpowers/notes/perf-baseline.md`

- [ ] **Step 1: 불필요 feature 점검**

`Cargo.toml` 의 reqwest/toml/serde feature 가 최소인지 확인. 다음과 같은지 검토.

```toml
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
toml = { version = "0.9", default-features = false, features = ["parse"] }
serde = { version = "1", features = ["derive"] }
```

`cargo bloat --release --crates -n 20` 결과를 비교해 top 항목을 다시 측정.

- [ ] **Step 2: `unwrap()` / `expect()` 인벤토리**

Run: `rg "\.unwrap\(\)|\.expect\(" src/ tests/`
사용자 입력 / API 응답 영향권의 `unwrap` 은 `match` 또는 `unwrap_or` 로 교체한다. 예: `payload.as_object_mut().unwrap()` 같은 *직전 줄에서 객체로 만든* 경우는 안전하므로 유지. tome 와 동일 패턴.

- [ ] **Step 3: `Box::leak` 사용 재검토 (issues.rs 의 `cf_<id>` 키)**

대안. 키를 `String` 으로 바꿔 `Vec<(String, String)>` 으로 모은 뒤 `client.get` 시그니처를 `&[(&str, String)]` → `params.iter().map(|(k, v)| (k.as_str(), v.clone()))` 로 어댑트하는 작은 헬퍼 추가. 메모리 누수 0. issues.rs Step 1 을 다시 수정.

`src/cli/issues.rs` 의 검색 부분 변경 예.

```rust
let mut params: Vec<(String, String)> = Vec::new();
// ... params.push(("project_id".into(), v)) ...
for spec in args.custom_field {
    let (id, val) = match config::parse_custom_field(&spec, &cfg.cf_aliases) {
        Ok(p) => p,
        Err(e) => output::print_error(&e),
    };
    params.push((format!("cf_{id}"), val));
}
let borrowed: Vec<(&str, String)> = params.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
let resp = client.search_issues(&borrowed);
```

기존 callers 도 동일 어댑트 적용.

- [ ] **Step 4: 재측정**

같은 hyperfine / bloat 측정을 다시 수행하고 `perf-baseline.md` 에 3차 비교 추가.

- [ ] **Step 5: 테스트 회귀 확인**

```bash
cargo test
```

Expected: 모든 테스트 PASS.

- [ ] **Step 6: 커밋**

```bash
git add Cargo.toml src/ docs/
git commit -m "perf: trim deps, remove Box::leak, harden unwrap callsites"
```

---

## Phase 7 — 배포

### Task 20: CI / Release GitHub Actions

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: `ci.yml` 작성**

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all-targets
```

- [ ] **Step 2: `release.yml` 작성**

```yaml
name: Release
on:
  push:
    tags:
      - "v*"

jobs:
  build:
    runs-on: ${{ matrix.runner }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: aarch64-apple-darwin
            runner: macos-14
            archive: redmine-${{ github.ref_name }}-aarch64-apple-darwin.tar.gz
          - target: x86_64-apple-darwin
            runner: macos-13
            archive: redmine-${{ github.ref_name }}-x86_64-apple-darwin.tar.gz
          - target: x86_64-unknown-linux-gnu
            runner: ubuntu-latest
            archive: redmine-${{ github.ref_name }}-x86_64-unknown-linux-gnu.tar.gz
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --target ${{ matrix.target }}
      - name: Package
        run: |
          mkdir -p dist
          tar -C target/${{ matrix.target }}/release -czf dist/${{ matrix.archive }} redmine
          shasum -a 256 dist/${{ matrix.archive }} > dist/${{ matrix.archive }}.sha256
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.archive }}
          path: dist/*

  publish:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: dist
          merge-multiple: true
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: dist/*
```

- [ ] **Step 3: 로컬 lint 확인 (없으면 패스)**

```bash
which actionlint && actionlint .github/workflows/*.yml || true
```

- [ ] **Step 4: 커밋**

```bash
git add .github/workflows/
git commit -m "ci: add CI and release workflows for multi-target binaries"
```

---

### Task 21: GitHub repo 생성 + 첫 릴리스

**Files:** (저장소 외부 작업)

- [ ] **Step 1: `gh` CLI 로 zacostudio org 에 repo 생성**

```bash
gh repo create zacostudio/redmine-cli \
  --public \
  --description "Standalone CLI for Redmine" \
  --source . \
  --remote origin \
  --push
```

Expected: `https://github.com/zacostudio/redmine-cli` 가 생성되고 `main` 이 푸시된다.

- [ ] **Step 2: CI 통과 확인**

```bash
gh run watch
```

Expected: ci 워크플로 SUCCESS.

- [ ] **Step 3: v0.1.0 태그 푸시**

```bash
git tag -a v0.1.0 -m "v0.1.0 — first release"
git push origin v0.1.0
```

- [ ] **Step 4: 릴리스 워크플로 완료 확인**

```bash
gh run watch
gh release view v0.1.0
```

Expected: 3개 tarball + 3개 .sha256 파일 첨부, 릴리스 페이지 노출.

- [ ] **Step 5: sha256 값 메모**

```bash
mkdir -p docs/superpowers/notes
gh release view v0.1.0 --json assets --jq '.assets[].name' > docs/superpowers/notes/release-v0.1.0-shas.txt
for asset in $(gh release view v0.1.0 --json assets --jq '.assets[].name' | grep '\.sha256$'); do
  echo "=== $asset ===" >> docs/superpowers/notes/release-v0.1.0-shas.txt
  gh release download v0.1.0 -p "$asset" -O - >> docs/superpowers/notes/release-v0.1.0-shas.txt
done
```

- [ ] **Step 6: 커밋 (메모 한정)**

```bash
git add docs/superpowers/notes/release-v0.1.0-shas.txt
git commit -m "docs: record v0.1.0 release SHA256 values"
git push
```

---

### Task 22: Homebrew tap 저장소 셋업

**Files:** (별도 저장소)
- Create: `Formula/redmine.rb` (in `zacostudio/homebrew-redmine`)
- Create: `README.md` (in `zacostudio/homebrew-redmine`)

- [ ] **Step 1: 별도 디렉터리에서 tap 저장소 생성**

```bash
cd /tmp
gh repo create zacostudio/homebrew-redmine \
  --public \
  --description "Homebrew tap for redmine-cli" \
  --clone
cd homebrew-redmine
mkdir Formula
```

- [ ] **Step 2: `Formula/redmine.rb` 작성**

Step 5(Task 21) 의 sha256 값을 채워 넣는다. 예시.

```ruby
class Redmine < Formula
  desc "Standalone CLI for Redmine"
  homepage "https://github.com/zacostudio/redmine-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/zacostudio/redmine-cli/releases/download/v#{version}/redmine-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "<aarch64-apple-darwin SHA>"
    end
    on_intel do
      url "https://github.com/zacostudio/redmine-cli/releases/download/v#{version}/redmine-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "<x86_64-apple-darwin SHA>"
    end
  end

  on_linux do
    url "https://github.com/zacostudio/redmine-cli/releases/download/v#{version}/redmine-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "<x86_64-linux SHA>"
  end

  def install
    bin.install "redmine"
  end

  test do
    assert_match "redmine", shell_output("#{bin}/redmine --version")
  end
end
```

- [ ] **Step 3: `README.md` 작성**

```markdown
# homebrew-redmine

Homebrew tap for [redmine-cli](https://github.com/zacostudio/redmine-cli).

## Install

```bash
brew tap zacostudio/redmine
brew install redmine
```
```

- [ ] **Step 4: 커밋 + 푸시**

```bash
git add Formula/redmine.rb README.md
git -c user.email=jhyoung75@gmail.com -c user.name="Jin Hyoung" commit -m "feat: add v0.1.0 formula"
git push origin main
```

- [ ] **Step 5: 로컬 설치 검증**

```bash
brew tap zacostudio/redmine
brew install --build-from-source --verbose redmine || brew install redmine
redmine --version
```

Expected: `redmine 0.1.0` 출력.

- [ ] **Step 6: smoke test (실제 Redmine 자격증명 필요)**

```bash
export REDMINE_URL=https://your.redmine
export REDMINE_API_TOKEN=...
redmine projects --limit 1
```

Expected: `{"projects":[...],"total_count":N}` 형태 JSON.

---

## 자체 점검 (Self-Review)

### Spec coverage
- §2 아키텍처 → Tasks 1, 2, 8 (스캐폴드 + cli 디렉터리).
- §3 명령 구조 → Tasks 9~15 (전 서브커맨드 핸들러).
- §4 설정/데이터 흐름 → Task 7 (config) + Task 8 (Cli derive + run).
- §5.1 테스트 → Tasks 16, 17.
- §5.2 빌드/배포 (Homebrew) → Tasks 20, 21, 22.
- §5.3 최적화 → Tasks 18, 19.

### Placeholder scan
- `<aarch64-apple-darwin SHA>` 등은 의도된 placeholder. Task 22 Step 2 가 Task 21 Step 5 의 측정값으로 치환한다. 명세 충족.
- 그 외 "TBD", "TODO", "later" 없음.

### Type consistency
- `Config`, `CliOverrides`, `RedmineClient` 시그니처가 Task 7, Task 8, Task 11~15 에서 일관.
- `IssueSub` 변형 (Create / Update / Delete / Relations / AddRelation / RemoveRelation) 가 Task 8 stub 과 Task 11 구현에서 일치.
- Task 19 에서 `Vec<(&'static str, String)>` (Box::leak) → `Vec<(String, String)>` 으로 변경 후 호출처 어댑트 명시. issues.rs 외 다른 핸들러는 `&[(&str, String)]` 그대로 사용 가능 (어댑트 불필요).
