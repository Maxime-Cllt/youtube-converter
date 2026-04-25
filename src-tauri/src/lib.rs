use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{RwLock, Semaphore};

const MAX_PARALLEL_DOWNLOADS: usize = 3;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    url: String,
    progress: f32,
    status: String,
    speed: Option<String>,
    eta: Option<String>,
    error: Option<String>,
}

#[derive(serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
struct DownloadOptions {
    // Mode: "audio" or "video"
    mode: String,

    // Output
    output_dir: Option<String>,
    output_template: String,

    // Audio
    audio_format: String,
    audio_quality: String,

    // Video
    video_resolution: String, // "best" | "2160" | "1440" | "1080" | "720" | "480" | "360"
    video_container: String,  // "default" | "mp4" | "mkv" | "webm"
    video_codec: String,      // "any" | "h264" | "h265" | "vp9" | "av1"
    prefer_free_formats: bool,

    // Embed
    embed_thumbnail: bool,
    add_metadata: bool,
    embed_chapters: bool,
    embed_subs: bool,

    // Subtitles
    write_subs: bool,
    write_auto_subs: bool,
    sub_langs: String, // e.g. "en,fr"

    // SponsorBlock
    sponsorblock_remove: bool,
    sponsorblock_categories: Vec<String>,

    // Network / performance
    limit_rate: String,         // "1M", "500K", or empty
    concurrent_fragments: u32,  // -N option, 1..=16
    retries: u32,               // default 10
    cookies_file: Option<String>,
    proxy: Option<String>,
    user_agent: Option<String>,

    // Playlist
    no_playlist: bool,
    playlist_items: String,

    // Misc
    write_info_json: bool,
    write_description: bool,
    custom_args: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum YtDlpCommand {
    Sidecar(PathBuf),
    Direct(PathBuf),
    PythonModule(String),
}

impl YtDlpCommand {
    fn create(&self) -> Command {
        match self {
            YtDlpCommand::Sidecar(path) | YtDlpCommand::Direct(path) => Command::new(path),
            YtDlpCommand::PythonModule(python) => {
                let mut cmd = Command::new(python);
                cmd.arg("-m").arg("yt_dlp");
                cmd
            }
        }
    }

    fn label(&self) -> String {
        match self {
            YtDlpCommand::Sidecar(p) => format!("bundled ({})", p.display()),
            YtDlpCommand::Direct(p) => format!("system ({})", p.display()),
            YtDlpCommand::PythonModule(py) => format!("python module via {}", py),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum FfmpegLocation {
    /// ffmpeg is available on PATH; no `--ffmpeg-location` needed.
    SystemPath,
    /// Resolved to a specific directory (system-detected absolute path or sidecar).
    Dir(PathBuf),
}

impl FfmpegLocation {
    fn label(&self) -> String {
        match self {
            FfmpegLocation::SystemPath => "system PATH".to_string(),
            FfmpegLocation::Dir(p) => p.display().to_string(),
        }
    }
}

#[derive(Default)]
struct AppState {
    ytdlp: Arc<RwLock<Option<YtDlpCommand>>>,
    ffmpeg: Arc<RwLock<Option<FfmpegLocation>>>,
}

impl AppState {
    async fn set_ytdlp(&self, cmd: YtDlpCommand) {
        *self.ytdlp.write().await = Some(cmd);
    }

    async fn get_ytdlp(&self) -> Option<YtDlpCommand> {
        self.ytdlp.read().await.clone()
    }

    async fn set_ffmpeg(&self, loc: FfmpegLocation) {
        *self.ffmpeg.write().await = Some(loc);
    }

    async fn get_ffmpeg(&self) -> Option<FfmpegLocation> {
        self.ffmpeg.read().await.clone()
    }
}

fn target_triple() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        "aarch64-pc-windows-msvc"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "aarch64-unknown-linux-gnu"
    } else {
        ""
    }
}

fn get_downloads_dir() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .map(|p| format!("{}\\Downloads", p))
            .map_err(|_| "Could not determine Windows downloads directory".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME")
            .map(|h| format!("{}/Downloads", h))
            .map_err(|_| "Could not determine downloads directory".to_string())
    }
}

fn version_check_sync(path: &std::path::Path) -> bool {
    std::process::Command::new(path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn version_check_cmd_sync(cmd: &str, args: &[&str]) -> bool {
    std::process::Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn sidecar_candidate_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.to_path_buf());
        }
    }
    if let Ok(resource_dir) = app.path().resource_dir() {
        dirs.push(resource_dir.clone());
        dirs.push(resource_dir.join("binaries"));
    }
    dirs
}

