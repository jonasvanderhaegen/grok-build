#!/usr/bin/env bash
# Install a forked grok release (macOS Apple Silicon).
# Usage:
#   bash scripts/install-from-release.sh [TAG]
#   bash scripts/install-from-release.sh v0.2.110-plugin-hooks.1
#   GROK_RELEASE_REPO=jonasvanderhaegen/grok-build bash scripts/install-from-release.sh
set -euo pipefail

REPO="${GROK_RELEASE_REPO:-jonasvanderhaegen/grok-build}"
TAG="${1:-}"
ASSET="grok-macos-aarch64.tar.gz"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script is for macOS. On Windows use scripts/install-from-release.ps1" >&2
  exit 1
fi
if [[ "$(uname -m)" != "arm64" ]]; then
  echo "This script installs the Apple Silicon (arm64) build." >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1 && ! command -v curl >/dev/null 2>&1; then
  echo "Need gh or curl on PATH" >&2
  exit 1
fi

if [[ -z "$TAG" ]]; then
  if command -v gh >/dev/null 2>&1; then
    TAG=$(gh release view --repo "$REPO" --json tagName -q .tagName)
  else
    echo "Pass TAG explicitly when gh is not installed, e.g. v0.2.110-plugin-hooks.1" >&2
    exit 1
  fi
fi
VERSION="${TAG#v}"
DEST_DIR="${HOME}/.grok/downloads"
BIN_DIR="${HOME}/.grok/bin"
DEST="${DEST_DIR}/grok-${VERSION}-macos-aarch64"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
echo "Downloading ${URL}"
if command -v gh >/dev/null 2>&1; then
  gh release download "$TAG" --repo "$REPO" --pattern "$ASSET" --dir "$TMP"
else
  curl -fsSL -o "${TMP}/${ASSET}" "$URL"
fi

tar -xzf "${TMP}/${ASSET}" -C "$TMP"
if [[ ! -f "${TMP}/grok" ]]; then
  echo "Archive missing grok binary" >&2
  exit 1
fi

mkdir -p "$DEST_DIR" "$BIN_DIR"
install -m 755 "${TMP}/grok" "$DEST"
ln -sfn "$DEST" "${BIN_DIR}/grok"
ln -sfn "$DEST" "${BIN_DIR}/agent"

echo "Installed: $DEST"
echo "Symlinks:  ${BIN_DIR}/grok -> $DEST"
"${BIN_DIR}/grok" --version || true
echo "Open a new terminal / new grok session so plugin hooks load at cold start."
