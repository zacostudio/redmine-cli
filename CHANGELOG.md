# Changelog

이 프로젝트의 변경 사항을 기록합니다.

본 문서는 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/) 형식을 따르며,
버전은 [Semantic Versioning](https://semver.org/lang/ko/)을 따릅니다.

## [Unreleased]

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

[Unreleased]: https://github.com/zacostudio/redmine-cli/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/zacostudio/redmine-cli/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/zacostudio/redmine-cli/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zacostudio/redmine-cli/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zacostudio/redmine-cli/releases/tag/v0.1.0