fn resolve_named_sidecar(app: &AppHandle, name: &str) -> Option<PathBuf> {
    let triple = target_triple();
    let plain = if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };
    let suffixed = if cfg!(windows) {
        format!("{}-{}.exe", name, triple)
    } else {
        format!("{}-{}", name, triple)
    };

    for dir in sidecar_candidate_dirs(app) {
        for variant in [plain.as_str(), suffixed.as_str()] {
            let p = dir.join(variant);
            if p.exists() && version_check_sync(&p) {
                return Some(p);
            }
        }
    }
    None
}

fn resolve_sidecar(app: &AppHandle) -> Option<PathBuf> {
    resolve_named_sidecar(app, "yt-dlp")
}

#[cfg(not(target_os = "windows"))]
fn resolve_via_login_shell(cmd: &str) -> Option<PathBuf> {
    let shells: Vec<String> = std::env::var("SHELL")
        .ok()
        .into_iter()
        .chain(["/bin/zsh", "/bin/bash", "/bin/sh"].iter().map(|s| s.to_string()))
        .collect();

    let invocation = format!("command -v {}", cmd);
    for shell in shells {
        if !std::path::Path::new(&shell).exists() {
            continue;
        }
        let output = std::process::Command::new(&shell)
            .args(["-l", "-c", &invocation])
            .output()
            .ok();
        if let Some(out) = output {
            if out.status.success() {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        let p = PathBuf::from(trimmed);
                        if p.exists() {
                            return Some(p);
                        }
                    }
                }
            }
        }
    }
    None
}

fn detect_system_ytdlp() -> Option<YtDlpCommand> {
    let direct_names: &[&str] = if cfg!(windows) {
        &["yt-dlp.exe", "yt-dlp"]
    } else {
        &["yt-dlp"]
    };

    for name in direct_names {
        if version_check_cmd_sync(name, &["--version"]) {
            return Some(YtDlpCommand::Direct(PathBuf::from(name)));
        }
    }

    let mut common: Vec<PathBuf> = Vec::new();

    if cfg!(windows) {
        common.push(PathBuf::from("C:\\Program Files\\yt-dlp\\yt-dlp.exe"));
        common.push(PathBuf::from("C:\\ProgramData\\chocolatey\\bin\\yt-dlp.exe"));
        common.push(PathBuf::from("C:\\Python3\\Scripts\\yt-dlp.exe"));
        common.push(PathBuf::from("C:\\Python\\Scripts\\yt-dlp.exe"));

        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            common.push(PathBuf::from(format!(
                "{}\\Microsoft\\WindowsApps\\yt-dlp.exe",
                local
            )));
            common.push(PathBuf::from(format!(
                "{}\\pipx\\venvs\\yt-dlp\\Scripts\\yt-dlp.exe",
                local
            )));
            for major in 8..=14 {
                common.push(PathBuf::from(format!(
                    "{}\\Programs\\Python\\Python3{}\\Scripts\\yt-dlp.exe",
                    local, major
                )));
            }
        }
        if let Ok(roaming) = std::env::var("APPDATA") {
            for major in 8..=14 {
                common.push(PathBuf::from(format!(
                    "{}\\Python\\Python3{}\\Scripts\\yt-dlp.exe",
                    roaming, major
                )));
            }
        }
    } else {
        common.extend([
            "/usr/local/bin/yt-dlp",
            "/usr/bin/yt-dlp",
            "/opt/homebrew/bin/yt-dlp",
            "/opt/local/bin/yt-dlp",
            "/snap/bin/yt-dlp",
        ].iter().map(PathBuf::from));

        if let Ok(home) = std::env::var("HOME") {
            common.push(PathBuf::from(format!("{}/.local/bin/yt-dlp", home)));
            common.push(PathBuf::from(format!("{}/bin/yt-dlp", home)));
            common.push(PathBuf::from(format!(
                "{}/.local/pipx/venvs/yt-dlp/bin/yt-dlp",
                home
            )));
            common.push(PathBuf::from(format!("{}/.cargo/bin/yt-dlp", home)));

            #[cfg(target_os = "macos")]
            for minor in 8..=14 {
                common.push(PathBuf::from(format!(
                    "{}/Library/Python/3.{}/bin/yt-dlp",
                    home, minor
                )));
            }
        }
    }

    for path in common {
        if path.exists() && version_check_sync(&path) {
            return Some(YtDlpCommand::Direct(path));
        }
    }

    #[cfg(not(target_os = "windows"))]
    if let Some(p) = resolve_via_login_shell("yt-dlp") {
        if version_check_sync(&p) {
            return Some(YtDlpCommand::Direct(p));
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(out) = std::process::Command::new("where").arg("yt-dlp").output() {
        if out.status.success() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                if let Some(line) = s.lines().next() {
                    let p = PathBuf::from(line.trim());
                    if p.exists() && version_check_sync(&p) {
                        return Some(YtDlpCommand::Direct(p));
                    }
                }
            }
        }
    }

    for python in &["python3", "python", "py"] {
        if version_check_cmd_sync(python, &["-m", "yt_dlp", "--version"]) {
            return Some(YtDlpCommand::PythonModule(python.to_string()));
        }
    }

    None
}

