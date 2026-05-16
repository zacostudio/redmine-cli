# Redmine CLI 설계 문서

- 작성일: 2026-05-16
- 작성자: jhyoung75@gmail.com (with Claude)
- 위치: `/Volumes/Projects/zacostudio/cli/issues`
- 참고 코드: `/Volumes/Projects/zacostudio/apps/tome` (Tauri 앱)

---

## 1. 개요 및 목적

`tome` 의 `src-tauri/src/services/redmine` 와 `src-tauri/src/cli/redmine.rs` 로직을 떼어 내어 Tauri 의존성 없이 동작하는 독립 Rust CLI 를 만든다. 결과물은 단일 바이너리 `redmine` 이며, `cargo build --release` 만으로 빌드 가능해야 한다. 출력은 tome 과 동일하게 JSON 전용으로 유지하여 기존 스크립트·파이프라인 호환성을 확보한다.

### 비목표

- Workflow, Docker, Google Sheets, AI 등 tome 의 다른 서비스 영역.
- 로컬 SQLite 캐시 및 `sync` 명령.
- 특정 Redmine 인스턴스에 종속된 `--cf-state` / `--cf-qa` 등 하드코딩 커스텀 필드 플래그. 범용 `--custom-field id=value` 로 대체한다.

### 패키지 / 컴파일러 정책

- 모든 의존성은 작성 시점 기준 최신 안정 버전으로 고정한다. 후보: `clap = "4.5"`, `reqwest = "0.12"`, `serde = "1"`, `serde_json = "1"`, `toml = "0.9"`, `directories = "6"`, `anyhow = "1"`, `urlencoding = "2"`, `libc = "0.2"`.
- `rust-toolchain.toml` 로 stable 채널 (작성 시점 최신, 예: 1.83 이상) 을 고정한다.

---

## 2. 아키텍처

```
issues/
├── Cargo.toml                  단일 crate, bin = "redmine"
├── rust-toolchain.toml         stable 채널 고정
├── README.md
└── src/
    ├── main.rs                 clap 파싱 → dispatch → 종료 코드
    ├── config.rs               env + ~/.config/redmine-cli/config.toml 로딩
    ├── output.rs               print_json / print_error
    ├── client.rs               RedmineClient (tome client.rs 이식)
    ├── types.rs                API 응답 타입 (tome types.rs 이식)
    └── cli/
        ├── mod.rs              Cli enum (clap Subcommand)
        ├── projects.rs
        ├── categories.rs
        ├── issues.rs           list/get/create/update/delete/relations
        ├── time_entries.rs
        ├── users.rs
        ├── activities.rs
        ├── enums.rs            statuses/trackers/priorities (API 직행)
        └── attachments.rs      list/download/upload/delete
```

### 핵심 의존성

- `clap = { version = "4", features = ["derive"] }`
- `reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }`
- `serde`, `serde_json`
- `toml`
- `directories`
- `anyhow`
- `libc`
- `urlencoding`

### 이식 변경점

- `RedmineClient::new` 시그니처와 메서드는 그대로 유지하여 재테스트 비용을 최소화한다.
- tome 의 `handle_*` 함수는 clap `Args` 구조체를 받도록 리팩터한다. 구조체 필드가 곧 플래그가 된다.
- `print_error` 내 `libc::_exit(1)` 는 그대로 유지한다 (tome 의도 보존).
- tome 의 로컬 store 의존부 (statuses/trackers/priorities/sync) 는 모두 Redmine API 직행으로 단순화한다.

---

## 3. 명령 구조 (clap derive)

```
redmine projects [--limit N] [--offset N]
redmine categories --project <id>
redmine issues [--project <id>] [--status <id>] [--query <text>]
              [--assigned-to <id>] [--tracker <id>] [--priority <id>]
              [--limit N] [--offset N] [--sort <field>]
              [--custom-field <id>=<value>]...
redmine issue <id>                                  # 단일 조회
redmine issue create --project <id> --subject <s>
                     [--description ...] [--tracker N] [--priority N]
                     [--assigned-to N] [--category N] [--parent N]
                     [--start-date YYYY-MM-DD] [--due-date YYYY-MM-DD]
                     [--estimated-hours F] [--done-ratio N]
                     [--target-version N]
                     [--custom-field <id>=<value>]...
redmine issue update <id> [...same fields as create...]
                          [--status N] [--notes ...] [--private-notes]
                          [--custom-field <id>=<value>]...
redmine issue delete <id>
redmine issue relations <id>
redmine issue add-relation <id> --to <id> [--type relates|blocks|...]
redmine issue remove-relation <relation-id>

redmine time-entry create --issue <id> --hours F
                          [--activity N] [--spent-on YYYY-MM-DD] [--comment ...]
redmine time-entry list [--user <id>] [--project <id>] [--issue <id>]
                        [--from YYYY-MM-DD] [--to YYYY-MM-DD] [--limit N]
redmine time-entry update <id> [--hours F] [--activity N] [--comment ...] [--spent-on ...]
redmine time-entry delete <id>

redmine users --name <q> [--limit N]
redmine activities
redmine statuses
redmine trackers
redmine priorities

redmine attachment list --issue <id>
redmine attachment download <id> --output <path>
redmine attachment upload --issue <id> --file <path> [--description ...]
redmine attachment delete <id>
```

