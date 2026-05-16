# Changelog

이 프로젝트의 변경 사항을 기록합니다.

본 문서는 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/) 형식을 따르며,
버전은 [Semantic Versioning](https://semver.org/lang/ko/)을 따릅니다.

## [Unreleased]

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

[Unreleased]: https://github.com/zacostudio/redmine-cli/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/zacostudio/redmine-cli/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zacostudio/redmine-cli/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zacostudio/redmine-cli/releases/tag/v0.1.0