fn detect_ytdlp(app: &AppHandle) -> Option<YtDlpCommand> {
    if let Some(p) = resolve_sidecar(app) {
        return Some(YtDlpCommand::Sidecar(p));
    }
    detect_system_ytdlp()
}

/// Resolves a system-installed ffmpeg's directory, ensuring ffprobe sits next to it.
fn detect_system_ffmpeg() -> Option<PathBuf> {
    fn check_pair(ffmpeg: &std::path::Path) -> Option<PathBuf> {
        if !version_check_sync(ffmpeg) {
            return None;
        }
        let dir = ffmpeg.parent()?;
        let probe_name = if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" };
        let probe = dir.join(probe_name);
        if probe.exists() && version_check_sync(&probe) {
            Some(dir.to_path_buf())
        } else {
            None
        }
    }

    // PATH
    let direct: &[&str] = if cfg!(windows) { &["ffmpeg.exe", "ffmpeg"] } else { &["ffmpeg"] };
    for name in direct {
        if version_check_cmd_sync(name, &["-version"]) {
            // We don't know the absolute path — try to resolve it via which/where
            #[cfg(not(target_os = "windows"))]
            if let Ok(out) = std::process::Command::new("which").arg(name).output() {
                if out.status.success() {
                    if let Ok(s) = String::from_utf8(out.stdout) {
                        let p = PathBuf::from(s.trim());
                        if let Some(dir) = check_pair(&p) {
                            return Some(dir);
                        }
                    }
                }
            }
            #[cfg(target_os = "windows")]
            if let Ok(out) = std::process::Command::new("where").arg(name).output() {
                if out.status.success() {
                    if let Ok(s) = String::from_utf8(out.stdout) {
                        if let Some(line) = s.lines().next() {
                            let p = PathBuf::from(line.trim());
                            if let Some(dir) = check_pair(&p) {
                                return Some(dir);
                            }
                        }
                    }
                }
            }
        }
    }

    // Known install locations
    let mut common: Vec<PathBuf> = Vec::new();
    if cfg!(windows) {
        common.push(PathBuf::from("C:\\Program Files\\ffmpeg\\bin\\ffmpeg.exe"));
        common.push(PathBuf::from("C:\\ProgramData\\chocolatey\\bin\\ffmpeg.exe"));
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            common.push(PathBuf::from(format!(
                "{}\\Microsoft\\WindowsApps\\ffmpeg.exe",
                local
            )));
        }
    } else {
        common.extend([
            "/usr/local/bin/ffmpeg",
            "/usr/bin/ffmpeg",
            "/opt/homebrew/bin/ffmpeg",
            "/opt/local/bin/ffmpeg",
            "/snap/bin/ffmpeg",
        ].iter().map(PathBuf::from));
        if let Ok(home) = std::env::var("HOME") {
            common.push(PathBuf::from(format!("{}/.local/bin/ffmpeg", home)));
            common.push(PathBuf::from(format!("{}/bin/ffmpeg", home)));
        }
    }

    for path in common {
        if let Some(dir) = check_pair(&path) {
            return Some(dir);
        }
    }

    // Login shell fallback (macOS/Linux GUI apps inherit a stripped PATH)
    #[cfg(not(target_os = "windows"))]
    if let Some(p) = resolve_via_login_shell("ffmpeg") {
        if let Some(dir) = check_pair(&p) {
            return Some(dir);
        }
    }

    None
}

