---
name: redmine-release
description: This skill should be used when the user asks to "릴리스", "배포", "release", "버전 올려", "homebrew tap 갱신", "Formula 업데이트", "tap에 배포", or mentions cutting a new redmine-cli version. Automates the end-to-end release flow for zacostudio/redmine-cli — Cargo.toml bump, CHANGELOG.md update, commit/tag/push, GitHub Release wait, and homebrew-redmine Formula update.
---

# Redmine CLI Release Skill

End-to-end release workflow for `zacostudio/redmine-cli` and the matching `zacostudio/homebrew-redmine` tap.

## When to Use

Trigger this skill on requests like:
- "v0.1.2로 배포해줘", "다음 버전 릴리스"
- "homebrew tap 갱신해줘", "Formula 업데이트"
- "릴리스 워크플로우 돌려"

## Prerequisites

Verify before starting:
- Working tree clean on `master` (`git status`, `git rev-parse --abbrev-ref HEAD`)
- `gh` CLI authenticated (`gh auth status`)
- Push permission on both `zacostudio/redmine-cli` and `zacostudio/homebrew-redmine`
- Latest tag matches expectation (`git tag --sort=-v:refname | head -1`)

If any check fails, stop and report to the user.

## Release Flow

Execute the steps in order. Pause for user confirmation before any push.

### 1. Decide next version

- Inspect commits since last tag: `git log <last-tag>..HEAD --oneline`
- Apply SemVer based on conventional commit prefixes:
  - `feat:` → MINOR bump
  - `fix:` / `perf:` → PATCH bump
  - `feat!:` or `BREAKING CHANGE:` → MAJOR bump
  - Only `docs:` / `chore:` / `ci:` / `test:` → confirm with user whether a release is warranted
- Ask the user to confirm the version if intent is ambiguous.

### 2. Bump Cargo.toml

Edit `Cargo.toml` and change the `version = "..."` line under `[package]` to the new version.

### 3. Refresh Cargo.lock

Run `cargo build` (debug profile is enough). This rewrites `Cargo.lock` with the new version.

### 4. Update CHANGELOG.md

**Must happen before commit.**

If `CHANGELOG.md` does not exist:
- Create it using the Keep a Changelog template in `references/changelog-format.md`.
- Backfill prior tags (`v0.1.0`, `v0.1.1`, ...) by reading `git log <prev>..<next>` for each existing tag.

If it exists:
- Move any `[Unreleased]` items into a new `## [X.Y.Z] - YYYY-MM-DD` section.
- Group new commits since the previous tag into sections (`Added` / `Changed` / `Fixed` / `Performance` / `Removed`) using `references/changelog-format.md` for the conventional-commit → section mapping.
- Use today's date (current real date, not a guess) for the new section heading.
- Update comparison links at the bottom if present.

Write CHANGELOG entries in Korean (사용자 기본 언어 일치). Each bullet should be user-facing and reference the underlying commit when helpful.

### 5. Review the diff

Show the user `git diff Cargo.toml Cargo.lock CHANGELOG.md` and wait for approval before committing.

### 6. Commit

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: release vX.Y.Z"
```

Match repo convention: lowercase `chore:` prefix, no body unless multi-line context is required.

### 7. Tag

```bash
git tag -a vX.Y.Z -m "vX.Y.Z — <one-line summary>"
```

The summary should match the most prominent CHANGELOG entry for that version.

### 8. Push (requires user confirmation)

Confirm with the user, then:

```bash
git push origin master --follow-tags
```

`--follow-tags` ensures both the commit and the new annotated tag reach `origin` in one shot.

### 9. Wait for the release workflow

The `.github/workflows/release.yml` workflow triggers on `v*` tag push and builds 3 targets:
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`

Watch progress:

```bash
gh run watch --repo zacostudio/redmine-cli --exit-status \
  $(gh run list --repo zacostudio/redmine-cli --workflow=Release --branch=vX.Y.Z --limit 1 --json databaseId --jq '.[0].databaseId')
```

If the workflow fails, stop and surface the failing job to the user — do not proceed to the tap update.

### 10. Collect SHA256 values

Once the GitHub Release is published, fetch the three `.sha256` sidecar files. The release workflow uploads them next to each `.tar.gz`:

```bash
VERSION=vX.Y.Z
BASE="https://github.com/zacostudio/redmine-cli/releases/download/${VERSION}"
for target in aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu; do
  curl -fsSL "${BASE}/redmine-${VERSION}-${target}.tar.gz.sha256"
done
```

Each line is `<sha256>  <filename>`. Extract the 64-character hex.

### 11. Update homebrew-redmine Formula

Use the helper script:

```bash
.claude/skills/release/scripts/update-formula.sh X.Y.Z
```

The script clones (or pulls) `zacostudio/homebrew-redmine` into `/tmp/homebrew-redmine`, downloads the three `.sha256` files, rewrites `Formula/redmine.rb`, and shows a diff.

If running manually, see `references/formula-template.md` for the exact field layout. Only two things change between versions:
1. `version "X.Y.Z"` line
2. The three `sha256 "..."` lines (one per platform block, in the order: arm mac → intel mac → linux)

### 12. Commit and push the tap (requires user confirmation)

```bash
cd /tmp/homebrew-redmine
git add Formula/redmine.rb
git commit -m "bump redmine to vX.Y.Z"
git push origin master
```

The tap repo's default branch is `master`.

### 13. Verify installation (optional but recommended)

```bash
brew update
brew upgrade zacostudio/redmine/redmine || brew install zacostudio/redmine/redmine
redmine --version  # should print X.Y.Z
```

## Safety Rules

- **Never** push (commit, tag, or tap update) without an explicit user confirmation for that push.
- **Never** force-push to either repo.
- **Never** use `main` — both repos use `master`. If a step suggests otherwise, treat it as a bug.
- If a step fails, stop the flow; do not paper over with a destructive command (`reset --hard`, `tag -d` on a pushed tag, etc.) without asking.
- If the GitHub Release build fails after the tag is pushed, prefer fixing forward (next patch) over deleting the tag, unless the user explicitly asks.

## Additional Resources

### References
- `references/changelog-format.md` — Keep a Changelog template + conventional-commit → section mapping
- `references/formula-template.md` — Annotated `redmine.rb` template

### Scripts
- `scripts/update-formula.sh` — Clone/pull tap, fetch SHA256s, rewrite Formula, print diff
