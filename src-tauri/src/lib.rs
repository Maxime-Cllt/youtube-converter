use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

// ===== Data Structures =====

#[derive(Clone, Serialize)]
struct DownloadProgress {
    url: String,
    progress: f32,
    status: String,
    error: Option<String>,
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DownloadOptions {
    audio_format: String,
    audio_quality: String,
    output_template: String,
    embed_thumbnail: bool,
    add_metadata: bool,
}

#[derive(Clone, Debug)]
enum YtDlpCommand {
    Direct(String),
    PythonModule(String),
}

impl YtDlpCommand {
    fn create_command(&self) -> Command {
        match self {
            YtDlpCommand::Direct(path) => Command::new(path),
            YtDlpCommand::PythonModule(python_cmd) => {
                let mut cmd = Command::new(python_cmd);
                cmd.arg("-m").arg("yt_dlp");
                cmd
            }
        }
    }
}

// Application state to store yt-dlp command path
struct AppState {
    ytdlp_command: Arc<Mutex<Option<YtDlpCommand>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            ytdlp_command: Arc::new(Mutex::new(None)),
        }
    }

    fn set_ytdlp_command(&self, cmd: YtDlpCommand) {
        if let Ok(mut guard) = self.ytdlp_command.lock() {
            *guard = Some(cmd);
        }
    }

    fn get_ytdlp_command(&self) -> Option<YtDlpCommand> {
        self.ytdlp_command.lock().ok()?.clone()
    }
}

// ===== Utility Functions =====

fn get_downloads_dir() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .map(|profile| format!("{}\\Downloads", profile))
            .map_err(|_| "Could not determine Windows downloads directory".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME")
            .map(|home| format!("{}/Downloads", home))
            .map_err(|_| "Could not determine downloads directory".to_string())
    }
}

fn try_command(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn find_ytdlp() -> Option<YtDlpCommand> {
    // Try direct commands in PATH
    let direct_commands = if cfg!(target_os = "windows") {
        vec!["yt-dlp.exe", "yt-dlp"]
    } else {
        vec!["yt-dlp"]
    };

    for cmd in &direct_commands {
        if try_command(cmd, &["--version"]) {
            return Some(YtDlpCommand::Direct(cmd.to_string()));
        }
    }

    // Try common installation paths
    let common_paths = if cfg!(target_os = "windows") {
        vec![
            "C:\\Program Files\\yt-dlp\\yt-dlp.exe",
            "C:\\Python3\\Scripts\\yt-dlp.exe",
            "C:\\Python\\Scripts\\yt-dlp.exe",
        ]
    } else {
        vec![
            "/usr/local/bin/yt-dlp",
            "/usr/bin/yt-dlp",
            "/opt/homebrew/bin/yt-dlp",
        ]
    };

    for path in &common_paths {
        if try_command(path, &["--version"]) {
            return Some(YtDlpCommand::Direct(path.to_string()));
        }
    }

    // Try user home directory paths (Unix-like systems)
    #[cfg(not(target_os = "windows"))]
    if let Ok(home) = std::env::var("HOME") {
        let home_paths = vec![
            format!("{}/.local/bin/yt-dlp", home),
            format!("{}/bin/yt-dlp", home),
        ];
        for path in home_paths {
            if try_command(&path, &["--version"]) {
                return Some(YtDlpCommand::Direct(path));
            }
        }
    }

    // Try Python module variants
    for python_cmd in &["python3", "python", "py"] {
        if try_command(python_cmd, &["-m", "yt_dlp", "--version"]) {
            return Some(YtDlpCommand::PythonModule(python_cmd.to_string()));
        }
    }

    // Try 'which' or 'where' command as last resort
    #[cfg(not(target_os = "windows"))]
    if let Ok(output) = Command::new("which").arg("yt-dlp").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                let path = path.trim();
                if !path.is_empty() {
                    return Some(YtDlpCommand::Direct(path.to_string()));
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(output) = Command::new("where").arg("yt-dlp").output() {
        if output.status.success() {
            if let Ok(path) = String::from_utf8(output.stdout) {
                if let Some(first_line) = path.lines().next() {
                    let path = first_line.trim();
                    if !path.is_empty() {
                        return Some(YtDlpCommand::Direct(path.to_string()));
                    }
                }
            }
        }
    }

    None
}

// ===== Tauri Commands =====

fn emit_progress(app: &AppHandle, url: &str, progress: f32, status: String, error: Option<String>) {
    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            url: url.to_string(),
            progress,
            status,
            error,
        },
    );
}

