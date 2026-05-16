# CHANGELOG.md 형식

Keep a Changelog 1.1.0 형식 + SemVer 기준.

## 신규 생성 템플릿

`CHANGELOG.md`가 없는 경우 다음을 초기 파일로 작성한 뒤 기존 태그 내역을 채워 넣는다.

```markdown
# Changelog

이 프로젝트의 변경 사항을 기록합니다.

본 문서는 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/) 형식을 따르며,
버전은 [Semantic Versioning](https://semver.org/lang/ko/)을 따릅니다.

## [Unreleased]

## [0.1.1] - 2026-05-16
### Added
- (기존 v0.1.1 변경 사항을 git log v0.1.0..v0.1.1 로 채움)

## [0.1.0] - 2026-05-16
### Added
- 초기 릴리스.

[Unreleased]: https://github.com/zacostudio/redmine-cli/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/zacostudio/redmine-cli/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zacostudio/redmine-cli/releases/tag/v0.1.0
```

날짜는 실제 태그 일자 기준. 알 수 없으면 `gh release view vX.Y.Z --json publishedAt --jq .publishedAt` 로 조회.

## 새 버전 추가

```markdown
## [Unreleased]

## [X.Y.Z] - YYYY-MM-DD
### Added
- ...
### Changed
- ...
### Fixed
- ...
### Performance
- ...
### Removed
- ...
```

비어 있는 섹션은 출력하지 말 것. 비교 링크도 함께 갱신:

```
[Unreleased]: https://github.com/zacostudio/redmine-cli/compare/vX.Y.Z...HEAD
[X.Y.Z]: https://github.com/zacostudio/redmine-cli/compare/vPREV...vX.Y.Z
```

## Conventional Commit → CHANGELOG 섹션 매핑

| 접두어 | 섹션 |
| --- | --- |
| `feat:` | Added |
| `fix:` | Fixed |
| `perf:` | Performance |
| `refactor:` | Changed |
| `docs:` | (CHANGELOG에서 보통 생략, 사용자 영향이 있으면 Changed) |
| `test:` | (생략) |
| `ci:` | (생략, 릴리스 워크플로우 변경처럼 사용자 영향이 있으면 Changed) |
| `chore:` | (생략, deps 메이저 bump처럼 영향 있으면 Changed) |
| `feat!:` / `BREAKING CHANGE:` | Changed + 명시적 `### Breaking` 추가 |

## 항목 작성 규칙

- 한국어, 사용자 관점으로 작성. 내부 구현 디테일은 생략.
  - 나쁨: `RedmineClient에서 Box::leak 제거`
  - 좋음: `issues 검색 메모리 사용량 감소`
- 한 줄 끝은 마침표. 사용자 응답이 한국어인 점과 일관성 유지.
- 가능하면 짧은 commit 해시(`(abc1234)`)나 PR 번호를 끝에 부기.
- 동일 섹션 내 다중 항목은 영향 큰 순서로 정렬.

## 예시: v0.1.2 항목

`git log v0.1.1..HEAD --oneline` 출력이 다음과 같다고 가정:

```
b62c90c feat(issue create): add --id-only flag for safe shell scripting
```

대응 CHANGELOG 항목:

```markdown
## [0.1.2] - 2026-05-16
### Added
- `issue create` 명령에 `--id-only` 플래그 추가. 셸 스크립트에서 새 이슈 ID만 추출하기 쉬워졌습니다. (b62c90c)
```
