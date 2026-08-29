# config.yml 다중 서버 전환 체크리스트

설계: `docs/superpowers/specs/2026-08-29-multi-server-config-design.md`

## 의존성
- [ ] `serde_norway` 추가, `toml` 은 legacy 읽기용으로 feature 축소
- [ ] `cargo audit` 통과 확인

## src/config.rs
- [ ] `FileConfig` / `ServerConfig` 새 스키마 (BTreeMap, 토큰 redact Debug)
- [ ] 서버 선택 규칙 4갈래 + 에러 타입
- [ ] flag override / ad-hoc 경로
- [ ] YAML 로드·저장 (0600 유지)
- [ ] config.toml → config.yml 1회 자동 변환
- [ ] `parse_custom_field` 인자 BTreeMap 으로 변경
- [ ] 단위 테스트 (선택 규칙, override, 변환, alias 분리)

## CLI
- [ ] `--server <name>` global flag 추가, env 언급 제거 (`src/cli/mod.rs`)
- [ ] `config server list` (토큰 미출력) / `config server use`
- [ ] `config alias *` 를 선택된 서버 기준으로 동작
- [ ] `src/cli/issues.rs` 호출부 타입 정리

## 테스트
- [ ] `tests/integration.rs` env 24곳 → `--server-url`/`--api-token`
- [ ] 다중 서버 round trip 테스트
- [ ] `server list` 토큰 노출 없음 테스트
- [ ] config.yml 0600 테스트

## 문서
- [ ] README Configure 절 재작성
- [ ] CHANGELOG [Unreleased] breaking change
- [ ] context notes 갱신

## 검증
- [ ] `cargo fmt --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo audit`
