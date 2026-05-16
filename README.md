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

See `redmine --help`.

## License

MIT.
