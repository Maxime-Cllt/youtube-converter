#!/usr/bin/env bash
# Downloads yt-dlp + ffmpeg + ffprobe binaries for all supported targets and
# names them with the Rust target triple suffix expected by Tauri's
# `externalBin` mechanism.
#
# Run before `bun tauri build` (or once before `bun tauri dev`).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/src-tauri/binaries"
mkdir -p "$OUT"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

YTDLP="https://github.com/yt-dlp/yt-dlp/releases/latest/download"
FFMPEG_BTBN="https://github.com/BtbN/FFmpeg-Builds/releases/latest/download"
EVERMEET="https://evermeet.cx/ffmpeg/getrelease"

dl() {
  local url="$1" dest="$2"
  echo "[fetch] $(basename "$dest")"
  curl -fL --retry 3 --progress-bar -o "$dest.tmp" "$url"
  mv "$dest.tmp" "$dest"
}

dl_bin() {
  local url="$1" dest="$2"
  if [[ -f "$dest" ]]; then
    echo "[skip] $(basename "$dest") exists"
    return
  fi
  dl "$url" "$dest"
  chmod +x "$dest" 2>/dev/null || true
}

extract_btbn_unix() {
  local archive="$1" triple="$2"
  local stage="$WORK/btbn-$triple"
  mkdir -p "$stage"
  # macOS tar doesn't support --wildcards, so extract everything then find.
  tar -xJf "$archive" -C "$stage"
  local ffm ffp
  ffm="$(find "$stage" -type f -name ffmpeg  -path '*/bin/*' | head -n1)"
  ffp="$(find "$stage" -type f -name ffprobe -path '*/bin/*' | head -n1)"
  if [[ -z "$ffm" || -z "$ffp" ]]; then
    echo "Error: could not locate ffmpeg/ffprobe inside $archive" >&2
    return 1
  fi
  mv "$ffm" "$OUT/ffmpeg-$triple"
  mv "$ffp" "$OUT/ffprobe-$triple"
  chmod +x "$OUT/ffmpeg-$triple" "$OUT/ffprobe-$triple"
}

extract_btbn_win() {
  local archive="$1" triple="$2"
  local stage="$WORK/btbn-$triple"
  mkdir -p "$stage"
  unzip -j "$archive" "*/bin/ffmpeg.exe" "*/bin/ffprobe.exe" -d "$stage" >/dev/null
  mv "$stage/ffmpeg.exe"  "$OUT/ffmpeg-$triple.exe"
  mv "$stage/ffprobe.exe" "$OUT/ffprobe-$triple.exe"
}

# ===== yt-dlp =====
echo "▼ yt-dlp"
dl_bin "$YTDLP/yt-dlp_macos"          "$OUT/yt-dlp-aarch64-apple-darwin"
cp -f "$OUT/yt-dlp-aarch64-apple-darwin" "$OUT/yt-dlp-x86_64-apple-darwin"
chmod +x "$OUT/yt-dlp-x86_64-apple-darwin"
dl_bin "$YTDLP/yt-dlp_linux"          "$OUT/yt-dlp-x86_64-unknown-linux-gnu"
dl_bin "$YTDLP/yt-dlp_linux_aarch64"  "$OUT/yt-dlp-aarch64-unknown-linux-gnu"
dl_bin "$YTDLP/yt-dlp.exe"            "$OUT/yt-dlp-x86_64-pc-windows-msvc.exe"

# ===== ffmpeg + ffprobe =====
echo
echo "▼ ffmpeg + ffprobe"

# Linux x86_64
if [[ ! -f "$OUT/ffmpeg-x86_64-unknown-linux-gnu" || ! -f "$OUT/ffprobe-x86_64-unknown-linux-gnu" ]]; then
  dl "$FFMPEG_BTBN/ffmpeg-master-latest-linux64-gpl.tar.xz" "$WORK/lin64.tar.xz"
  extract_btbn_unix "$WORK/lin64.tar.xz" "x86_64-unknown-linux-gnu"
else
  echo "[skip] linux x86_64 ffmpeg/ffprobe present"
fi

# Linux aarch64
if [[ ! -f "$OUT/ffmpeg-aarch64-unknown-linux-gnu" || ! -f "$OUT/ffprobe-aarch64-unknown-linux-gnu" ]]; then
  dl "$FFMPEG_BTBN/ffmpeg-master-latest-linuxarm64-gpl.tar.xz" "$WORK/linarm64.tar.xz"
  extract_btbn_unix "$WORK/linarm64.tar.xz" "aarch64-unknown-linux-gnu"
else
  echo "[skip] linux aarch64 ffmpeg/ffprobe present"
fi

# Windows x86_64
if [[ ! -f "$OUT/ffmpeg-x86_64-pc-windows-msvc.exe" || ! -f "$OUT/ffprobe-x86_64-pc-windows-msvc.exe" ]]; then
  dl "$FFMPEG_BTBN/ffmpeg-master-latest-win64-gpl.zip" "$WORK/win64.zip"
  extract_btbn_win "$WORK/win64.zip" "x86_64-pc-windows-msvc"
else
  echo "[skip] windows x86_64 ffmpeg/ffprobe present"
fi

# macOS — evermeet.cx ships Intel-only static builds; copied for both arches.
# Apple Silicon runs them via Rosetta 2 with negligible perf impact for
# audio extraction / muxing.
if [[ ! -f "$OUT/ffmpeg-x86_64-apple-darwin" || ! -f "$OUT/ffprobe-x86_64-apple-darwin" \
   || ! -f "$OUT/ffmpeg-aarch64-apple-darwin" || ! -f "$OUT/ffprobe-aarch64-apple-darwin" ]]; then
  dl "$EVERMEET/ffmpeg/zip"  "$WORK/ffmpeg-mac.zip"
  dl "$EVERMEET/ffprobe/zip" "$WORK/ffprobe-mac.zip"
  unzip -o "$WORK/ffmpeg-mac.zip"  -d "$WORK/mac" >/dev/null
  unzip -o "$WORK/ffprobe-mac.zip" -d "$WORK/mac" >/dev/null
  cp "$WORK/mac/ffmpeg"  "$OUT/ffmpeg-x86_64-apple-darwin"
  cp "$WORK/mac/ffprobe" "$OUT/ffprobe-x86_64-apple-darwin"
  cp "$WORK/mac/ffmpeg"  "$OUT/ffmpeg-aarch64-apple-darwin"
  cp "$WORK/mac/ffprobe" "$OUT/ffprobe-aarch64-apple-darwin"
  chmod +x "$OUT"/ffmpeg-*-apple-darwin "$OUT"/ffprobe-*-apple-darwin
else
  echo "[skip] macOS ffmpeg/ffprobe present"
fi

echo
echo "✓ Binaries ready in $OUT"
ls -1 "$OUT"