fn detect_ffmpeg(app: &AppHandle) -> Option<FfmpegLocation> {
    // 1. System install — preferred (fresher, smaller bundle uses)
    if let Some(dir) = detect_system_ffmpeg() {
        return Some(FfmpegLocation::Dir(dir));
    }
    // 2. Bundled sidecar — pin yt-dlp to its directory
    let ffmpeg_sidecar = resolve_named_sidecar(app, "ffmpeg")?;
    let ffprobe_sidecar = resolve_named_sidecar(app, "ffprobe")?;
    let dir = ffmpeg_sidecar.parent()?.to_path_buf();
    // Sanity check: ffprobe is in the same dir
    if ffprobe_sidecar.parent() == Some(&dir) {
        Some(FfmpegLocation::Dir(dir))
    } else {
        None
    }
}

fn emit_progress(
    app: &AppHandle,
    url: &str,
    progress: f32,
    status: &str,
    speed: Option<String>,
    eta: Option<String>,
    error: Option<String>,
) {
    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            url: url.to_string(),
            progress,
            status: status.to_string(),
            speed,
            eta,
            error,
        },
    );
}

fn video_format_selector(resolution: &str, codec: &str) -> String {
    let height_filter = if resolution == "best" || resolution.is_empty() {
        String::new()
    } else {
        format!("[height<={}]", resolution)
    };

    let codec_filter = match codec {
        "h264" => "[vcodec^=avc1]",
        "h265" => "[vcodec^=hev1]",
        "vp9" => "[vcodec^=vp9]",
        "av1" => "[vcodec^=av01]",
        _ => "",
    };

    format!(
        "bv*{h}{c}+ba/b{h}{c}/bv*{h}+ba/b{h}",
        h = height_filter,
        c = codec_filter
    )
}

