# Install forked grok (plugin-hooks-on-spawn)

Prebuilt binary: https://github.com/jonasvanderhaegen/grok-build/releases

| Platform | Asset |
|----------|--------|
| macOS Apple Silicon | `grok-macos-aarch64.tar.gz` |

## Install (Mac mini / Apple Silicon)

```bash
bash scripts/install-from-release.sh
# or pin a tag:
bash scripts/install-from-release.sh v0.2.110-plugin-hooks.1
```

Installs to `~/.grok/downloads/` and symlinks `~/.grok/bin/grok`.

## After install

Start a **new** Grok session so plugin PreToolUse hooks load at cold start (no `/plugins reload`).

## Cutting a release

```bash
git tag v0.2.110-plugin-hooks.1
git push fork main
git push fork v0.2.110-plugin-hooks.1
```

Or: Actions → **Release** → Run workflow (with tag).

Single-job build on `macos-14`. Expect multi-hour wall time on a cold cache.
