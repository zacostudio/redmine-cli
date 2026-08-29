# config.yml 다중 서버 전환 체크리스트

설계: `docs/superpowers/specs/2026-08-29-multi-server-config-design.md`

## 의존성
- [x] `serde_norway` 추가, `toml` 은 legacy 읽기용으로 feature 축소
- [x] `cargo audit` 통과 확인

## src/config.rs
- [x] `FileConfig` / `ServerConfig` 새 스키마 (BTreeMap, 토큰 redact Debug)
- [x] 서버 선택 규칙 4갈래 + 에러 타입
- [x] flag override / ad-hoc 경로
- [x] YAML 로드·저장 (0600 유지)
- [x] config.toml → config.yml 1회 자동 변환
- [x] `parse_custom_field` 인자 BTreeMap 으로 변경
- [x] 단위 테스트 (선택 규칙, override, 변환, alias 분리)

## CLI
- [x] `--server <name>` global flag 추가, env 언급 제거 (`src/cli/mod.rs`)
- [x] `config server list` (토큰 미출력) / `config server use`
- [x] `config alias *` 를 선택된 서버 기준으로 동작
- [x] `src/cli/issues.rs` 호출부 타입 정리

## 테스트
- [x] `tests/integration.rs` env 24곳 → `--server-url`/`--api-token`
- [x] 다중 서버 round trip 테스트
- [x] `server list` 토큰 노출 없음 테스트
- [x] config.yml 0600 테스트

## 문서
- [x] README Configure 절 재작성
- [x] CHANGELOG [Unreleased] breaking change
- [x] context notes 갱신

## 검증
- [x] `cargo fmt --check`
- [x] `cargo clippy -- -D warnings`
- [x] `cargo test`
- [x] `cargo audit`

## 완료 기록 (2026-08-29)

- 구현 커밋: 57e7186
- `cargo test`: lib 20 / cli_parse 36 / integration 29 통과
- `cargo clippy --all-targets -- -D warnings`: 경고 없음
- `cargo audit`: serde_norway / unsafe-libyaml-norway 는 advisory 없음.
  h2(RUSTSEC-2026-0258), quinn-proto(RUSTSEC-2026-0185) 2건은 reqwest/wiremock
  경유의 **기존** 문제로 이번 변경과 무관하다. 별도로 처리한다.
