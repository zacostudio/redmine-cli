# 다중 Redmine 서버 설정 (config.yml) 설계

작성일: 2026-08-29
상태: 승인됨 (구현 대기)

## 배경

현재 자격증명 해석은 `src/config.rs:69` 의 `resolve()` 하나로, `--server-url`/`--api-token`
flag > `REDMINE_URL`/`REDMINE_API_TOKEN` env > `config.toml` 순으로 머지한다. 파일 스키마는
단일 서버(`server_url`, `api_token`, `custom_fields`)다.

동기는 **회사 Redmine 과 개인 Redmine 을 구분해서 쓰는 것**이다. env 는 서버가 하나라는 전제를
셸 세션에 박아 넣기 때문에 두 서버를 오가는 데 맞지 않는다. env 를 걷어내고 설정을 파일 하나로
모은 뒤, tome 앱(`apps/tome/.../config_db/redmine_servers.rs`)처럼 이름 붙은 서버 여러 개를
두고 골라 쓰게 한다.

## 범위

- `REDMINE_URL` / `REDMINE_API_TOKEN` 읽기 삭제
- 설정 파일을 `config.toml`(TOML) 에서 `config.yml`(YAML) 로 전환
- 이름 붙은 서버 여러 개 + 기본 서버 지정
- `--server <name>` global flag
- custom field alias 를 서버별로 분리
- `config server list` / `config server use` 추가, `config alias *` 를 서버 인식하도록 변경

범위 밖: 서버 CRUD 명령(`server add/remove`), YAML 주석 보존, 토큰의 keychain 보관,
`config.toml` 자동 변환.

## 1. 파일 포맷

경로는 `--config` 로 override 가능하고, 기본값은 `directories` 의 config dir 아래 `config.yml`
이다(macOS `~/Library/Application Support/redmine-cli/`, Linux `~/.config/redmine-cli/`).
저장 권한은 현행대로 Unix 0600 을 유지한다(`src/config.rs:118`).

```yaml
default_server: company
servers:
  company:
    url: https://redmine.example.com
    api_token: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    custom_fields:
      state: 7
      qa: 8
  personal:
    url: https://redmine.home.net
    api_token: yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy
```

Rust 표현:

```rust
pub struct FileConfig {
    pub default_server: Option<String>,
    pub servers: BTreeMap<String, ServerConfig>,   // 이름이 key = 중복 불가
}

pub struct ServerConfig {
    pub url: String,              // serde(default) — 변환 손실 없이 빈 값 허용
    pub api_token: String,        // serde(default)
    pub custom_fields: BTreeMap<String, u64>,
}
```

`BTreeMap` 을 쓰는 이유는 저장 시 출력 순서가 고정되기 때문이다. `HashMap` 이면 `alias set` 을
할 때마다 파일 전체의 줄 순서가 흔들려 diff 가 무의미해진다. `Config.cf_aliases` 와
`parse_custom_field()` 의 인자 타입도 함께 `BTreeMap` 으로 바꾼다(호출부는 `src/cli/issues.rs:78`,
`src/cli/issues.rs:359` 두 곳).

YAML 크레이트는 `serde_norway` 를 쓴다. `serde_yaml` 0.9.34 는 아카이브된 unmaintained 크레이트라
`.github/workflows/audit.yml` 의 `rustsec/audit-check` 에 걸릴 위험이 있다. 추가 후 `cargo audit`
로 실제 확인한다.

`toml` 의존성은 제거한다.

## 2. 서버 선택 규칙

`--server <name>` 은 global flag 라 서브커맨드 앞뒤 어디에나 올 수 있다.

1. `--server <name>` 이 있으면 그 이름으로 선택. 없는 이름이면 에러(설정된 이름 목록을 함께 출력).
2. 없으면 `default_server` 로 선택. `default_server` 가 가리키는 항목이 없으면 에러.
3. `default_server` 도 없고 서버가 **정확히 하나**면 그것을 쓴다.
4. 그 외(서버 0개, 또는 기본값 없이 2개 이상)는 에러.

tome 의 `resolve_server_with_conn` 과 같은 규칙이되, CLI 에는 DB 귀속이 없으므로 id 없이 이름만 쓴다.

## 3. flag override 와 설정 없는 경로

`--server-url` / `--api-token` 은 남긴다. 일회성 호출과 통합 테스트의 자격증명 주입 경로다.

