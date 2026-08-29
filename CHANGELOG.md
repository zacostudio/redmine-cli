# Changelog

이 프로젝트의 변경 사항을 기록합니다.

본 문서는 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/) 형식을 따르며,
버전은 [Semantic Versioning](https://semver.org/lang/ko/)을 따릅니다.

## [Unreleased]
### Added
- Redmine 서버를 여러 개 설정하고 `--server <name>` 으로 골라 쓸 수 있다. 회사 Redmine 과 개인 Redmine 을 이름으로 구분한다. (57e7186)
- `config server list` — 설정된 서버 목록. API 토큰은 출력하지 않는다. (57e7186)
- `config server use <name>` — `--server` 없이 쓸 기본 서버 지정. (57e7186)

### Changed
- **BREAKING**: 설정 파일이 `config.toml` 에서 `config.yml` 로 바뀌었다. `config.toml` 은 읽지 않으며 자동 변환도 하지 않는다. `config.yml` 을 새로 작성해야 한다. (57e7186, b51c4a7)
- **BREAKING**: `REDMINE_URL` / `REDMINE_API_TOKEN` 환경 변수를 더 이상 읽지 않는다. 자격증명은 `config.yml` 또는 `--server-url` / `--api-token` 으로만 지정한다. (57e7186)
- custom field alias 가 서버별로 분리됐다. `config alias set/remove/list` 는 선택된 서버(`--server` 또는 `default_server`)에만 적용된다. (57e7186)
- `--server-url` / `--api-token` 은 선택된 서버의 해당 필드를 덮어쓴다. `--server` 없이 둘 다 주면 설정 파일과 무관한 ad-hoc 호출이 된다. (57e7186)
- `toml` 의존성 제거. 설정 파일 파싱은 `serde_norway`(YAML) 하나로 끝난다. (b51c4a7)
- `config::parse_custom_field` 의 alias 인자가 `HashMap` 에서 `BTreeMap` 으로 바뀌었다. 저장 시 줄 순서를 고정해 diff 를 읽을 수 있게 하기 위해서다. **라이브러리 사용자 영향.** (57e7186)

### Fixed
- CI 워크플로의 push 트리거가 `main` 을 보고 있어 기본 브랜치 `master` 푸시에 CI 가 돌지 않던 문제. (af41a0a)

## [0.3.0] - 2026-05-16
### Added
- 모든 리소스 생성 명령(`issue create`, `time-entry create`, `version create`, `membership add`, `news create`, `group create`)에 `--id-only` 플래그 확장. `attachment upload` 에는 `--token-only` 추가. (4b0eba7)

### Changed
- `issue note` 플래그를 `--private-notes` 로 통일. 기존 `--private` 은 alias 로 유지되어 호환된다. (936a0a1)
- `client::get_issue(id, include)` 시그니처 변경. `attachment list` 가 더 이상 journals/children/relations 까지 끌어오지 않는다. **라이브러리 사용자 영향.** (936a0a1)
- `client::update_issue` 가 PUT 만 수행하도록 변경. `issue update` CLI 출력은 호출자가 후속 GET 으로 유지하지만, `issue note` 는 한 번의 PUT 으로 완료되어 RTT 가 절반으로 감소. (936a0a1)
- 클라이언트 에러 타입이 단순 `String` 에서 `ClientError` enum 으로 교체됨. 4xx/5xx 는 `ClientError::Http { status, body }` 로 구조화. Display 텍스트는 호환 유지. **라이브러리 사용자 영향.** (936a0a1)

### Performance
- 첨부 파일 업로드/다운로드를 스트리밍으로 변경. 파일 전체를 메모리에 적재하던 동작을 제거해 대용량 첨부에서 OOM 위험 없음. (936a0a1)
- 사용하지 않던 `anyhow` 의존성 제거. 빌드 시간 단축. (4fa2a9a)

### Security
- HTTP 리다이렉트를 전체 차단 (`reqwest::redirect::Policy::none`). 잘못 설정된 `server_url` 이나 중간자 공격이 발생해도 `X-Redmine-API-Key` 가 외부 호스트로 따라가지 않는다. (936a0a1)
- `config.toml` 을 Unix 에서 0600 권한으로 저장. 평문 API 토큰을 동일 호스트의 다른 사용자가 읽을 수 없도록 `OpenOptions::mode` 와 `set_permissions` 로 강등. (4fa2a9a)
- `Cli`, `Config`, `FileConfig`, `CliOverrides` 의 `Debug` 출력에서 `api_token` 을 `<REDACTED>` 로 마스킹. `eprintln!("{:?}", ...)` 한 줄로 토큰이 stderr 에 새는 잠재 결함을 차단. (cfbe56f)
- 프로젝트 identifier 13곳을 URL path 에 그대로 박던 동작을 `urlencoding::encode` 처리. `--project "foo/../admin"` 같은 입력으로 의도 외 endpoint 가 호출되는 path manipulation 가능성을 막는다. (cfbe56f)
- `attachment download` 가 서버 응답의 `content_url` 을 그대로 GET 하기 전에 host/scheme/port 가 `server_url` 과 일치하는지 검증. 응답 조작이나 손상된 응답이 외부 URL 을 가리켜도 API 키가 외부로 따라가지 않는다. (cfbe56f)
- `cargo audit` 자동화 워크플로 추가. Cargo.toml/lock 변경 PR + 매주 월요일 정기 + 수동 트리거로 RustSec advisory DB 와 대조한다. (dec917b)

## [0.2.0] - 2026-05-16
### Added
- `redmine wiki` 서브커맨드. 위키 페이지 목록·조회·생성·수정·삭제 가능. 본문은 `--text -` 로 stdin 입력 지원. (86d031d)
- `redmine version` 서브커맨드. 프로젝트 버전(마일스톤) CRUD. (23ad52a)
- `redmine membership` 서브커맨드. 프로젝트 멤버 추가·역할 변경·삭제. user/group 모두 지원. (442be56)
- `redmine news` 서브커맨드. 전역/프로젝트별 공지 목록·조회·생성(5+). (8a1da80)
- `redmine file` 서브커맨드. 프로젝트 파일 보관함 목록·업로드(버전 연결 가능). (96ce7d9)
- `redmine query` 명령. 저장된 이슈 쿼리 목록 조회(REST API는 read-only). (9234fc8)
- `redmine group` 서브커맨드. 그룹 CRUD + 사용자 추가/제거(admin 권한). (60e56c0)
- `redmine my-account` 서브커맨드. 현재 사용자 프로필 조회·수정. (f40a431)
- `redmine issue <id> watcher add/remove`. 이슈 워처 관리. (8a97864)
- `redmine issue <id> note --message`. 이슈에 노트(저널) 게시 전용 진입점. `--private` 플래그 지원. (8a97864)
- `redmine roles`. 역할 목록(admin 권한). (034883a)
- `redmine document-categories`. 문서 카테고리 목록. (b1f4479)
- `redmine custom-fields`. 커스텀 필드 정의 메타데이터(admin 권한). (f57103c)
- `redmine search <query>`. 전역 검색. `--scope`, `--all-words`, `--titles-only` 등 옵션. (5d2d18b)

### Changed
- `RedmineUser` 타입에 `admin`, `created_on`, `last_login_on` 필드 추가. 기존 응답과 호환됩니다.

## [0.1.2] - 2026-05-16
### Added
- `issue create` 명령에 `--id-only` 플래그 추가. 셸 스크립트에서 새 이슈 ID만 추출하기 쉬워졌습니다. (b62c90c)

## [0.1.1] - 2026-05-16
### Added
- `redmine config alias` 서브커맨드로 커스텀 필드 별칭을 관리할 수 있습니다. (90ba504)

## [0.1.0] - 2026-05-16
### Added
- 초기 릴리스. Redmine REST API를 감싸는 단일 바이너리 CLI입니다.
- 핵심 리소스 커맨드 제공: `projects`, `issues`, `time-entry`, `users`, `categories`, `attachments`, `activities`, `statuses`, `trackers`, `priorities`.
- 설정 우선순위(플래그 > 환경변수 > TOML)와 커스텀 필드 alias 파서.
- macOS(aarch64/x86_64) 및 Linux(x86_64)용 바이너리 릴리스 워크플로우.

[Unreleased]: https://github.com/zacostudio/redmine-cli/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/zacostudio/redmine-cli/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/zacostudio/redmine-cli/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/zacostudio/redmine-cli/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zacostudio/redmine-cli/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zacostudio/redmine-cli/releases/tag/v0.1.0
