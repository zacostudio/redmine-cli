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

# Print just the new issue ID — safe for shell scripting.
id=$(redmine issue create --project myproj --subject "..." --id-only)

redmine time-entry create --issue 1234 --hours 2.5 --comment "..."
```

The default `issue create` output is the full JSON of the created issue
(unwrapped — fields live at the top level, not under `{"issue": ...}`).
Use `--id-only` when scripting: it bypasses JSON parsing entirely, so a
broken `jq` / `python -c` pipeline on the caller side can no longer be
mistaken for a failed create — which would otherwise tempt the caller to
retry and produce a duplicate issue.

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
