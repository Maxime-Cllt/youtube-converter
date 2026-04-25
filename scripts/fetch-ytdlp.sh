#!/usr/bin/env bash
# Downloads yt-dlp binaries for all supported targets and renames them with the
# Rust target triple suffix expected by Tauri's `externalBin` mechanism.
#
# Run before `bun tauri build` (or once before `bun tauri dev`).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/src-tauri/binaries"
RELEASE="https://github.com/yt-dlp/yt-dlp/releases/latest/download"

mkdir -p "$OUT"

download() {
  local url="$1"
  local dest="$2"
  if [[ -f "$dest" ]]; then
    echo "[skip] $dest already exists"
    return
  fi
  echo "[fetch] $(basename "$dest")"
  curl -fL --retry 3 --progress-bar -o "$dest.tmp" "$url"
  mv "$dest.tmp" "$dest"
  chmod +x "$dest" 2>/dev/null || true
}

# macOS — single universal binary, copy for both arches
download "$RELEASE/yt-dlp_macos" "$OUT/yt-dlp-aarch64-apple-darwin"
cp -f "$OUT/yt-dlp-aarch64-apple-darwin" "$OUT/yt-dlp-x86_64-apple-darwin"
chmod +x "$OUT/yt-dlp-x86_64-apple-darwin"

# Linux
download "$RELEASE/yt-dlp_linux"        "$OUT/yt-dlp-x86_64-unknown-linux-gnu"
download "$RELEASE/yt-dlp_linux_aarch64" "$OUT/yt-dlp-aarch64-unknown-linux-gnu"

# Windows
download "$RELEASE/yt-dlp.exe" "$OUT/yt-dlp-x86_64-pc-windows-msvc.exe"

echo
echo "yt-dlp binaries ready in $OUT"
ls -1 "$OUT"