fn build_command(
    ytdlp: &YtDlpCommand,
    ffmpeg: Option<&FfmpegLocation>,
    url: &str,
    opts: &DownloadOptions,
    downloads_dir: &str,
) -> Command {
    let mut cmd = ytdlp.create();

    if let Some(FfmpegLocation::Dir(dir)) = ffmpeg {
        cmd.arg("--ffmpeg-location").arg(dir);
    }

    let separator = if cfg!(windows) { "\\" } else { "/" };
    let dir = opts
        .output_dir
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(downloads_dir);
    let template = if opts.output_template.trim().is_empty() {
        "%(title)s.%(ext)s"
    } else {
        opts.output_template.trim()
    };
    let output = format!("{}{}{}", dir, separator, template);

    cmd.arg("-o")
        .arg(output)
        .arg("--newline")
        .arg("--no-warnings")
        .arg("--progress-template")
        .arg("PROG|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s");

    // Mode: audio extraction or video download
    let mode = if opts.mode.is_empty() { "audio" } else { opts.mode.as_str() };
    if mode == "audio" {
        let fmt = if opts.audio_format.is_empty() { "mp3" } else { opts.audio_format.as_str() };
        let q = if opts.audio_quality.is_empty() { "0" } else { opts.audio_quality.as_str() };
        cmd.arg("-x")
            .arg("--audio-format").arg(fmt)
            .arg("--audio-quality").arg(q);
    } else {
        let codec = if opts.video_codec.is_empty() { "any" } else { opts.video_codec.as_str() };
        let resolution = if opts.video_resolution.is_empty() { "best" } else { opts.video_resolution.as_str() };
        cmd.arg("-f").arg(video_format_selector(resolution, codec));

        if opts.video_container != "default" && !opts.video_container.is_empty() {
            cmd.arg("--merge-output-format").arg(&opts.video_container);
        }
        if opts.prefer_free_formats {
            cmd.arg("--prefer-free-formats");
        }
    }

    // Embed
    if opts.add_metadata {
        cmd.arg("--add-metadata");
    }
    if opts.embed_thumbnail {
        cmd.arg("--embed-thumbnail");
    }
    if opts.embed_chapters {
        cmd.arg("--embed-chapters");
    }
    if opts.embed_subs {
        cmd.arg("--embed-subs");
    }

    // Subtitles
    if opts.write_subs {
        cmd.arg("--write-subs");
    }
    if opts.write_auto_subs {
        cmd.arg("--write-auto-subs");
    }
    if (opts.write_subs || opts.write_auto_subs || opts.embed_subs) && !opts.sub_langs.trim().is_empty() {
        cmd.arg("--sub-langs").arg(opts.sub_langs.trim());
    }

    // SponsorBlock
    if opts.sponsorblock_remove && !opts.sponsorblock_categories.is_empty() {
        cmd.arg("--sponsorblock-remove")
            .arg(opts.sponsorblock_categories.join(","));
    }

    // Network / performance
    if !opts.limit_rate.trim().is_empty() {
        cmd.arg("--limit-rate").arg(opts.limit_rate.trim());
    }
    if opts.concurrent_fragments >= 2 {
        let n = opts.concurrent_fragments.min(16);
        cmd.arg("-N").arg(n.to_string());
    }
    if opts.retries > 0 && opts.retries != 10 {
        cmd.arg("--retries").arg(opts.retries.to_string());
    }
    if let Some(cookies) = opts.cookies_file.as_deref().filter(|s| !s.trim().is_empty()) {
        cmd.arg("--cookies").arg(cookies);
    }
    if let Some(proxy) = opts.proxy.as_deref().filter(|s| !s.trim().is_empty()) {
        cmd.arg("--proxy").arg(proxy);
    }
    if let Some(ua) = opts.user_agent.as_deref().filter(|s| !s.trim().is_empty()) {
        cmd.arg("--user-agent").arg(ua);
    }

    // Playlist
    if opts.no_playlist {
        cmd.arg("--no-playlist");
    } else if !opts.playlist_items.trim().is_empty() {
        cmd.arg("--playlist-items").arg(opts.playlist_items.trim());
    }

    // Misc
    if opts.write_info_json {
        cmd.arg("--write-info-json");
    }
    if opts.write_description {
        cmd.arg("--write-description");
    }

    // Raw extra args (split on whitespace; advanced power-user escape hatch)
    if !opts.custom_args.trim().is_empty() {
        for a in opts.custom_args.split_whitespace() {
            cmd.arg(a);
        }
    }

    cmd.arg(url).stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd
}

struct ProgressLine {
    percent: f32,
    speed: Option<String>,
    eta: Option<String>,
}

fn parse_progress(line: &str) -> Option<ProgressLine> {
    if let Some(rest) = line.strip_prefix("PROG|") {
        let mut parts = rest.split('|');
        let percent_raw = parts.next()?.trim().trim_end_matches('%');
        let percent = percent_raw.parse::<f32>().ok()?;
        let speed = parts.next().map(|s| s.trim().to_string()).filter(|s| !s.is_empty() && s != "N/A");
        let eta = parts.next().map(|s| s.trim().to_string()).filter(|s| !s.is_empty() && s != "N/A");
        return Some(ProgressLine { percent, speed, eta });
    }

    if line.contains("[download]") && line.contains('%') {
        let percent = line
            .split_whitespace()
            .find(|s| s.ends_with('%'))
            .and_then(|s| s.strip_suffix('%'))
            .and_then(|s| s.parse::<f32>().ok())?;
        return Some(ProgressLine { percent, speed: None, eta: None });
    }

    None
}

