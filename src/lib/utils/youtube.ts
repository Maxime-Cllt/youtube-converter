const YT_PATTERNS = [
  /^https?:\/\/(www\.|m\.|music\.)?youtube\.com\/(watch\?v=|shorts\/|playlist\?list=|embed\/)/i,
  /^https?:\/\/youtu\.be\/[\w-]+/i,
];

export function isValidYoutubeUrl(url: string): boolean {
  const trimmed = url.trim();
  if (!trimmed) return false;
  return YT_PATTERNS.some((p) => p.test(trimmed));
}

export function extractVideoId(url: string): string | null {
  const m =
    url.match(/[?&]v=([\w-]{6,})/) ??
    url.match(/youtu\.be\/([\w-]{6,})/) ??
    url.match(/shorts\/([\w-]{6,})/);
  return m?.[1] ?? null;
}

export function thumbnailFor(url: string): string | null {
  const id = extractVideoId(url);
  return id ? `https://i.ytimg.com/vi/${id}/mqdefault.jpg` : null;
}
