export interface DownloadProgress {
  url: string;
  progress: number;
  status: string;
  speed?: string | null;
  eta?: string | null;
  error?: string | null;
}

export interface VideoItem {
  url: string;
  progress: number;
  status: string;
  speed?: string | null;
  eta?: string | null;
  error?: string | null;
  thumbnail?: string | null;
}

export type DownloadMode = "audio" | "video";

export type SponsorBlockCategory =
  | "sponsor"
  | "intro"
  | "outro"
  | "selfpromo"
  | "interaction"
  | "music_offtopic"
  | "preview"
  | "filler";

export interface DownloadOptions {
  // Mode
  mode: DownloadMode;

  // Output
  outputDir: string | null;
  outputTemplate: string;

  // Audio
  audioFormat: string;
  audioQuality: string;

  // Video
  videoResolution: string;
  videoContainer: string;
  videoCodec: string;
  preferFreeFormats: boolean;

  // Embed
  embedThumbnail: boolean;
  addMetadata: boolean;
  embedChapters: boolean;
  embedSubs: boolean;

  // Subtitles
  writeSubs: boolean;
  writeAutoSubs: boolean;
  subLangs: string;

  // SponsorBlock
  sponsorblockRemove: boolean;
  sponsorblockCategories: SponsorBlockCategory[];

  // Network
  limitRate: string;
  concurrentFragments: number;
  retries: number;
  cookiesFile: string | null;
  proxy: string | null;
  userAgent: string | null;

  // Playlist
  noPlaylist: boolean;
  playlistItems: string;

  // Misc
  writeInfoJson: boolean;
  writeDescription: boolean;
  customArgs: string;
}

export interface YtDlpStatus {
  available: boolean;
  source?: string | null;
  ffmpegAvailable: boolean;
  ffmpegSource?: string | null;
}