async fn perform_download(
    app: &AppHandle,
    ytdlp: &YtDlpCommand,
    ffmpeg: Option<&FfmpegLocation>,
    url: &str,
    opts: &DownloadOptions,
) -> Result<String, String> {
    let downloads_dir = get_downloads_dir()?;

    emit_progress(app, url, 0.0, "Queued", None, None, None);

    let mut cmd = build_command(ytdlp, ffmpeg, url, opts, &downloads_dir);
    cmd.kill_on_drop(true);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start yt-dlp: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let app_clone = app.clone();
    let url_clone = url.to_string();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        let mut last_err = String::new();
        while let Ok(Some(line)) = reader.next_line().await {
            if line.to_lowercase().contains("error") {
                last_err = line;
            }
        }
        if !last_err.is_empty() {
            emit_progress(
                &app_clone,
                &url_clone,
                0.0,
                "Error",
                None,
                None,
                Some(last_err.clone()),
            );
        }
        last_err
    });

    let mut reader = BufReader::new(stdout).lines();
    let mut last_file = String::new();

    while let Ok(Some(line)) = reader.next_line().await {
        if let Some(p) = parse_progress(&line) {
            emit_progress(
                app,
                url,
                p.percent,
                "Downloading",
                p.speed,
                p.eta,
                None,
            );
        } else if line.contains("[ExtractAudio]") {
            emit_progress(app, url, 99.0, "Converting", None, None, None);
        } else if let Some(idx) = line.find("Destination:") {
            let path = line[idx + "Destination:".len()..].trim();
            if !path.is_empty() {
                last_file = path.to_string();
            }
        } else if let Some(idx) = line.find("[download]") {
            if line[idx..].contains("has already been downloaded") {
                emit_progress(app, url, 100.0, "Already downloaded", None, None, None);
            }
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for yt-dlp: {}", e))?;

    let stderr_msg = stderr_task.await.unwrap_or_default();

    if status.success() {
        emit_progress(app, url, 100.0, "Completed", None, None, None);
        Ok(if last_file.is_empty() {
            "Download completed".to_string()
        } else {
            format!("Completed: {}", last_file)
        })
    } else {
        let err = if stderr_msg.is_empty() {
            "Download failed".to_string()
        } else {
            stderr_msg
        };
        emit_progress(app, url, 0.0, "Failed", None, None, Some(err.clone()));
        Err(err)
    }
}

#[tauri::command]
async fn download_video(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
    options: DownloadOptions,
) -> Result<String, String> {
    let cmd = state
        .get_ytdlp()
        .await
        .ok_or_else(|| "yt-dlp is not available".to_string())?;
    let ffmpeg = state.get_ffmpeg().await;
    perform_download(&app, &cmd, ffmpeg.as_ref(), &url, &options).await
}

#[tauri::command]
async fn download_multiple_videos(
    app: AppHandle,
    state: State<'_, AppState>,
    urls: Vec<String>,
    options: DownloadOptions,
) -> Result<Vec<String>, String> {
    let cmd = state
        .get_ytdlp()
        .await
        .ok_or_else(|| "yt-dlp is not available".to_string())?;
    let ffmpeg = state.get_ffmpeg().await;

    let semaphore = Arc::new(Semaphore::new(MAX_PARALLEL_DOWNLOADS));
    let mut handles = Vec::with_capacity(urls.len());

    for url in urls {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let app = app.clone();
        let cmd = cmd.clone();
        let ffmpeg = ffmpeg.clone();
        let opts = options.clone();
        handles.push(tokio::spawn(async move {
            let _permit = permit;
            (
                url.clone(),
                perform_download(&app, &cmd, ffmpeg.as_ref(), &url, &opts).await,
            )
        }));
    }

    let mut results = Vec::with_capacity(handles.len());
    for h in handles {
        match h.await {
            Ok((_, Ok(msg))) => results.push(msg),
            Ok((url, Err(e))) => results.push(format!("Error for {}: {}", url, e)),
            Err(e) => results.push(format!("Task error: {}", e)),
        }
    }

    Ok(results)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct YtDlpStatus {
    available: bool,
    source: Option<String>,
    ffmpeg_available: bool,
    ffmpeg_source: Option<String>,
}

async fn build_status(state: &AppState) -> YtDlpStatus {
    let ytdlp = state.get_ytdlp().await;
    let ffmpeg = state.get_ffmpeg().await;
    YtDlpStatus {
        available: ytdlp.is_some(),
        source: ytdlp.map(|c| c.label()),
        ffmpeg_available: ffmpeg.is_some(),
        ffmpeg_source: ffmpeg.map(|f| f.label()),
    }
}

#[tauri::command]
async fn check_ytdlp_status(state: State<'_, AppState>) -> Result<YtDlpStatus, String> {
    Ok(build_status(&state).await)
}

#[tauri::command]
async fn redetect_ytdlp(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<YtDlpStatus, String> {
    let app_handle = app.clone();
    let detected = tokio::task::spawn_blocking(move || {
        let ytdlp = detect_ytdlp(&app_handle);
        let ffmpeg = detect_ffmpeg(&app_handle);
        (ytdlp, ffmpeg)
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Some(cmd) = detected.0 {
        state.set_ytdlp(cmd).await;
    } else {
        *state.ytdlp.write().await = None;
    }
    if let Some(loc) = detected.1 {
        state.set_ffmpeg(loc).await;
    } else {
        *state.ffmpeg.write().await = None;
    }

    Ok(build_status(&state).await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let state: State<AppState> = handle.state();
            let ytdlp_state = state.ytdlp.clone();
            let ffmpeg_state = state.ffmpeg.clone();
            tauri::async_runtime::spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    (detect_ytdlp(&handle), detect_ffmpeg(&handle))
                })
                .await
                .ok();
                if let Some((ytdlp, ffmpeg)) = result {
                    if let Some(cmd) = ytdlp {
                        *ytdlp_state.write().await = Some(cmd);
                    }
                    if let Some(loc) = ffmpeg {
                        *ffmpeg_state.write().await = Some(loc);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            download_video,
            download_multiple_videos,
            check_ytdlp_status,
            redetect_ytdlp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_downloads_dir() {
        let result = get_downloads_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("Downloads"));
    }

    #[test]
    fn test_parse_progress_template() {
        let p = parse_progress("PROG| 42.5%|1.20MiB/s|00:30").unwrap();
        assert_eq!(p.percent, 42.5);
        assert_eq!(p.speed.as_deref(), Some("1.20MiB/s"));
        assert_eq!(p.eta.as_deref(), Some("00:30"));
    }

    #[test]
    fn test_parse_progress_legacy() {
        let p = parse_progress("[download]  50.0% of 1.00MiB at 100KiB/s ETA 00:01").unwrap();
        assert_eq!(p.percent, 50.0);
        assert!(p.speed.is_none());
    }

    #[test]
    fn test_parse_progress_none() {
        assert!(parse_progress("no match here").is_none());
    }

    #[test]
    fn test_target_triple_non_empty() {
        assert!(!target_triple().is_empty());
    }

    #[test]
    fn test_options_deserialization_partial() {
        // Tests that #[serde(default)] lets the frontend send partial payloads.
        let json = r#"{
            "audioFormat": "mp3",
            "audioQuality": "0",
            "outputTemplate": "%(title)s.%(ext)s",
            "embedThumbnail": true,
            "addMetadata": true
        }"#;
        let opts: DownloadOptions = serde_json::from_str(json).unwrap();
        assert_eq!(opts.audio_format, "mp3");
        assert!(opts.embed_thumbnail);
        assert!(opts.sponsorblock_categories.is_empty());
        assert_eq!(opts.concurrent_fragments, 0);
    }

    #[test]
    fn test_video_format_selector() {
        assert_eq!(
            video_format_selector("best", "any"),
            "bv*+ba/b/bv*+ba/b"
        );
        assert!(video_format_selector("1080", "any").contains("[height<=1080]"));
        assert!(video_format_selector("720", "h264").contains("[vcodec^=avc1]"));
        assert!(video_format_selector("1080", "av1").contains("[vcodec^=av01]"));
    }
}
