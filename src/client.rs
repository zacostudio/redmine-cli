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
            &[(
                "include",
                "journals,attachments,children,relations".to_string(),
            )],
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

    pub fn list_roles(&self) -> Result<RolesResponse, String> {
        self.get("/roles.json", &[])
    }

    pub fn list_document_categories(&self) -> Result<DocumentCategoriesResponse, String> {
        self.get("/enumerations/document_categories.json", &[])
    }

    pub fn list_custom_fields(&self) -> Result<CustomFieldsResponse, String> {
        self.get("/custom_fields.json", &[])
    }

    pub fn search(&self, params: &[(&str, String)]) -> Result<SearchResponse, String> {
        self.get("/search.json", params)
    }
}
