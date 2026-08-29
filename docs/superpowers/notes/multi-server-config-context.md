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

**config.toml 자동 변환은 넣었다가 다시 뺐다.** 처음에는 1회 변환을 구현했지만(57e7186),
같은 날 사용자가 "config.yml 만 사용" 으로 정리했다. 설정 파일이 하나뿐이어야 "어디를 고쳐야 하나"
라는 질문에 답이 하나가 되고, 변환 코드는 한 번 쓰이고 영영 남는 종류의 코드다. `toml` 의존성도
함께 제거했다.

## 2026-08-29 부수 정리

`.github/workflows/ci.yml` 의 push 트리거가 `[main]` 이라 master 푸시에 CI 가 돌지 않고 있었다.
audit.yml 의 `[main, master]` 와 함께 `[master]` 로 통일했다 (af41a0a). 이 저장소의 기본 브랜치는
master 이고 리모트에도 master 만 있다.

## 2026-08-29 구현 중 결정

**`resolve_from` 을 파일 I/O 에서 분리했다.** 선택 규칙이 이 프로젝트에서 가장 헷갈리는 부분인데,
파일과 엮여 있으면 tempdir 없이는 한 갈래도 검증할 수 없다. 지금은 단위 테스트가 `FileConfig` 를
직접 만들어 4갈래를 전부 찌른다.

**에러 문구에 설정 경로와 사용 가능한 서버 이름을 넣었다.** config dir 이 OS 마다 달라서
"설정이 없다"만으로는 사용자가 어디를 고쳐야 할지 모른다. macOS 실제 경로는
`~/Library/Application Support/redmine-cli/config.yml` 이다 (`config server list` 로 확인).

**alias 명령은 서버를 자동 생성하지 않는다.** 예전에는 빈 config.toml 에 `alias set` 하면 파일이
생겼다. 지금은 서버가 없으면 에러다 — 이름 없는 서버에 alias 를 붙이는 건 의미가 없고, 그 상태로
저장하면 사용자는 자기가 무슨 서버를 설정했다고 착각하게 된다.

## 2026-08-29 config.yml 단일화

`config.toml` 지원(자동 변환 포함)을 전부 걷어냈다. 부수 효과로 ad-hoc 경로 테스트의 성격이
바뀌었다 — 예전에는 "flag 만 주면 변환이 일어나지 않는다" 를 검증했는데, 변환이 없어졌으므로
지금은 "**깨진** config.yml 이 있어도 flag 만으로 한 호출은 성공한다" 를 검증한다. 파일을 읽지
않는다는 사실을 증명하려면 읽었을 때 반드시 실패하는 파일이어야 한다.

## 2026-08-29 server add / remove 추가

범위 밖으로 뒀던 서버 CRUD 를 넣었다. env 를 없앤 순간 "첫 서버를 어떻게 넣느냐" 가 손으로 YAML
쓰기밖에 남지 않아서, 설정을 처음 만드는 경로가 CLI 에 없다는 게 문제였다.

**토큰은 `--api-token` global flag 를 재사용한다.** 서브커맨드에 같은 이름의 인자를 다시 정의하면
clap 이 중복으로 죽는다. 덕분에 `redmine --api-token xxx config server add ...` 와
`redmine config server add ... --api-token xxx` 가 둘 다 같은 값으로 파싱된다.

**`--api-token` 이 없으면 stdin 에서 읽는다.** 다만 터미널이 그대로 붙어 있으면 입력을 기다리며
멈추므로, `isatty(0)` 로 파이프 여부를 먼저 보고 아니면 에러를 낸다. 멈춘 CLI 는 사용자에게
"응답 없음" 으로 보인다.

**`--force` 덮어쓰기는 alias 를 살린다.** URL·토큰만 갱신하려는 경우가 대부분인데 alias 까지
날아가면 서버별로 모아둔 cf id 를 다시 넣어야 한다.

## 2026-08-29 코드 리뷰 반영

리뷰가 15건을 보고했고 전부 손으로 재현한 뒤 12건을 고쳤다. 판단이 갈린 3건을 남겨 둔다.

**고치지 않은 것 1 — ad-hoc 경로에서 alias 가 비는 것.** `--server` 없이 자격증명 두 개를 주면
`cf_aliases` 가 빈 채로 간다. 리뷰는 regression 이라고 봤지만, 다른 서버의 alias 를 그대로
적용하면 **엉뚱한 custom field id 로 값이 나간다.** 잘못된 값을 쓰는 것보다 `unknown custom
field alias` 로 멈추는 편이 안전하다. README 에 명시하고 `--server` 를 함께 주는 방법을 적었다.

**2, 3 은 같은 날 사용자 요청으로 다시 열어 처리했다.** (아래 절 참고)

**deny_unknown_fields 를 켠 이유.** 오타 난 키(`api-token`, `custom_field`)가 조용히 무시되면
증상이 원인에서 멀어진다("토큰 없음", "alias 없음"). 게다가 전체 rewrite 방식이라 다음 저장에서
그 키가 사라진다. 파싱 단계에서 키 이름과 줄 번호를 찍어 주는 편이 낫다.

**save 를 임시 파일 + rename 으로 바꾼 이유.** env 폴백을 없앤 뒤로 이 파일 하나가 모든 서버
토큰의 유일한 사본이다. 예전에는 쓰다 말아도 env 로 계속 쓸 수 있었지만 지금은 복구할 곳이 없다.

## 2026-08-29 토큰 파일 입력과 legacy 안내

**`--api-token-file` 을 골랐고 stdin 은 쓰지 않았다.** `issue create` 와 `wiki` 가 이미 본문을
stdin 으로 읽는다(`src/cli/issues.rs:337`, `src/cli/wiki.rs:108`). 토큰까지 stdin 으로 받으면
같은 명령에서 둘이 충돌한다. 파일 경로 방식은 CI 의 secret 마운트와도 그대로 맞는다.
`--api-token` 과는 `conflicts_with` 로 묶어, 어느 쪽이 이겼는지 모르는 상태를 없앴다.

**config.toml 안내는 존재 확인 한 줄이다.** 파일을 열지 않고 `path.with_extension("toml")` 의
존재만 본다. toml 파서도 의존성도 돌아오지 않는다 — 설정 포맷은 config.yml 하나이고, 이 메시지는
오히려 "남은 파일을 하나로 옮기라" 고 말하는 쪽이다. 사용자가 포맷 이원화를 명확히 거부했으므로
문구도 "더 이상 읽지 않는다 + 옮기는 명령" 으로 적었다.