### tome 대비 변경

- `redmine issue <id>` 단일 조회는 위치 인자 `<id>` 사용 (tome 은 `--id` 플래그).
- `time-entry` 를 `create/list/update/delete` 서브커맨드로 일관화 (tome 의 `time-entry`/`time-entries`/`time-entry-update`/`time-entry-delete` 4개를 1개로 통합).
- `--cf-state` 등 10개 고정 플래그 제거 → `--custom-field 7=Dev` 반복으로 통일.
- `statuses`/`trackers`/`priorities` 는 로컬 store 대신 Redmine API 직행.
- `sync` 명령 제거.

---

## 4. 설정 로딩 + 데이터 흐름

### 우선순위 (높음 → 낮음)

1. CLI 플래그 `--server-url`, `--api-token` (글로벌 옵션).
2. 환경변수 `REDMINE_URL`, `REDMINE_API_TOKEN`.
3. config 파일 `$XDG_CONFIG_HOME/redmine-cli/config.toml` (없으면 `~/.config/redmine-cli/config.toml`).

### config.toml 형식

```toml
server_url = "https://redmine.example.com"
api_token  = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# 선택. 사람이 읽는 이름 → custom field id 별칭.
[custom_fields]
state         = 7
qa            = 8
code_review   = 9
```

별칭은 `--custom-field state=Dev` 처럼 쓸 수 있는 편의 기능이다. 별칭이 없으면 그대로 정수로 파싱한다.

### 데이터 흐름

```
argv
 └─► clap derive 파싱 → Cli { global: Global, command: Command }
        ├─ Global { server_url?, api_token?, config? }
        └─ Command::Issues(IssuesArgs) | Command::Issue { id, sub } | ...

resolve_config(global) → Config { server_url, api_token, cf_aliases }
        ├─ flag > env > file 순으로 머지
        └─ 미설정 시 print_error 로 종료

RedmineClient::new(&cfg.server_url, &cfg.api_token)

dispatch(command, &client, &cfg) → ()
        ├─ 각 핸들러는 args → query/payload 변환
        ├─ client.{method} → 결과
        └─ output::print_json(...)
```

### 에러 처리

- 모든 실패 경로는 tome 그대로 `print_error("...")` → `libc::_exit(1)` 로 처리한다 (stdout flush 회피).
- 정상 경로는 `print_json(...)` 후 0 으로 종료한다.
- panic 은 자연스럽게 비정상 종료한다. 별도 hook 은 두지 않는다.

---

## 5. 테스트 · 빌드/배포 · 최적화

### 5.1 테스트

- **단위 테스트.** `client.rs` 는 reqwest 호출이라 mock 이 필요하다. `wiremock = "0.6"` 으로 가짜 서버를 띄우거나, 직렬화/역직렬화 + URL/쿼리 생성 로직만 분리해 테스트한다.
- **CLI 파싱 테스트.** clap 의 `try_parse_from` 으로 각 서브커맨드의 인자 해석을 검증한다. 외부 의존성 없이 빠르므로 우선순위 높다.
- **통합 테스트.** `assert_cmd = "2"` + `wiremock` 으로 가짜 Redmine 서버에 실제 바이너리를 호출한다. golden path 1~2개만 (issues 검색, issue 조회).
- **수동 smoke.** 실제 Redmine 인스턴스 (`.env` 로 URL/토큰 주입) 에 대해 `redmine projects`, `redmine issues --limit 1`, `redmine issue <id>` 3종을 확인한다.

### 5.2 빌드 / 배포 (Homebrew)

#### A. 릴리스 바이너리 빌드

GitHub Actions 로 태그 푸시 (`v*.*.*`) 시 다음 타깃을 cross-compile 한다.