fn build_ytdlp_command(
    ytdlp_cmd: &YtDlpCommand,
    url: &str,
    options: &DownloadOptions,
    downloads_dir: &str,
) -> Command {
    let mut cmd = ytdlp_cmd.create_command();

    cmd.arg("-x")
        .arg("--audio-format")
        .arg(&options.audio_format)
        .arg("--audio-quality")
        .arg(&options.audio_quality)
        .arg("-o")
        .arg(format!("{}/{}", downloads_dir, options.output_template))
        .arg("--newline");

    if options.add_metadata {
        cmd.arg("--add-metadata");
    }

    if options.embed_thumbnail {
        cmd.arg("--embed-thumbnail");
    }

    cmd.arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    cmd
}

fn parse_progress_line(line: &str) -> Option<f32> {
    if line.contains("[download]") && line.contains("%") {
        line.split_whitespace()
            .find(|s| s.ends_with("%"))
            .and_then(|s| s.strip_suffix("%"))
            .and_then(|s| s.parse::<f32>().ok())
    } else {
        None
    }
}

async fn perform_download(
    app: &AppHandle,
    ytdlp_cmd: &YtDlpCommand,
    url: &str,
    options: &DownloadOptions,
) -> Result<String, String> {
    let downloads_dir = get_downloads_dir()?;

    emit_progress(app, url, 0.0, "Starting download...".to_string(), None);

    let mut cmd = build_ytdlp_command(ytdlp_cmd, url, options, &downloads_dir);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start yt-dlp: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    let mut last_file_path = String::new();

    for line in reader.lines().map_while(Result::ok) {
        if let Some(percent) = parse_progress_line(&line) {
            emit_progress(
                app,
                url,
                percent,
                format!("Downloading... {}%", percent as u32),
                None,
            );
        }

        if line.contains("Destination:") {
            if let Some(path) = line.split("Destination:").nth(1) {
                last_file_path = path.trim().to_string();
            }
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for yt-dlp: {}", e))?;

    if status.success() {
        emit_progress(app, url, 100.0, "Completed".to_string(), None);
        Ok(format!("Download completed: {}", last_file_path))
    } else {
        let error_msg = "Download failed".to_string();
        emit_progress(app, url, 0.0, "Failed".to_string(), Some(error_msg.clone()));
        Err(error_msg)
    }
}

#[tauri::command]
async fn download_video(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
    options: DownloadOptions,
) -> Result<String, String> {
    let ytdlp_cmd = state.get_ytdlp_command().ok_or_else(|| {
        "yt-dlp is not available. Please ensure it's installed: https://github.com/yt-dlp/yt-dlp#installation".to_string()
    })?;

    perform_download(&app, &ytdlp_cmd, &url, &options).await
}

#[tauri::command]
async fn download_multiple_videos(
    app: AppHandle,
    state: State<'_, AppState>,
    urls: Vec<String>,
    options: DownloadOptions,
) -> Result<Vec<String>, String> {
    let ytdlp_cmd = state.get_ytdlp_command().ok_or_else(|| {
        "yt-dlp is not available. Please ensure it's installed: https://github.com/yt-dlp/yt-dlp#installation".to_string()
    })?;

    let mut results = Vec::new();

    for url in urls {
        match perform_download(&app, &ytdlp_cmd, &url, &options).await {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Error for {}: {}", url, e)),
        }
    }

    Ok(results)
}

#[tauri::command]
fn check_ytdlp_installed(state: State<'_, AppState>) -> bool {
    state.get_ytdlp_command().is_some()
}

// ===== Application Setup =====

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize application state
    let app_state = AppState::new();

    // Search for yt-dlp at startup
    if let Some(ytdlp_cmd) = find_ytdlp() {
        app_state.set_ytdlp_command(ytdlp_cmd);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            download_video,
            download_multiple_videos,
            check_ytdlp_installed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_downloads_dir_returns_valid_path() {
        let result = get_downloads_dir();
        assert!(result.is_ok(), "Should return a valid downloads directory");

        let path = result.unwrap();
        assert!(!path.is_empty(), "Path should not be empty");
        assert!(path.contains("Downloads"), "Path should contain 'Downloads'");
    }

    #[test]
    fn test_get_downloads_dir_format() {
        let result = get_downloads_dir();
        if let Ok(path) = result {
            #[cfg(target_os = "macos")]
            {
                assert!(path.starts_with("/") || path.starts_with("~"), "macOS path should start with / or ~");
                assert!(path.ends_with("Downloads"), "macOS path should end with Downloads");
            }

            #[cfg(target_os = "windows")]
            {
                assert!(path.contains(":\\"), "Windows path should contain drive letter");
                assert!(path.ends_with("Downloads"), "Windows path should end with Downloads");
            }

            #[cfg(target_os = "linux")]
            {
                assert!(path.starts_with("/"), "Linux path should start with /");
                assert!(path.ends_with("Downloads"), "Linux path should end with Downloads");
            }
        }
    }

    #[test]
    fn test_find_ytdlp() {
        // This test checks that find_ytdlp runs without panicking
        // The result depends on whether yt-dlp is installed on the system
        let result = find_ytdlp();
        // Test passes if it returns Some or None without panicking
        assert!(result.is_some() || result.is_none(), "Should return an Option value");
    }

    #[test]
    fn test_download_options_deserialization() {
        let json = r#"{
            "audioFormat": "mp3",
            "audioQuality": "0",
            "outputTemplate": "%(title)s.%(ext)s",
            "embedThumbnail": true,
            "addMetadata": true
        }"#;

        let options: Result<DownloadOptions, _> = serde_json::from_str(json);
        assert!(options.is_ok(), "Should deserialize valid JSON");

        let options = options.unwrap();
        assert_eq!(options.audio_format, "mp3");
        assert_eq!(options.audio_quality, "0");
        assert_eq!(options.output_template, "%(title)s.%(ext)s");
        assert_eq!(options.embed_thumbnail, true);
        assert_eq!(options.add_metadata, true);
    }

    #[test]
    fn test_download_options_with_false_flags() {
        let json = r#"{
            "audioFormat": "m4a",
            "audioQuality": "5",
            "outputTemplate": "%(artist)s - %(title)s.%(ext)s",
            "embedThumbnail": false,
            "addMetadata": false
        }"#;

        let options: Result<DownloadOptions, _> = serde_json::from_str(json);
        assert!(options.is_ok(), "Should deserialize valid JSON");

        let options = options.unwrap();
        assert_eq!(options.audio_format, "m4a");
        assert_eq!(options.audio_quality, "5");
        assert_eq!(options.embed_thumbnail, false);
        assert_eq!(options.add_metadata, false);
    }

    #[test]
    fn test_download_progress_serialization() {
        let progress = DownloadProgress {
            url: "https://www.youtube.com/watch?v=test".to_string(),
            progress: 50.5,
            status: "Downloading...".to_string(),
            error: None,
        };

        let json = serde_json::to_string(&progress);
        assert!(json.is_ok(), "Should serialize DownloadProgress");

        let json_str = json.unwrap();
        assert!(json_str.contains("youtube.com"));
        assert!(json_str.contains("50.5"));
        assert!(json_str.contains("Downloading..."));
    }

    #[test]
    fn test_download_progress_with_error() {
        let progress = DownloadProgress {
            url: "https://www.youtube.com/watch?v=test".to_string(),
            progress: 0.0,
            status: "Failed".to_string(),
            error: Some("Network error".to_string()),
        };

        let json = serde_json::to_string(&progress);
        assert!(json.is_ok(), "Should serialize DownloadProgress with error");

        let json_str = json.unwrap();
        assert!(json_str.contains("Failed"));
        assert!(json_str.contains("Network error"));
    }

    #[test]
    fn test_download_options_clone() {
        let options = DownloadOptions {
            audio_format: "mp3".to_string(),
            audio_quality: "0".to_string(),
            output_template: "%(title)s.%(ext)s".to_string(),
            embed_thumbnail: true,
            add_metadata: true,
        };

        let cloned = options.clone();
        assert_eq!(cloned.audio_format, options.audio_format);
        assert_eq!(cloned.audio_quality, options.audio_quality);
        assert_eq!(cloned.output_template, options.output_template);
        assert_eq!(cloned.embed_thumbnail, options.embed_thumbnail);
        assert_eq!(cloned.add_metadata, options.add_metadata);
    }

    #[test]
    fn test_download_progress_clone() {
        let progress = DownloadProgress {
            url: "https://test.com".to_string(),
            progress: 25.0,
            status: "In progress".to_string(),
            error: None,
        };

        let cloned = progress.clone();
        assert_eq!(cloned.url, progress.url);
        assert_eq!(cloned.progress, progress.progress);
        assert_eq!(cloned.status, progress.status);
        assert_eq!(cloned.error, progress.error);
    }
}
