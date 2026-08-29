# config.yml 다중 서버 전환 — context notes

## 2026-08-29 설계 결정

**왜 하는가.** 회사 Redmine 과 개인 Redmine 을 구분해서 쓰기 위해서다. env 는 "서버는 하나"라는
전제를 셸 세션에 박아 넣어서 두 서버를 오가는 데 맞지 않는다.

**스키마는 list 가 아니라 이름 key 의 map.** tome 은 `RedmineServerConfig` 에 random id 를 두는데,
그건 SQLite 에 저장된 이슈 데이터의 서버 귀속(attribution)이 URL 변경에도 유지돼야 하기 때문이다.
CLI 는 저장하는 상태가 없으므로 id 가 필요 없고, map 이면 이름 중복이 구조적으로 불가능하다.

**BTreeMap 을 쓴 이유는 정렬이 아니라 diff 안정성.** HashMap 이면 `alias set` 한 번에 파일 전체
줄 순서가 흔들려서 사용자가 직접 관리하는 파일의 diff 가 쓸모없어진다.

**`--server-url`/`--api-token` 을 남긴 이유.** 통합 테스트 24개가 자격증명을 주입할 경로가 필요하고,
설정에 없는 서버를 즉석에서 호출하는 용도도 있다. 다만 이 둘이 **동시에** 주어지면 서버 선택 자체를
건너뛴다 — 그래야 사용자 홈의 실제 config.yml 이 테스트 결과를 오염시키지 않는다.

**serde_yaml 대신 serde_norway.** serde_yaml 0.9.34 는 아카이브된 unmaintained 크레이트라
`rustsec/audit-check` 워크플로에 걸릴 위험이 있다.

**주석은 지워진다.** `alias set` / `server use` 는 파일을 통째로 다시 쓴다. 주석 보존은 별도 파서가
필요해서 범위에서 뺐고, README 에 명시하기로 했다.

**config.toml 은 지우지 않는다.** 변환은 1회, `path.with_extension("toml")` 규칙으로만 일어난다.
원본을 남겨야 변환이 잘못돼도 되돌릴 수 있고, 명시 경로(`--config`)에서도 같은 규칙이라 테스트가 쉽다.

## 2026-08-29 부수 정리

`.github/workflows/ci.yml` 의 push 트리거가 `[main]` 이라 master 푸시에 CI 가 돌지 않고 있었다.
audit.yml 의 `[main, master]` 와 함께 `[master]` 로 통일했다 (af41a0a). 이 저장소의 기본 브랜치는
master 이고 리모트에도 master 만 있다.