- `aarch64-apple-darwin` (Apple Silicon)
- `x86_64-apple-darwin` (Intel Mac)
- `x86_64-unknown-linux-gnu` (선택)

각 타깃 결과물을 `redmine-<version>-<target>.tar.gz` 로 압축하고 SHA256 함께 GitHub Release 에 업로드한다.

#### B. Homebrew tap 저장소

별도 GitHub 저장소 `zacostudio/homebrew-redmine` 을 만든다 (이름 규칙 `homebrew-<tap>` 필수). 구조.

```
homebrew-redmine/
└── Formula/
    └── redmine.rb
```

사용자 설치 흐름.

```
brew tap zacostudio/redmine
brew install redmine
```

#### C. 바이너리 기반 Formula 예시

```ruby
class Redmine < Formula
  desc "Standalone CLI for Redmine"
  homepage "https://github.com/zacostudio/redmine-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/zacostudio/redmine-cli/releases/download/v#{version}/redmine-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
    on_intel do
      url "https://github.com/zacostudio/redmine-cli/releases/download/v#{version}/redmine-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  def install
    bin.install "redmine"
  end

  test do
    assert_match "redmine", shell_output("#{bin}/redmine --version")
  end
end
```

#### D. 소스 기반 fallback Formula (선택)

```ruby
depends_on "rust" => :build

def install
  system "cargo", "install", *std_cargo_args
end
```

#### E. 릴리스 자동화

GitHub Actions `release.yml` 에서.

1. 태그 푸시 감지 → 매트릭스 빌드.
2. tarball + sha256 업로드.
3. `homebrew-redmine` 저장소에 PR/커밋 (sha256 자동 치환).

자동화 도구 후보: `dawidd6/action-homebrew-bump-formula`, 또는 자체 스크립트.

#### F. homebrew-core 등록 검토

core 는 notability 기준 (스타 30, 30일 이상, 의존성 안정성) 을 통과해야 하고 PR 리뷰가 길어서 사내용·초기엔 비추한다. tap 부터 시작하고 사용량이 쌓이면 core 이주를 검토한다.

### 5.3 최적화 단계 (구현 완료 후 별도 작업)

기능이 동작하는 v0.1 빌드를 먼저 끝낸 뒤, 다음 순서로 진행한다.

1. **측정 베이스라인 확보.** `hyperfine` 으로 대표 시나리오 5종 시간 측정. `cargo bloat --release --crates` 로 바이너리 크기·의존성 분포 확인.
2. **컴파일 옵션 튜닝.** `Cargo.toml` `[profile.release]` 에 `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"` 적용 → 바이너리 크기·시작 시간 단축. 빌드 시간은 늘어난다.
3. **의존성 다이어트.** reqwest 의 default-features 끄고 필요한 기능만 (`rustls-tls`, `json`, `blocking`). serde 의 derive feature 만. toml 의 parse-only feature 검토.
4. **할당 줄이기.** 핫 패스에서 불필요한 `to_string()`/`clone()` 제거. 핸들러 내부 `format!` → `write!` 가능한 곳 점검.
5. **응답 파싱 비용.** issues 검색이 큰 페이지면 `serde_json::from_reader` 로 스트리밍 검토. 측정해서 의미 있으면 적용, 없으면 패스.
6. **panic 경로 점검.** `unwrap()`/`expect()` 인벤토리 후 사용자 입력 영향 받는 곳은 `print_error` 로 교체.

각 단계마다 baseline 대비 측정값을 기록한다. 측정 없는 추측 최적화는 금지한다.

---

## 6. 실행 순서

1. v0.1 구현 (Task #5).
2. 테스트 작성 (Task #6).
3. 측정 기반 최적화 (Task #7).
4. GitHub repo 생성 + 릴리스 자동화 (Task #8).
5. Homebrew tap 저장소 설정 (Task #9).

각 단계마다 `checklist.md` 와 `context-notes.md` 를 갱신한다 (CLAUDE.md §7 준수).

---

## 7. 참고

- tome 원본: `/Volumes/Projects/zacostudio/apps/tome/src-tauri/src/services/redmine/`, `cli/redmine.rs`.
- Homebrew 배포 참고: <https://kawamurakazushi.com/20200217-publishing-a-rust-cli-to-homebrew/>, <https://ivaniscoding.github.io/posts/rustpackaging2/>.
- Redmine REST API: <https://www.redmine.org/projects/redmine/wiki/Rest_api>.
