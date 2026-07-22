# Install forked grok (plugin-hooks-on-spawn)

Prebuilt binaries: https://github.com/jonasvanderhaegen/grok-build/releases

| Platform | Asset |
|----------|--------|
| macOS Apple Silicon | `grok-macos-aarch64.tar.gz` |
| Windows 11 x64 | `grok-windows-x86_64.zip` |
| Windows 11 ARM | `grok-windows-aarch64.zip` |

## macOS (Apple Silicon)

```bash
bash scripts/install-from-release.sh
# or pin a tag:
bash scripts/install-from-release.sh v0.2.110-plugin-hooks.1
```

Installs to `~/.grok/downloads/` and symlinks `~/.grok/bin/grok`.

## Windows 11 (x64 or ARM)

```powershell
.\scripts\install-from-release.ps1
# or:
.\scripts\install-from-release.ps1 -Tag v0.2.110-plugin-hooks.1
```

Installs `grok.exe` under `%USERPROFILE%\.grok\bin` and adds that folder to user PATH if needed.

## After install

Start a **new** Grok session so plugin PreToolUse hooks load at cold start (no `/plugins reload`).

## Cutting a release

```bash
git tag v0.2.110-plugin-hooks.1
git push fork main
git push fork v0.2.110-plugin-hooks.1
```

Or: Actions → **Release** → Run workflow (with tag).

Matrix builds run in parallel (macOS-14 + Windows x64 + Windows ARM). Expect multi-hour wall time on first build.
