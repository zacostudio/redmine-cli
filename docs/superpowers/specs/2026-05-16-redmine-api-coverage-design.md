# Redmine REST API 전면 커버리지 — v0.2.0 설계

생성일: 2026-05-16  
대상 릴리스: v0.2.0  
상태: 승인됨 (사용자 자율 진행 지시)

## 배경

`v0.1.x` 시점의 `redmine-cli`는 Redmine REST API 중 자주 쓰이는 일부만 노출하고 있다. 사용자 요구에 따라 미구현 14개 항목을 모두 추가하여 사실상 전체 커버리지를 확보한다. 단일 릴리스(v0.2.0)로 묶어 배포하되, 구현·커밋은 기능별로 누적한다.

## 비범위(Non-goals)

- 새 출력 형식(YAML/CSV 등) 추가 안 함. JSON만 유지.
- 인증 방식 변경 없음. 기존 `X-Redmine-API-Key` 헤더 유지.
- `RedmineClient`에 신규 HTTP 메서드 추가 안 함 (`get`/`post`/`put_no_content`/`delete` 재사용).
- Redmine 5+ 전용 동작은 안전한 공통분만 사용. 5+ 전용 write 일부(예: News create)는 일단 구현하되 서버 거부 시 HTTP 오류 그대로 표면화.
- 한국어 출력 메시지/i18n 추가 없음.

## 아키텍처

기존 4점 패턴 그대로 따른다.

| 레이어 | 위치 | 추가 내용 |
| --- | --- | --- |
| 응답/리소스 모델 | `src/types.rs` | 리소스 struct + `*Response` 래퍼 |
| HTTP 호출 | `src/client.rs` | 엔드포인트별 메서드 |
| CLI 핸들러 | `src/cli/<name>.rs` | `Args`/`Command` enum + `handle()` |
| 디스패치 | `src/cli/mod.rs` | `Command` variant 등록 |
| 통합 테스트 | `tests/integration.rs` | wiremock 기반 happy path |
| 파싱 테스트 | `tests/cli_parse.rs` | clap derive parsing |

각 모듈은 단일 파일에 격리. 한 파일이 300줄을 넘으면 분할 검토.

## CLI 인터페이스 컨벤션

- write 가 있는 리소스는 **단수형** + `#[command(subcommand)]`. `list`/`show`/`create`/`update`/`delete` 서브커맨드 노출.
- list만 있는 enum/메타 리소스는 **복수형** 단일 커맨드 (기존 `activities`/`statuses`/`trackers` 답습).
- 모든 출력은 JSON. `output::print_json`.
- 오류: HTTP 응답 그대로 stderr + exit 1.

## 14개 기능 명세

### Stage A — 단순 enum/list (4개)

| # | 커맨드 | HTTP | 권한 |
|---|---|---|---|
| A1 | `redmine roles` | `GET /roles.json` | admin |
| A2 | `redmine document-categories` | `GET /enumerations/document_categories.json` | 일반 |
| A3 | `redmine custom-fields` | `GET /custom_fields.json` | admin |
| A4 | `redmine search <query> [--scope <s>] [--limit <n>] [--offset <n>]` | `GET /search.json` | 일반 |

### Stage B — 프로젝트 하위 리소스 (7개)

| # | 커맨드 | HTTP | 비고 |
|---|---|---|---|
| B1 | `redmine version [list \| show \| create \| update \| delete]` | `/projects/:p/versions.json`, `/versions/:id.json` | list/show는 GET, create는 POST(project scope), update/delete는 PUT/DELETE(id scope) |
| B2 | `redmine membership [list \| show \| add \| update \| remove]` | `/projects/:p/memberships.json`, `/memberships/:id.json` | add: POST project scope. role_ids 필수 |
| B3 | `redmine news [list \| show \| create]` | `/projects/:p/news.json`, `/news.json`, `/news/:id.json` | create는 5+ 전용. 동작 안 하면 HTTP 오류 표면화 |
| B4 | `redmine file [list \| upload]` | `/projects/:p/files.json` | upload는 토큰 받아 파일 등록 패턴(기존 attachments 답습) |
| B5 | `redmine query` | `GET /queries.json` | API 자체가 read-only |
| B6 | `redmine wiki [list \| show \| create \| update \| delete]` | `/projects/:p/wiki/index.json`, `/projects/:p/wiki/:title.json` | create/update 둘 다 PUT (Redmine 관례) |
| B7 | `redmine group [list \| show \| create \| update \| delete \| add-user \| remove-user]` | `/groups.json`, `/groups/:id.json`, `/groups/:id/users.json` | admin 권한 |

### Stage C — 사용자 계정 & 이슈 보조 (3개)

| # | 커맨드 | HTTP | 비고 |
|---|---|---|---|
| C1 | `redmine my-account [show \| update]` | `/my/account.json` GET/PUT | self user 조회/수정 |
| C2 | `redmine issue watcher [add \| remove] <issue> <user>` | `/issues/:id/watchers.json` POST, `/issues/:id/watchers/:uid.json` DELETE | |
| C3 | `redmine issue note <issue> --message <text> [--private]` | `PUT /issues/:id.json` | 기존 `update_issue` 재사용, notes 전용 진입점만 추가 |

## 테스트 전략

각 새 모듈 기준:

- **integration test (wiremock)**: read happy path 1개, write 가 있으면 create happy path 1개. Wiki/Memberships는 404/403 케이스 1개 추가.
- **cli_parse test**: 각 서브커맨드의 옵션 파싱 검증 1~2개.

총 추가 추정: integration ~20개, parse ~15개.

## 커밋 / 배포 흐름

- 각 기능 = 1 커밋 (~14개 기능 커밋). 추가로 CHANGELOG 갱신과 릴리스 커밋이 따라옴.
- 매 커밋 직전 `cargo build && cargo test` 통과 + self 코드 리뷰. 실패 시에만 사용자 보고.
- 14개 완료 후 CHANGELOG 일괄 갱신 → `chore: release v0.2.0` → 태그 → 푸시 → release.yml → `.claude/skills/release/scripts/update-formula.sh 0.2.0` 으로 tap 갱신.

## 위험 / 미해결

- **서버 버전 호환**: 4.2 공통분을 기준으로 함. 5+ 전용 동작은 호출 시도 후 HTTP 오류 노출.
- **권한**: roles/custom-fields/groups 는 admin 필요. 일반 사용자 호출 시 403 → 그대로 표시.
- **`issue note` 중복성**: 기존 `issue update --notes` 와 기능 중복이지만 편의 진입점으로 유지.
- **Queries write 불가**: REST API 한계. read-only 커맨드만 노출.
- **File upload 본문 컨벤션**: `POST /projects/:p/files.json` 은 `{file: {token, ...}}` 형태로 uploads.json 토큰을 참조. 기존 issue attach 패턴과 유사.
