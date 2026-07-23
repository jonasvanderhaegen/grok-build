# Install forked grok

Prebuilt binary: https://github.com/jonasvanderhaegen/grok-build/releases

| Platform | Asset |
|----------|--------|
| macOS Apple Silicon | `grok-macos-aarch64.tar.gz` |

Includes: plugin hooks at session spawn + **MCP tools/call progress** (Progressive `use_tool` path).

## Install (Mac mini / Apple Silicon)

```bash
bash scripts/install-from-release.sh
# or pin a tag:
bash scripts/install-from-release.sh v0.2.110-mcp-progress.1
```

Installs to `~/.grok/downloads/` and symlinks `~/.grok/bin/grok`.

## After install

Start a **new** Grok session so plugin PreToolUse hooks load at cold start (no `/plugins reload`).

## Cutting a release (GitHub Actions — not local cargo)

Heavy builds run on **macos-14** via `.github/workflows/release.yml`:

```bash
git push fork main
git tag v0.2.110-mcp-progress.1
git push fork v0.2.110-mcp-progress.1
# or: Actions → Release → Run workflow (tag input)
```

Targeted unit tests run on **ubuntu-latest** via `.github/workflows/ci.yml` on every push to `main`.

Single-job macOS release. Expect multi-hour wall time on a cold cache; warm cache is faster.
