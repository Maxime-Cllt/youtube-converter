import type { DownloadOptions } from "$lib/types";

const STORAGE_KEY = "ytconv:options:v2";

export const DEFAULTS: DownloadOptions = {
  mode: "audio",

  outputDir: null,
  outputTemplate: "%(title)s.%(ext)s",

  audioFormat: "mp3",
  audioQuality: "0",

  videoResolution: "1080",
  videoContainer: "mp4",
  videoCodec: "any",
  preferFreeFormats: false,

  embedThumbnail: true,
  addMetadata: true,
  embedChapters: false,
  embedSubs: false,

  writeSubs: false,
  writeAutoSubs: false,
  subLangs: "en",

  sponsorblockRemove: false,
  sponsorblockCategories: ["sponsor", "selfpromo", "interaction"],

  limitRate: "",
  concurrentFragments: 4,
  retries: 10,
  cookiesFile: null,
  proxy: null,
  userAgent: null,

  noPlaylist: true,
  playlistItems: "",

  writeInfoJson: false,
  writeDescription: false,
  customArgs: "",
};

function load(): DownloadOptions {
  if (typeof localStorage === "undefined") return { ...DEFAULTS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<DownloadOptions>;
    return { ...DEFAULTS, ...parsed };
  } catch {
    return { ...DEFAULTS };
  }
}

function createOptions() {
  let state = $state<DownloadOptions>(load());

  function persist(next: DownloadOptions) {
    state = next;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      /* quota / serialization errors silently ignored */
    }
  }

  return {
    get current() {
      return state;
    },
    set(next: DownloadOptions) {
      persist(next);
    },
    update(patch: Partial<DownloadOptions>) {
      persist({ ...state, ...patch });
    },
    reset() {
      persist({ ...DEFAULTS });
    },
  };
}

export const options = createOptions();
