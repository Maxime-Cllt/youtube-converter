import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Download, X, Music, Loader2, AlertCircle, Settings, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Select } from "@/components/ui/select";
import { Label } from "@/components/ui/label";

interface DownloadProgress {
  url: string;
  progress: number;
  status: string;
  error?: string;
}

interface VideoItem {
  url: string;
  progress: number;
  status: string;
  error?: string;
}

interface DownloadOptions {
  audioFormat: string;
  audioQuality: string;
  outputTemplate: string;
  embedThumbnail: boolean;
  addMetadata: boolean;
}

function App() {
  const [urlInput, setUrlInput] = useState("");
  const [videos, setVideos] = useState<VideoItem[]>([]);
  const [isDownloading, setIsDownloading] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [ytdlpInstalled, setYtdlpInstalled] = useState(true);
  const [options, setOptions] = useState<DownloadOptions>({
    audioFormat: "mp3",
    audioQuality: "0",
    outputTemplate: "%(title)s.%(ext)s",
    embedThumbnail: true,
    addMetadata: true,
  });

  useEffect(() => {
    // Check if yt-dlp is installed
    invoke<boolean>("check_ytdlp_installed").then((installed) => {
      setYtdlpInstalled(installed);
    });

    const unlisten = listen<DownloadProgress>("download-progress", (event) => {
      const { url, progress, status, error } = event.payload;

      setVideos((prev) =>
        prev.map((video) =>
          video.url === url
            ? { ...video, progress, status, error }
            : video
        )
      );
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const addUrl = () => {
    const url = urlInput.trim();
    if (!url) return;

    if (!url.includes("youtube.com") && !url.includes("youtu.be")) {
      alert("Please enter a valid YouTube URL");
      return;
    }

    if (videos.some((v) => v.url === url)) {
      alert("This URL has already been added");
      return;
    }

    setVideos([
      ...videos,
      {
        url,
        progress: 0,
        status: "Pending",
        error: undefined,
      },
    ]);
    setUrlInput("");
  };

  const removeUrl = (url: string) => {
    setVideos(videos.filter((v) => v.url !== url));
  };

  const downloadAll = async () => {
    if (videos.length === 0) {
      alert("Please add at least one URL");
      return;
    }

    setIsDownloading(true);

    try {
      const urls = videos.map((v) => v.url);
      await invoke("download_multiple_videos", { urls, options });
    } catch (error) {
      console.error("Download error:", error);
      alert(`Error: ${error}`);
    } finally {
      setIsDownloading(false);
    }
  };

  const clearAll = () => {
    setVideos([]);
  };

  return (
    <div className="min-h-screen relative overflow-hidden">
      {/* Animated background */}
      <div className="fixed inset-0 -z-10">
        <div className="absolute inset-0 bg-gradient-to-br from-black via-gray-950 to-black" />
        <div className="absolute top-0 left-1/4 w-96 h-96 bg-red-500/10 rounded-full blur-3xl animate-pulse" />
        <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-pink-500/10 rounded-full blur-3xl animate-pulse delay-700" />
      </div>

      <div className="container mx-auto max-w-5xl p-6 space-y-8 relative">
        {/* Header */}
        <div className="text-center space-y-4 pt-12 pb-8">
          <div className="flex items-center justify-center gap-4">
            <div className="relative">
              <div className="absolute inset-0 bg-gradient-to-r from-red-500 to-pink-600 rounded-full blur-xl opacity-50" />
              <Music className="h-14 w-14 text-red-500 relative animate-pulse" />
            </div>
            <h1 className="text-6xl font-black tracking-tight">
              <span className="bg-gradient-to-r from-red-500 via-pink-500 to-red-600 bg-clip-text text-transparent">
                YouTube
              </span>
              <span className="text-white"> to </span>
              <span className="bg-gradient-to-r from-red-500 via-pink-500 to-red-600 bg-clip-text text-transparent">
                MP3
              </span>
            </h1>
          </div>
          <p className="text-gray-400 text-lg font-medium">
            Transform YouTube videos into high-quality audio files
          </p>
        </div>

        {/* URL Input Section */}
        <Card className="glass-strong animate-in">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <CardTitle className="flex items-center gap-2">
                  <Sparkles className="h-5 w-5 text-red-500" />
                  Add YouTube URL
                </CardTitle>
                <CardDescription>Paste your YouTube video URL to get started</CardDescription>
              </div>
              <Button
                variant="outline"
                size="icon"
                onClick={() => setSettingsOpen(true)}
                className="shrink-0"
                title="Settings"
              >
                <Settings className="h-4 w-4" />
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <div className="flex gap-3">
              <Input
                type="text"
                value={urlInput}
                onChange={(e) => setUrlInput(e.target.value)}
                onKeyPress={(e) => e.key === "Enter" && addUrl()}
                placeholder="https://www.youtube.com/watch?v=..."
                disabled={isDownloading}
                className="flex-1"
              />
              <Button onClick={addUrl} disabled={isDownloading} size="lg">
                <Download className="h-4 w-4 mr-2" />
                Add
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Video List */}
        {videos.length > 0 && (
          <Card className="glass-strong animate-in">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <CardTitle>Download Queue</CardTitle>
                  <CardDescription>
                    {videos.length} video{videos.length !== 1 ? 's' : ''} ready to download
                  </CardDescription>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={clearAll}
                  disabled={isDownloading}
                >
                  Clear All
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              {videos.map((video, index) => (
                <div
                  key={index}
                  className="group relative rounded-2xl border-2 border-white/5 bg-white/5 backdrop-blur-xl p-5 transition-all duration-300 hover:border-red-500/30 hover:bg-white/10"
                >
                  <div className="flex items-start gap-4">
                    <div className="flex-1 space-y-3">
                      <div className="flex items-start justify-between gap-3">
                        <p className="text-sm font-medium text-gray-200 line-clamp-2 flex-1 leading-relaxed">
                          {video.url}
                        </p>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => removeUrl(video.url)}
                          disabled={isDownloading}
                          className="h-8 w-8 shrink-0 rounded-lg"
                        >
                          <X className="h-4 w-4" />
                        </Button>
                      </div>

                      <div className="flex items-center gap-2 text-sm">
                        {video.error ? (
                          <div className="flex items-center gap-2 text-red-400 font-medium">
                            <AlertCircle className="h-4 w-4" />
                            <span>{video.error}</span>
                          </div>
                        ) : video.status === "Downloading..." || video.progress > 0 && video.progress < 100 ? (
                          <div className="flex items-center gap-2 text-blue-400 font-medium">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            <span>{video.status}</span>
                          </div>
                        ) : video.status === "Completed" ? (
                          <span className="text-green-400 font-semibold flex items-center gap-1">
                            ✓ {video.status}
                          </span>
                        ) : (
                          <span className="text-gray-400 font-medium">{video.status}</span>
                        )}
                      </div>

                      <div className="flex items-center gap-4">
                        <Progress value={video.progress} className="flex-1 h-2" />
                        <span className="text-xs font-bold text-gray-300 w-12 text-right tabular-nums">
                          {Math.round(video.progress)}%
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              ))}

              <Button
                onClick={downloadAll}
                disabled={isDownloading}
                className="w-full mt-6"
                size="lg"
              >
                {isDownloading ? (
                  <>
                    <Loader2 className="h-5 w-5 mr-2 animate-spin" />
                    Downloading...
                  </>
                ) : (
                  <>
                    <Download className="h-5 w-5 mr-2" />
                    Download All ({videos.length})
                  </>
                )}
              </Button>
            </CardContent>
          </Card>
        )}

        {/* Empty State */}
        {videos.length === 0 && (
          <Card className="glass border-dashed border-2 animate-in">
            <CardContent className="flex flex-col items-center justify-center py-20 text-center">
              <div className="relative mb-6">
                <div className="absolute inset-0 bg-red-500/20 rounded-full blur-2xl" />
                <div className="relative rounded-full bg-white/5 p-6">
                  <Music className="h-12 w-12 text-gray-500" />
                </div>
              </div>
              <h3 className="text-xl font-bold mb-2 text-white">No videos added yet</h3>
              <p className="text-sm text-gray-400 max-w-sm">
                Start by pasting a YouTube URL above to begin downloading
              </p>
            </CardContent>
          </Card>
        )}

        {/* Footer */}
        <div className="text-center space-y-2 text-sm text-gray-500 pb-12">
          <p className="font-medium">Files will be saved to your Downloads folder</p>
          {!ytdlpInstalled && (
            <p className="text-red-400 font-semibold flex items-center justify-center gap-2">
              <AlertCircle className="h-4 w-4" />
              yt-dlp is not installed. Please install it from{" "}
              <a
                href="https://github.com/yt-dlp/yt-dlp#installation"
                target="_blank"
                rel="noopener noreferrer"
                className="underline hover:text-red-300 transition-colors"
              >
                here
              </a>
            </p>
          )}
        </div>
      </div>

      {/* Settings Dialog */}
      <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-3">
              <Settings className="h-6 w-6 text-red-500" />
              Download Settings
            </DialogTitle>
            <DialogDescription>
              Customize your download preferences for optimal quality
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-6 py-6">
            <div className="grid grid-cols-2 gap-6">
              <div className="space-y-3">
                <Label htmlFor="format" className="text-sm font-semibold text-gray-300">
                  Audio Format
                </Label>
                <Select
                  id="format"
                  options={[
                    { value: "mp3", label: "MP3" },
                    { value: "m4a", label: "M4A" },
                    { value: "opus", label: "Opus" },
                    { value: "vorbis", label: "Vorbis" },
                    { value: "wav", label: "WAV" },
                    { value: "flac", label: "FLAC" },
                  ]}
                  value={options.audioFormat}
                  onChange={(e) => setOptions({ ...options, audioFormat: e.target.value })}
                />
              </div>

              <div className="space-y-3">
                <Label htmlFor="quality" className="text-sm font-semibold text-gray-300">
                  Audio Quality
                </Label>
                <Select
                  id="quality"
                  options={[
                    { value: "0", label: "Best (320kbps)" },
                    { value: "2", label: "High (256kbps)" },
                    { value: "5", label: "Medium (192kbps)" },
                    { value: "7", label: "Low (128kbps)" },
                    { value: "9", label: "Lowest (64kbps)" },
                  ]}
                  value={options.audioQuality}
                  onChange={(e) => setOptions({ ...options, audioQuality: e.target.value })}
                />
              </div>
            </div>

            <div className="space-y-3">
              <Label htmlFor="template" className="text-sm font-semibold text-gray-300">
                Output Filename Template
              </Label>
              <Input
                id="template"
                value={options.outputTemplate}
                onChange={(e) => setOptions({ ...options, outputTemplate: e.target.value })}
                placeholder="%(title)s.%(ext)s"
              />
              <p className="text-xs text-gray-500 mt-1">
                Use %(title)s for video title, %(uploader)s for channel name
              </p>
            </div>

            <div className="space-y-4 pt-4 border-t border-white/10">
              <div className="flex items-center space-x-3 group">
                <input
                  type="checkbox"
                  id="thumbnail"
                  checked={options.embedThumbnail}
                  onChange={(e) => setOptions({ ...options, embedThumbnail: e.target.checked })}
                  className="h-5 w-5 rounded-lg border-2 border-white/20 bg-white/5 text-red-500 focus:ring-2 focus:ring-red-500/50 cursor-pointer transition-all"
                />
                <Label htmlFor="thumbnail" className="cursor-pointer font-medium text-gray-300 group-hover:text-white transition-colors">
                  Embed thumbnail in audio file
                </Label>
              </div>

              <div className="flex items-center space-x-3 group">
                <input
                  type="checkbox"
                  id="metadata"
                  checked={options.addMetadata}
                  onChange={(e) => setOptions({ ...options, addMetadata: e.target.checked })}
                  className="h-5 w-5 rounded-lg border-2 border-white/20 bg-white/5 text-red-500 focus:ring-2 focus:ring-red-500/50 cursor-pointer transition-all"
                />
                <Label htmlFor="metadata" className="cursor-pointer font-medium text-gray-300 group-hover:text-white transition-colors">
                  Add metadata (title, artist, album)
                </Label>
              </div>
            </div>
          </div>

          <div className="flex justify-end gap-3 pt-4 border-t border-white/10">
            <Button variant="outline" onClick={() => setSettingsOpen(false)}>
              Close
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default App;
