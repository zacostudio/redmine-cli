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

// ── Roles ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RolesResponse {
    pub roles: Vec<IdName>,
}

// ── Document categories ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DocumentCategoriesResponse {
    pub document_categories: Vec<RedmineDocumentCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineDocumentCategory {
    pub id: u64,
    pub name: String,
    pub is_default: Option<bool>,
}

// ── Versions ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineVersion {
    pub id: u64,
    pub project: Option<IdName>,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub due_date: Option<String>,
    pub sharing: Option<String>,
    pub wiki_page_title: Option<String>,
    pub created_on: Option<String>,
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VersionsResponse {
    pub versions: Vec<RedmineVersion>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct VersionResponse {
    pub version: RedmineVersion,
}

// ── News ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineNews {
    pub id: u64,
    pub project: Option<IdName>,
    pub author: Option<IdName>,
    pub title: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub created_on: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewsListResponse {
    pub news: Vec<RedmineNews>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct NewsResponse {
    pub news: RedmineNews,
}

// ── Memberships ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineMembership {
    pub id: u64,
    pub project: Option<IdName>,
    pub user: Option<IdName>,
    pub group: Option<IdName>,
    pub roles: Option<Vec<IdName>>,
}

#[derive(Debug, Deserialize)]
pub struct MembershipsResponse {
    pub memberships: Vec<RedmineMembership>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct MembershipResponse {
    pub membership: RedmineMembership,
}

// ── Search ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<RedmineSearchResult>,
    pub total_count: Option<u64>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineSearchResult {
    pub id: u64,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub datetime: Option<String>,
}

// ── Custom fields (metadata) ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CustomFieldsResponse {
    pub custom_fields: Vec<RedmineCustomFieldMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedmineCustomFieldMeta {
    pub id: u64,
    pub name: String,
    pub customized_type: Option<String>,
    pub field_format: Option<String>,
    pub is_required: Option<bool>,
    pub is_filter: Option<bool>,
    pub multiple: Option<bool>,
    pub default_value: Option<serde_json::Value>,
    pub visible: Option<bool>,
    pub possible_values: Option<serde_json::Value>,
}
