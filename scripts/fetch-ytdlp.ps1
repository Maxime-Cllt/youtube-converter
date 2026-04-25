# Downloads yt-dlp binaries for all supported targets and names them with the
# Rust target triple suffix expected by Tauri's `externalBin`.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts/fetch-ytdlp.ps1

$ErrorActionPreference = "Stop"

$root    = Split-Path -Parent $PSScriptRoot
$out     = Join-Path $root "src-tauri\binaries"
$release = "https://github.com/yt-dlp/yt-dlp/releases/latest/download"

New-Item -ItemType Directory -Force -Path $out | Out-Null

function Get-Bin($url, $dest) {
    if (Test-Path $dest) {
        Write-Host "[skip] $(Split-Path -Leaf $dest) already exists"
        return
    }
    Write-Host "[fetch] $(Split-Path -Leaf $dest)"
    Invoke-WebRequest -Uri $url -OutFile "$dest.tmp" -UseBasicParsing
    Move-Item -Force "$dest.tmp" $dest
}

Get-Bin "$release/yt-dlp.exe"           (Join-Path $out "yt-dlp-x86_64-pc-windows-msvc.exe")
Get-Bin "$release/yt-dlp_macos"         (Join-Path $out "yt-dlp-aarch64-apple-darwin")
Copy-Item -Force (Join-Path $out "yt-dlp-aarch64-apple-darwin") (Join-Path $out "yt-dlp-x86_64-apple-darwin")
Get-Bin "$release/yt-dlp_linux"         (Join-Path $out "yt-dlp-x86_64-unknown-linux-gnu")
Get-Bin "$release/yt-dlp_linux_aarch64" (Join-Path $out "yt-dlp-aarch64-unknown-linux-gnu")

Write-Host ""
Write-Host "yt-dlp binaries ready in $out"
Get-ChildItem $out | Select-Object -ExpandProperty Name