- `--server` 가 있으면: 그 서버를 고른 뒤 주어진 flag 로 해당 필드만 덮어쓴다.
- `--server` 가 없고 `--server-url` 과 `--api-token` **둘 다** 있으면: 서버 선택을 건너뛴다
  (ad-hoc 서버, `cf_aliases` 는 비어 있음). 설정 파일이 있어도 이 경로가 우선이라 테스트가
  사용자 홈의 설정에 영향받지 않는다.
- 그 외에는 2절의 선택 규칙을 따르고 flag 로 필드를 덮어쓴다.

선택된 값이 빈 문자열이면 지금과 같이 `MissingServer` / `MissingToken` 으로 처리한다. 에러 문구에서
env 언급을 걷어낸다(`src/config.rs:58-61`).

## 4. custom field alias 는 서버별

`Config.cf_aliases` 는 선택된 서버의 `custom_fields` 에서만 온다. 회사 서버의 `state=7` 이 개인
서버 호출로 새지 않는다. 서버에 해당 alias 가 없으면 지금과 같은 `unknown custom field alias` 에러다.

## 5. config 서브커맨드

```
redmine config server list                      # 이름/URL/기본 여부/alias — 토큰은 출력 안 함
redmine config server use <name>                # default_server 갱신
redmine config alias list                       # 선택된 서버의 alias
redmine --server company config alias set state 7
redmine --server company config alias remove state
```

- `server list` 출력에 **api_token 을 절대 포함하지 않는다**. 통합 테스트로 고정한다.
- `alias set/remove` 는 대상 서버가 이미 존재해야 한다. 서버가 하나도 없으면 에러이며, 자동으로
  서버를 만들지 않는다.
- `server use <name>` 은 없는 이름이면 에러.

트레이드오프: 쓰기 명령은 파일을 통째로 다시 쓴다. 손으로 넣은 YAML **주석은 지워진다.** 주석 보존은
별도 파서가 필요해 이번 범위에서 제외한다. README 에 명시한다.

## 6. config.toml 은 지원하지 않는다

설정 파일은 `config.yml` 하나다. 예전 `config.toml` 은 읽지 않고, 자동 변환도 하지 않는다.
`toml` 의존성 자체를 제거한다.

> 최초 설계에서는 `config.toml` 을 1회 자동 변환하기로 했으나, 2026-08-29 사용자 요청으로
> "config.yml 만 사용" 으로 바꿨다. 설정 경로가 하나뿐이어야 동작을 설명하기 쉽고, 변환 코드는
> 한 번 쓰이고 영영 남는 종류의 코드다.

## 7. 에러

```rust
enum ConfigError {
    MissingServer,                       // 문구에서 env 제거
    MissingToken,                        // 문구에서 env 제거
    ServerNotFound { name: String, available: Vec<String> },
    NoServerSelected { available: Vec<String> },   // 기본값 없이 2개 이상
    NoServerConfigured,                             // 서버 0개
    Io(PathBuf, String),
    Parse(PathBuf, String),
}
```

`ServerConfig` 와 `FileConfig` 의 `Debug` 는 현행 `FileConfig`/`Config` 처럼 토큰을 `<REDACTED>`
로 가린다(`src/config.rs:14`).

## 8. 테스트

단위(`src/config.rs`):
- 선택 규칙 4갈래 — 명시 이름 / `default_server` / 단일 서버 폴백 / 모호할 때 에러
- 없는 서버 이름 에러에 사용 가능한 이름이 담기는지
- flag override 가 선택된 서버의 필드만 덮어쓰는지
- `--server-url` + `--api-token` 동시 지정 시 서버 선택을 건너뛰는지
- 서버별 alias 분리

통합(`tests/integration.rs`):
- env 를 쓰는 24곳을 `--server-url` / `--api-token` 으로 치환
- config.yml 다중 서버 round trip: `server list` → `server use` → `--server` 로 다른 서버 호출
- `server list` 출력에 토큰 문자열이 없는지
- 저장된 `config.yml` 이 0600 인지 (기존 검증을 yml 대상으로 유지)

## 9. 문서

- README `Configure` 절 재작성 — env 삭제, 다중 서버 예시, 주석이 지워진다는 주의
- CHANGELOG `[Unreleased]` 에 breaking change 기재
- `docs/superpowers/notes/` 에 checklist 와 context notes
