# Downloads yt-dlp + ffmpeg + ffprobe binaries and names them with the Rust
# target triple suffix expected by Tauri's `externalBin`.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts/fetch-binaries.ps1

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$out  = Join-Path $root "src-tauri\binaries"
$work = Join-Path $env:TEMP "ytconv-fetch-$(Get-Random)"

New-Item -ItemType Directory -Force -Path $out  | Out-Null
New-Item -ItemType Directory -Force -Path $work | Out-Null

try {
    $ytdlp     = "https://github.com/yt-dlp/yt-dlp/releases/latest/download"
    $ffmpegBtb = "https://github.com/BtbN/FFmpeg-Builds/releases/latest/download"
    $evermeet  = "https://evermeet.cx/ffmpeg/getrelease"

    function Get-File($url, $dest) {
        Write-Host "[fetch] $(Split-Path -Leaf $dest)"
        Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing
    }

    function Get-Bin($url, $dest) {
        if (Test-Path $dest) {
            Write-Host "[skip] $(Split-Path -Leaf $dest) exists"
            return
        }
        Get-File $url $dest
    }

    # ===== yt-dlp =====
    Write-Host "▼ yt-dlp"
    Get-Bin "$ytdlp/yt-dlp.exe"           (Join-Path $out "yt-dlp-x86_64-pc-windows-msvc.exe")
    Get-Bin "$ytdlp/yt-dlp_macos"         (Join-Path $out "yt-dlp-aarch64-apple-darwin")
    Copy-Item -Force (Join-Path $out "yt-dlp-aarch64-apple-darwin") (Join-Path $out "yt-dlp-x86_64-apple-darwin")
    Get-Bin "$ytdlp/yt-dlp_linux"         (Join-Path $out "yt-dlp-x86_64-unknown-linux-gnu")
    Get-Bin "$ytdlp/yt-dlp_linux_aarch64" (Join-Path $out "yt-dlp-aarch64-unknown-linux-gnu")

    # ===== ffmpeg + ffprobe =====
    Write-Host ""
    Write-Host "▼ ffmpeg + ffprobe"

    function Extract-BtbN-Win($archive, $triple) {
        $stage = Join-Path $work "btbn-$triple"
        New-Item -ItemType Directory -Force -Path $stage | Out-Null
        Expand-Archive -Force -Path $archive -DestinationPath $stage
        $ffm = Get-ChildItem -Path $stage -Filter "ffmpeg.exe"  -Recurse | Select-Object -First 1
        $ffp = Get-ChildItem -Path $stage -Filter "ffprobe.exe" -Recurse | Select-Object -First 1
        Move-Item -Force $ffm.FullName (Join-Path $out "ffmpeg-$triple.exe")
        Move-Item -Force $ffp.FullName (Join-Path $out "ffprobe-$triple.exe")
    }

    function Extract-BtbN-Unix($archive, $triple) {
        $stage = Join-Path $work "btbn-$triple"
        New-Item -ItemType Directory -Force -Path $stage | Out-Null
        # tar.exe ships with Win10+
        & tar -xJf $archive -C $stage
        $ffm = Get-ChildItem -Path $stage -Filter "ffmpeg"  -Recurse -File | Select-Object -First 1
        $ffp = Get-ChildItem -Path $stage -Filter "ffprobe" -Recurse -File | Select-Object -First 1
        Move-Item -Force $ffm.FullName (Join-Path $out "ffmpeg-$triple")
        Move-Item -Force $ffp.FullName (Join-Path $out "ffprobe-$triple")
    }

    # Windows x86_64
    if (-not (Test-Path (Join-Path $out "ffmpeg-x86_64-pc-windows-msvc.exe"))) {
        $a = Join-Path $work "win64.zip"
        Get-File "$ffmpegBtb/ffmpeg-master-latest-win64-gpl.zip" $a
        Extract-BtbN-Win $a "x86_64-pc-windows-msvc"
    } else { Write-Host "[skip] windows x86_64 ffmpeg/ffprobe present" }

    # Linux x86_64
    if (-not (Test-Path (Join-Path $out "ffmpeg-x86_64-unknown-linux-gnu"))) {
        $a = Join-Path $work "lin64.tar.xz"
        Get-File "$ffmpegBtb/ffmpeg-master-latest-linux64-gpl.tar.xz" $a
        Extract-BtbN-Unix $a "x86_64-unknown-linux-gnu"
    } else { Write-Host "[skip] linux x86_64 ffmpeg/ffprobe present" }

    # Linux aarch64
    if (-not (Test-Path (Join-Path $out "ffmpeg-aarch64-unknown-linux-gnu"))) {
        $a = Join-Path $work "linarm64.tar.xz"
        Get-File "$ffmpegBtb/ffmpeg-master-latest-linuxarm64-gpl.tar.xz" $a
        Extract-BtbN-Unix $a "aarch64-unknown-linux-gnu"
    } else { Write-Host "[skip] linux aarch64 ffmpeg/ffprobe present" }

    # macOS — evermeet.cx Intel binaries (Rosetta 2 on Apple Silicon)
    if (-not (Test-Path (Join-Path $out "ffmpeg-x86_64-apple-darwin"))) {
        Get-File "$evermeet/ffmpeg/zip"  (Join-Path $work "ffmpeg-mac.zip")
        Get-File "$evermeet/ffprobe/zip" (Join-Path $work "ffprobe-mac.zip")
        Expand-Archive -Force -Path (Join-Path $work "ffmpeg-mac.zip")  -DestinationPath (Join-Path $work "mac")
        Expand-Archive -Force -Path (Join-Path $work "ffprobe-mac.zip") -DestinationPath (Join-Path $work "mac")
        Copy-Item -Force (Join-Path $work "mac\ffmpeg")  (Join-Path $out "ffmpeg-x86_64-apple-darwin")
        Copy-Item -Force (Join-Path $work "mac\ffprobe") (Join-Path $out "ffprobe-x86_64-apple-darwin")
        Copy-Item -Force (Join-Path $work "mac\ffmpeg")  (Join-Path $out "ffmpeg-aarch64-apple-darwin")
        Copy-Item -Force (Join-Path $work "mac\ffprobe") (Join-Path $out "ffprobe-aarch64-apple-darwin")
    } else { Write-Host "[skip] macOS ffmpeg/ffprobe present" }

    Write-Host ""
    Write-Host "✓ Binaries ready in $out"
    Get-ChildItem $out | Select-Object -ExpandProperty Name
}
finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
