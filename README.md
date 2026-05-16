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

Set environment variables:

```bash
export REDMINE_URL=https://redmine.example.com
export REDMINE_API_TOKEN=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

Or create `~/.config/redmine-cli/config.toml`:

```toml
server_url = "https://redmine.example.com"
api_token  = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

[custom_fields]
state = 7
qa    = 8
```

CLI flags `--server-url` and `--api-token` override env and file.

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

```bash
redmine config alias list
redmine config alias set state 7
redmine config alias remove state
```

Aliases are persisted to `~/.config/redmine-cli/config.toml`.

See `redmine --help`.

## License

MIT.
