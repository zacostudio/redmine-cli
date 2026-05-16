// Redmine REST API 호출용 reqwest blocking 클라이언트.
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::types::*;

/// 클라이언트 계층 에러.
/// - `Http`: 서버가 4xx/5xx 를 반환한 경우. status code 로 상위 계층 분기 가능.
/// - `Other`: 전송/파싱/IO 등 그 외 모두. 호출 컨텍스트별 prefix 메시지를 보존하기 위해 String 으로 둔다.
///
/// Display 텍스트는 기존의 단순 `String` 에러 시절과 동일해서 호출자 코드의 `format!("{e}")` 출력이 그대로 유지된다.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP {status} — {body}")]
    Http { status: u16, body: String },
    #[error("{0}")]
    Other(String),
}

impl From<String> for ClientError {
    fn from(s: String) -> Self {
        ClientError::Other(s)
    }
}

pub struct RedmineClient {
    client: Client,
    base_url: String,
}

impl RedmineClient {
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ClientError> {
        let mut headers = HeaderMap::new();
        // API 키는 모든 요청에 따라가야 하므로 default 에 둔다.
        // Content-Type 은 body 가 있는 요청만 필요하므로 reqwest 의 .json()/명시 헤더가 자동 부여하도록 맡긴다.
        headers.insert(
            "X-Redmine-API-Key",
            HeaderValue::from_str(api_key).map_err(|e| format!("invalid API key: {e}"))?,
        );

        // 리다이렉트를 따라가면 X-Redmine-API-Key 가 임의 호스트로 전송될 수 있다.
        // Redmine 정상 사용 흐름에서 cross-origin 리다이렉트는 발생하지 않으므로 전체 차단한다.
        let client = Client::builder()
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("failed to create HTTP client: {e}"))?;

