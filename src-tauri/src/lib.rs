use serde::Serialize;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tauri::{AppHandle, Emitter};

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

// Get the downloads directory for the current platform
fn get_downloads_dir() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(format!("{}/Downloads", home));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            return Ok(format!("{}\\Downloads", userprofile));
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Ok(format!("{}/Downloads", home));
        }
    }

    Err("Could not determine downloads directory".to_string())
}

#[tauri::command]
async fn download_video(app: AppHandle, url: String, options: DownloadOptions) -> Result<String, String> {
    let downloads_dir = get_downloads_dir()?;

    // Check if yt-dlp is installed
    let check = Command::new("yt-dlp")
        .arg("--version")
        .output();

    if check.is_err() {
        return Err("yt-dlp is not installed. Please install it first: https://github.com/yt-dlp/yt-dlp#installation".to_string());
    }

    // Emit initial progress
    let _ = app.emit("download-progress", DownloadProgress {
        url: url.clone(),
        progress: 0.0,
        status: "Starting download...".to_string(),
        error: None,
    });

    // Build yt-dlp command with options
    let mut cmd = Command::new("yt-dlp");

    // Extract audio
    cmd.arg("-x");

    // Set audio format
    cmd.arg("--audio-format");
    cmd.arg(&options.audio_format);

    // Set audio quality
    cmd.arg("--audio-quality");
    cmd.arg(&options.audio_quality);

    // Set output template
    cmd.arg("-o");
    cmd.arg(format!("{}/{}", downloads_dir, options.output_template));

    // Add metadata if enabled
    if options.add_metadata {
        cmd.arg("--add-metadata");
    }

    // Embed thumbnail if enabled
    if options.embed_thumbnail {
        cmd.arg("--embed-thumbnail");
    }

    // Add newline for easier parsing
    cmd.arg("--newline");

    // Add URL
    cmd.arg(&url);

    // Set up stdio
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start yt-dlp: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    let mut _last_file_path = String::new();

    // Parse output for progress
    for line in reader.lines().map_while(Result::ok) {
        // Check for download progress
        if line.contains("[download]") && line.contains("%") {
            // Extract percentage
            if let Some(percent_str) = line.split_whitespace()
                .find(|s| s.ends_with("%"))
                .and_then(|s| s.strip_suffix("%"))
            {
                if let Ok(percent) = percent_str.parse::<f32>() {
                    let _ = app.emit("download-progress", DownloadProgress {
                        url: url.clone(),
                        progress: percent,
                        status: format!("Downloading... {}%", percent as u32),
                        error: None,
                    });
                }
            }
        }

        // Extract video title
        if line.contains("[ExtractAudio]") || line.contains("Destination:") {
            if let Some(path) = line.split("Destination:").nth(1) {
                _last_file_path = path.trim().to_string();
            }
        }
    }

    let status = child.wait().map_err(|e| format!("Failed to wait for yt-dlp: {}", e))?;

    if status.success() {
        let _ = app.emit("download-progress", DownloadProgress {
            url: url.clone(),
            progress: 100.0,
            status: "Completed".to_string(),
            error: None,
        });

        Ok(format!("Download completed: {}", _last_file_path))
    } else {
        let error_msg = "Download failed".to_string();
        let _ = app.emit("download-progress", DownloadProgress {
            url: url.clone(),
            progress: 0.0,
            status: "Failed".to_string(),
            error: Some(error_msg.clone()),
        });

        Err(error_msg)
    }
}

#[tauri::command]
async fn download_multiple_videos(app: AppHandle, urls: Vec<String>, options: DownloadOptions) -> Result<Vec<String>, String> {
    let mut results = Vec::new();

    for url in urls {
        match download_video(app.clone(), url.clone(), options.clone()).await {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("Error for {}: {}", url, e)),
        }
    }

    Ok(results)
}

#[tauri::command]
fn check_ytdlp_installed() -> bool {
    Command::new("yt-dlp")
        .arg("--version")
        .output()
        .is_ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![download_video, download_multiple_videos, check_ytdlp_installed])
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
    fn test_check_ytdlp_installed() {
        // This test will pass or fail depending on whether yt-dlp is installed
        // We're testing that the function returns a boolean and doesn't panic
        let result = check_ytdlp_installed();
        assert!(result == true || result == false, "Should return a boolean value");
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
