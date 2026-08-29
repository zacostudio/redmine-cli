# redmine-cli

Standalone CLI for Redmine. Ported from the [tome](https://github.com/zacostudio/tome) Tauri app.

## Install

```bash
brew tap zacostudio/redmine
brew install redmine
```

Or from source:

```bash
cargo install --git https://github.com/zacostudio/redmine-cli
```

## Configure

`config.yml` 하나에 Redmine 서버를 여러 개 적어 두고 이름으로 골라 씁니다. 회사 Redmine 과
개인 Redmine 을 같은 CLI 로 오갈 때 쓰는 구조입니다.

경로는 macOS 는 `~/Library/Application Support/redmine-cli/config.yml`,
Linux 는 `~/.config/redmine-cli/config.yml` 입니다. `--config <path>` 로 바꿀 수 있고,
`redmine config server list` 를 실행하면 실제 경로가 출력됩니다.

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

파일에는 평문 토큰이 들어가므로 CLI 가 저장할 때 Unix 에서 0600 으로 만듭니다. 직접 만들 때도
`chmod 600` 을 권장합니다.

### 서버 추가·삭제

```bash
# 토큰을 인자로 주는 방식
redmine config server add company --url https://redmine.example.com --api-token xxxx

# 셸 히스토리에 토큰을 남기지 않으려면 stdin 으로
pbpaste | redmine config server add personal --url https://redmine.home.net

redmine config server remove personal
```

첫 서버는 자동으로 `default_server` 가 됩니다. 같은 이름이 이미 있으면 에러이고, `--force` 를
주면 덮어씁니다(이 경우 그 서버의 alias 는 유지됩니다). 기본 서버를 삭제하면 `default_server`
도 함께 비워집니다.

### 서버 고르기

```bash
redmine issues                          # default_server (company)
redmine --server personal issues        # 이름으로 지정
redmine config server list              # 설정된 서버 목록 (토큰은 출력되지 않습니다)
redmine config server use personal      # 기본 서버 변경
```

`--server` 를 생략하면 `default_server` 를, 그것도 없고 서버가 하나뿐이면 그 서버를 씁니다.
서버가 둘 이상인데 기본값이 없으면 이름 목록과 함께 에러가 납니다.

`--server-url` / `--api-token` 은 선택된 서버의 해당 필드를 덮어씁니다. `--server` 없이 두 값을
모두 주면 설정 파일과 무관한 일회성 호출이 됩니다 — 이때는 **custom field alias 도 비어 있습니다.**
다른 서버의 alias 를 그대로 적용하면 엉뚱한 custom field id 로 값이 나가기 때문입니다. alias 가
필요하면 `--server <name>` 을 함께 주세요.

```bash
redmine --server-url https://other.example.com --api-token zzzz projects
redmine --server company --server-url https://staging.example.com issues   # alias 는 company 것
```

참고: `--api-token` 은 인자로 노출되므로 `ps` 와 셸 히스토리에 남습니다. 상시 사용은 `config.yml`
에 저장하는 쪽이 안전합니다.

### 0.3.0 이하에서 올라오는 경우

설정 파일은 `config.yml` 하나만 읽습니다. 예전 `config.toml` 과 `REDMINE_URL` /
`REDMINE_API_TOKEN` 환경 변수는 더 이상 사용하지 않습니다. 환경 변수를 쓰고 있었다면 한 줄로
옮길 수 있습니다.

```bash
printf '%s' "$REDMINE_API_TOKEN" | redmine config server add company --url "$REDMINE_URL"
```

## Usage

```bash
redmine projects
redmine issues --project myproj --status 1
redmine issue 1234
redmine issue create --project myproj --subject "..." --description "..."
redmine time-entry create --issue 1234 --hours 2.5 --comment "..."
```

### Scripting-safe creates (`--id-only` / `--token-only`)

Single-resource commands (`issue create`, `issue 1234`, `issue update`,
`time-entry create`, etc.) output the **unwrapped** resource JSON — the
fields live at the top level, not under a `{"issue": ...}` wrapper. List
commands wrap with their plural key (`{"issues": [...], "total_count": N}`).

For destructive commands that produce a new resource, plain JSON parsing
in shell scripts is fragile — a broken `jq` / `python -c` pipeline on the
caller side has been mistaken for a failed create in practice, tempting
the caller to retry and creating a duplicate. To eliminate that hazard,
every resource-creating command exposes a stripped-output flag:

```bash
issue_id=$(redmine issue create --project myproj --subject "..." --id-only)
te_id=$(redmine time-entry create --issue 1234 --hours 2.5 --id-only)
relation_id=$(redmine issue 1234 add-relation --to 1235 --id-only)
ver_id=$(redmine version create demo --name v2 --id-only)
news_id=$(redmine news create demo --title "Release" --id-only)
group_id=$(redmine group create --name QA --id-only)
membership_id=$(redmine membership add demo --user 11 --role 4 --id-only)

# Attachment upload returns a token (used by the follow-up issue update).
token=$(redmine attachment upload --issue 1234 --file ./screenshot.png --token-only)
```

These flags print **only** the integer ID (or token string) followed by a
newline — no JSON, no wrapper, nothing else to parse.

### Manage aliases

custom field alias 는 서버마다 따로 관리됩니다. 회사 서버의 `state` 가 7 이고 개인 서버에서는
3 이어도 서로 섞이지 않습니다.

```bash
redmine config alias list                            # 선택된 서버의 alias
redmine config alias set state 7                     # default_server 에 저장
redmine --server personal config alias set state 3   # 특정 서버에 저장
redmine config alias remove state
```

주의: alias 를 저장하거나 `config server add/remove/use` 를 실행하면 `config.yml` 을 통째로 다시
씁니다. **손으로 넣은 YAML 주석은 지워집니다.** 쓰기는 같은 디렉터리의 임시 파일에 기록한 뒤
rename 하므로, 도중에 중단돼도 기존 파일이 깨지지는 않습니다.

CLI 가 모르는 키가 있으면 저장이 아니라 **읽기 단계에서 에러**가 납니다. `api-token` 처럼 오타 난
키가 조용히 무시되다가 "토큰이 없다" 로 나타나는 것을 막기 위해서입니다.

See `redmine --help`.

## License

MIT.