        let base_url = base_url.trim_end_matches('/').to_string();
        Ok(Self { client, base_url })
    }

    pub(crate) fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<T, ClientError> {
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
            return Err(ClientError::Http { status: status.as_u16(), body });
        }
        resp.json::<T>()
            .map_err(|e| ClientError::Other(format!("failed to parse response: {e}")))
    }

    fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T, ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ClientError::Http { status: status.as_u16(), body });
        }
        resp.json::<T>()
            .map_err(|e| ClientError::Other(format!("failed to parse response: {e}")))
    }

    fn post_no_content(&self, path: &str, body: &serde_json::Value) -> Result<(), ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ClientError::Http { status: status.as_u16(), body });
        }
        Ok(())
    }

    fn put_no_content(&self, path: &str, body: &serde_json::Value) -> Result<(), ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .put(&url)
            .json(body)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ClientError::Http { status: status.as_u16(), body });
        }
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), ClientError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .delete(&url)
            .send()
            .map_err(|e| format!("request failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ClientError::Http { status: status.as_u16(), body });
        }
        Ok(())
    }

    // ── API methods ─────────────────────────────────────────────────

    pub fn search_issues(&self, params: &[(&str, String)]) -> Result<IssuesResponse, ClientError> {
        self.get("/issues.json", params)
    }

    /// `include` 가 비어 있으면 include 파라미터를 보내지 않는다.
    /// 일반 조회는 `&["journals","attachments","children","relations"]`,
    /// 부분 조회(첨부만, relations 만 등)는 필요한 키만 전달한다.
    pub fn get_issue(&self, id: u64, include: &[&str]) -> Result<IssueResponse, ClientError> {
        let path = format!("/issues/{}.json", id);
        if include.is_empty() {
            self.get(&path, &[])
        } else {
            self.get(&path, &[("include", include.join(","))])
        }
    }

    pub fn create_issue(&self, payload: serde_json::Value) -> Result<IssueResponse, ClientError> {
        self.post("/issues.json", &serde_json::json!({ "issue": payload }))
    }

    /// PUT 만 수행한다. 갱신된 본문을 다시 받고 싶으면 호출자가 별도로 `get_issue` 를 호출한다.
    pub fn update_issue(&self, id: u64, payload: serde_json::Value) -> Result<(), ClientError> {
        let body = serde_json::json!({ "issue": payload });
        self.put_no_content(&format!("/issues/{}.json", id), &body)
    }

    pub fn list_projects(&self, limit: u64, offset: u64) -> Result<ProjectsResponse, ClientError> {
        self.get(
            "/projects.json",
            &[("limit", limit.to_string()), ("offset", offset.to_string())],
        )
    }

    pub fn list_categories(&self, project_id: &str) -> Result<CategoriesResponse, ClientError> {
        let path = format!("/projects/{}/issue_categories.json", project_id);
        self.get(&path, &[])
    }

    pub fn create_time_entry(
        &self,
        payload: serde_json::Value,
    ) -> Result<TimeEntryResponse, ClientError> {
        self.post(
            "/time_entries.json",
            &serde_json::json!({ "time_entry": payload }),
        )
    }

    pub fn search_users(&self, name: &str, limit: u64) -> Result<UsersResponse, ClientError> {
        self.get(
            "/users.json",
            &[("name", name.to_string()), ("limit", limit.to_string())],
        )
    }

    pub fn list_activities(&self) -> Result<ActivitiesResponse, ClientError> {
        self.get("/enumerations/time_entry_activities.json", &[])
    }

    pub fn delete_issue(&self, id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/issues/{}.json", id))
    }

    pub fn delete_attachment(&self, id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/attachments/{}.json", id))
    }

    /// `/attachments/{id}.json` 의 메타데이터(특히 download 에 필요한 `content_url`)를 raw JSON 으로 반환.
    /// 전용 타입을 만들기엔 단발성 호출이라 `serde_json::Value` 로 둔다.
    pub fn get_attachment_info(&self, id: u64) -> Result<serde_json::Value, ClientError> {
        self.get(&format!("/attachments/{}.json", id), &[])
    }

    pub fn upload_file(&self, file_path: &std::path::Path) -> Result<UploadResponse, ClientError> {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload");
        // 파일 전체를 메모리에 읽지 않고 reqwest::blocking::Body 로 스트리밍한다.
        // sized body 가 필요하므로 metadata 로 길이를 미리 얻는다.
        let file = std::fs::File::open(file_path)
            .map_err(|e| format!("failed to open file: {e}"))?;
        let len = file
            .metadata()
            .map_err(|e| format!("failed to stat file: {e}"))?
            .len();
        let body = reqwest::blocking::Body::sized(file, len);
        let url = format!(
            "{}/uploads.json?filename={}",
            self.base_url,
            urlencoding::encode(filename)
        );
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(body)
            .send()
            .map_err(|e| format!("upload failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(ClientError::Http { status: status.as_u16(), body });
        }
        resp.json::<UploadResponse>()
            .map_err(|e| ClientError::Other(format!("failed to parse upload response: {e}")))
    }

    pub fn attach_to_issue(
        &self,
        issue_id: u64,
        token: &str,
        filename: &str,
        content_type: &str,
        description: Option<&str>,
    ) -> Result<(), ClientError> {
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
    ) -> Result<(), ClientError> {
        // 본문 전체를 메모리에 적재하지 않고 파일에 직접 흘려보낸다.
        let mut resp = self
            .client
            .get(url)
            .send()
            .map_err(|e| format!("download failed: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(ClientError::Http { status: status.as_u16(), body: String::new() });
        }
        let mut file = std::fs::File::create(output_path)
            .map_err(|e| format!("write failed: {e}"))?;
        resp.copy_to(&mut file)
            .map_err(|e| format!("write failed: {e}"))?;
        Ok(())
    }

    pub fn list_time_entries(
        &self,
        params: &[(&str, String)],
    ) -> Result<TimeEntriesResponse, ClientError> {
        self.get("/time_entries.json", params)
    }

    pub fn update_time_entry(&self, id: u64, payload: serde_json::Value) -> Result<(), ClientError> {
        let body = serde_json::json!({ "time_entry": payload });
        self.put_no_content(&format!("/time_entries/{}.json", id), &body)
    }

    pub fn delete_time_entry(&self, id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/time_entries/{}.json", id))
    }

    pub fn create_relation(
        &self,
        issue_id: u64,
        target_id: u64,
        relation_type: &str,
    ) -> Result<RelationResponse, ClientError> {
        let body = serde_json::json!({
            "relation": {
                "issue_to_id": target_id,
                "relation_type": relation_type,
            }
        });
        let path = format!("/issues/{}/relations.json", issue_id);
        self.post(&path, &body)
    }

    pub fn delete_relation(&self, id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/relations/{}.json", id))
    }

    pub fn list_statuses(&self) -> Result<StatusesResponse, ClientError> {
        self.get("/issue_statuses.json", &[])
    }

    pub fn list_trackers(&self) -> Result<TrackersResponse, ClientError> {
        self.get("/trackers.json", &[])
    }

    pub fn list_priorities(&self) -> Result<PrioritiesResponse, ClientError> {
        self.get("/enumerations/issue_priorities.json", &[])
    }

    pub fn list_roles(&self) -> Result<RolesResponse, ClientError> {
        self.get("/roles.json", &[])
    }

    pub fn list_document_categories(&self) -> Result<DocumentCategoriesResponse, ClientError> {
        self.get("/enumerations/document_categories.json", &[])
    }

    pub fn list_custom_fields(&self) -> Result<CustomFieldsResponse, ClientError> {
        self.get("/custom_fields.json", &[])
    }

    pub fn search(&self, params: &[(&str, String)]) -> Result<SearchResponse, ClientError> {
        self.get("/search.json", params)
    }

    pub fn list_versions(&self, project_id: &str) -> Result<VersionsResponse, ClientError> {
        self.get(&format!("/projects/{}/versions.json", project_id), &[])
    }

    pub fn get_version(&self, id: u64) -> Result<VersionResponse, ClientError> {
        self.get(&format!("/versions/{}.json", id), &[])
    }

    pub fn create_version(
        &self,
        project_id: &str,
        payload: serde_json::Value,
    ) -> Result<VersionResponse, ClientError> {
        self.post(
            &format!("/projects/{}/versions.json", project_id),
            &serde_json::json!({ "version": payload }),
        )
    }

    pub fn update_version(&self, id: u64, payload: serde_json::Value) -> Result<(), ClientError> {
        self.put_no_content(
            &format!("/versions/{}.json", id),
            &serde_json::json!({ "version": payload }),
        )
    }

    pub fn delete_version(&self, id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/versions/{}.json", id))
    }

    pub fn list_memberships(&self, project_id: &str) -> Result<MembershipsResponse, ClientError> {
        self.get(
            &format!("/projects/{}/memberships.json", project_id),
            &[],
        )
    }

    pub fn get_membership(&self, id: u64) -> Result<MembershipResponse, ClientError> {
        self.get(&format!("/memberships/{}.json", id), &[])
    }

    pub fn create_membership(
        &self,
        project_id: &str,
        payload: serde_json::Value,
    ) -> Result<MembershipResponse, ClientError> {
        self.post(
            &format!("/projects/{}/memberships.json", project_id),
            &serde_json::json!({ "membership": payload }),
        )
    }

    pub fn update_membership(&self, id: u64, payload: serde_json::Value) -> Result<(), ClientError> {
        self.put_no_content(
            &format!("/memberships/{}.json", id),
            &serde_json::json!({ "membership": payload }),
        )
    }

    pub fn delete_membership(&self, id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/memberships/{}.json", id))
    }

    pub fn list_news(
        &self,
        project: Option<&str>,
        params: &[(&str, String)],
    ) -> Result<NewsListResponse, ClientError> {
        let path = match project {
            Some(p) => format!("/projects/{}/news.json", p),
            None => "/news.json".to_string(),
        };
        self.get(&path, params)
    }

    pub fn get_news(&self, id: u64) -> Result<NewsResponse, ClientError> {
        self.get(&format!("/news/{}.json", id), &[])
    }

    pub fn create_news(
        &self,
        project: &str,
        payload: serde_json::Value,
    ) -> Result<NewsResponse, ClientError> {
        self.post(
            &format!("/projects/{}/news.json", project),
            &serde_json::json!({ "news": payload }),
        )
    }

    pub fn list_files(&self, project: &str) -> Result<FilesResponse, ClientError> {
        self.get(&format!("/projects/{}/files.json", project), &[])
    }

    pub fn attach_file_to_project(
        &self,
        project: &str,
        payload: serde_json::Value,
    ) -> Result<(), ClientError> {
        self.post_no_content(
            &format!("/projects/{}/files.json", project),
            &serde_json::json!({ "file": payload }),
        )
    }

    pub fn list_queries(&self) -> Result<QueriesResponse, ClientError> {
        self.get("/queries.json", &[])
    }

    pub fn list_wiki_pages(&self, project: &str) -> Result<WikiIndexResponse, ClientError> {
        self.get(&format!("/projects/{}/wiki/index.json", project), &[])
    }

    pub fn get_wiki_page(
        &self,
        project: &str,
        title: &str,
    ) -> Result<WikiPageResponse, ClientError> {
        let encoded = urlencoding::encode(title);
        self.get(
            &format!("/projects/{}/wiki/{}.json", project, encoded),
            &[("include", "attachments".to_string())],
        )
    }

    pub fn put_wiki_page(
        &self,
        project: &str,
        title: &str,
        payload: serde_json::Value,
    ) -> Result<(), ClientError> {
        let encoded = urlencoding::encode(title);
        self.put_no_content(
            &format!("/projects/{}/wiki/{}.json", project, encoded),
            &serde_json::json!({ "wiki_page": payload }),
        )
    }

    pub fn delete_wiki_page(&self, project: &str, title: &str) -> Result<(), ClientError> {
        let encoded = urlencoding::encode(title);
        self.delete(&format!("/projects/{}/wiki/{}.json", project, encoded))
    }

    pub fn list_groups(&self) -> Result<GroupsResponse, ClientError> {
        self.get("/groups.json", &[])
    }

    pub fn get_group(&self, id: u64) -> Result<GroupResponse, ClientError> {
        self.get(
            &format!("/groups/{}.json", id),
            &[("include", "users,memberships".to_string())],
        )
    }

    pub fn create_group(&self, payload: serde_json::Value) -> Result<GroupResponse, ClientError> {
        self.post(
            "/groups.json",
            &serde_json::json!({ "group": payload }),
        )
    }

    pub fn update_group(&self, id: u64, payload: serde_json::Value) -> Result<(), ClientError> {
        self.put_no_content(
            &format!("/groups/{}.json", id),
            &serde_json::json!({ "group": payload }),
        )
    }

    pub fn delete_group(&self, id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/groups/{}.json", id))
    }

    pub fn add_user_to_group(&self, group_id: u64, user_id: u64) -> Result<(), ClientError> {
        self.post_no_content(
            &format!("/groups/{}/users.json", group_id),
            &serde_json::json!({ "user_id": user_id }),
        )
    }

    pub fn remove_user_from_group(&self, group_id: u64, user_id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/groups/{}/users/{}.json", group_id, user_id))
    }

    pub fn get_my_account(&self) -> Result<UserResponse, ClientError> {
        self.get("/my/account.json", &[])
    }

    pub fn update_my_account(&self, payload: serde_json::Value) -> Result<(), ClientError> {
        self.put_no_content(
            "/my/account.json",
            &serde_json::json!({ "user": payload }),
        )
    }

    pub fn add_issue_watcher(&self, issue_id: u64, user_id: u64) -> Result<(), ClientError> {
        self.post_no_content(
            &format!("/issues/{}/watchers.json", issue_id),
            &serde_json::json!({ "user_id": user_id }),
        )
    }

    pub fn remove_issue_watcher(&self, issue_id: u64, user_id: u64) -> Result<(), ClientError> {
        self.delete(&format!("/issues/{}/watchers/{}.json", issue_id, user_id))
    }
}
