<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { fly, fade } from "svelte/transition";
  import { flip } from "svelte/animate";
  import {
    Download,
    X,
    Music,
    Loader2,
    AlertCircle,
    Settings,
    Sparkles,
    Check,
    RefreshCw,
    ListPlus,
    Trash2,
    ExternalLink,
    Clipboard,
  } from "lucide-svelte";

  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import CardHeader from "$lib/components/ui/CardHeader.svelte";
  import CardTitle from "$lib/components/ui/CardTitle.svelte";
  import CardDescription from "$lib/components/ui/CardDescription.svelte";
  import CardContent from "$lib/components/ui/CardContent.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";
  import Progress from "$lib/components/ui/Progress.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";

  import { options } from "$lib/stores/options.svelte";
  import { isValidYoutubeUrl, thumbnailFor } from "$lib/utils/youtube";
  import type { DownloadProgress, VideoItem, YtDlpStatus } from "$lib/types";

  let urlInput = $state("");
  let videos = $state<VideoItem[]>([]);
  let isDownloading = $state(false);
  let settingsOpen = $state(false);
  let ytdlpStatus = $state<YtDlpStatus>({ available: false, source: null });
  let detecting = $state(true);
  let toast = $state<string | null>(null);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  let unlisten: UnlistenFn | null = null;

  const inputIsValid = $derived(urlInput.trim() === "" || isValidYoutubeUrl(urlInput));
  const totalProgress = $derived(
    videos.length === 0
      ? 0
      : videos.reduce((acc, v) => acc + v.progress, 0) / videos.length,
  );
  const completedCount = $derived(videos.filter((v) => v.progress >= 100 && !v.error).length);
  const failedCount = $derived(videos.filter((v) => v.error).length);

  onMount(async () => {
    unlisten = await listen<DownloadProgress>("download-progress", (event) => {
      const { url, progress, status, speed, eta, error } = event.payload;
      videos = videos.map((v) =>
        v.url === url ? { ...v, progress, status, speed, eta, error } : v,
      );
    });

    await refreshStatus();
  });

  onDestroy(() => {
    unlisten?.();
    if (toastTimer) clearTimeout(toastTimer);
  });

  async function refreshStatus() {
    detecting = true;
    try {
      ytdlpStatus = await invoke<YtDlpStatus>("check_ytdlp_status");
      if (!ytdlpStatus.available) {
        ytdlpStatus = await invoke<YtDlpStatus>("redetect_ytdlp");
      }
    } finally {
      detecting = false;
    }
  }

  async function redetect() {
    detecting = true;
    try {
      ytdlpStatus = await invoke<YtDlpStatus>("redetect_ytdlp");
      showToast(
        ytdlpStatus.available
          ? `yt-dlp détecté (${ytdlpStatus.source})`
          : "yt-dlp introuvable",
      );
    } finally {
      detecting = false;
    }
  }

  function showToast(message: string) {
    toast = message;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 3200);
  }

  function addOne(raw: string): boolean {
    const url = raw.trim();
    if (!url) return false;
    if (!isValidYoutubeUrl(url)) {
      showToast("URL YouTube invalide");
      return false;
    }
    if (videos.some((v) => v.url === url)) {
      showToast("URL déjà ajoutée");
      return false;
    }
    videos = [
      ...videos,
      {
        url,
        progress: 0,
        status: "Pending",
        speed: null,
        eta: null,
        error: null,
        thumbnail: thumbnailFor(url),
      },
    ];
    return true;
  }

  function addUrl() {
    const lines = urlInput
      .split(/[\s,]+/)
      .map((l) => l.trim())
      .filter(Boolean);

    if (lines.length === 0) return;

    let added = 0;
    for (const line of lines) {
      if (addOne(line)) added++;
    }
    if (added > 0) {
      urlInput = "";
      if (added > 1) showToast(`${added} URLs ajoutées`);
    }
  }

  async function pasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (text) {
        urlInput = text;
        addUrl();
      }
    } catch {
      showToast("Impossible de lire le presse-papiers");
    }
  }

  function removeUrl(url: string) {
    videos = videos.filter((v) => v.url !== url);
  }

  function clearAll() {
    videos = [];
  }

  function clearCompleted() {
    videos = videos.filter((v) => v.progress < 100 || v.error);
  }

  async function downloadAll() {
    if (videos.length === 0) {
      showToast("Ajoute au moins une URL");
      return;
    }
    if (!ytdlpStatus.available) {
      showToast("yt-dlp n'est pas disponible");
      return;
    }

    isDownloading = true;
    try {
      const urls = videos.map((v) => v.url);
      await invoke("download_multiple_videos", { urls, options: options.current });
    } catch (err) {
      console.error("Download error:", err);
      showToast(`Erreur : ${err}`);
    } finally {
      isDownloading = false;
    }
  }

  function statusOf(v: VideoItem): "pending" | "downloading" | "converting" | "done" | "error" {
    if (v.error) return "error";
    if (v.progress >= 100 || v.status === "Completed" || v.status === "Already downloaded")
      return "done";
    if (v.status === "Converting") return "converting";
    if (v.progress > 0 || v.status === "Downloading") return "downloading";
    return "pending";
  }
</script>

<div class="min-h-screen relative overflow-x-hidden">
  <!-- Animated background -->
  <div class="fixed inset-0 -z-10 pointer-events-none">
    <div class="absolute inset-0 bg-gradient-to-br from-black via-gray-950 to-black"></div>
    <div
      class="absolute -top-32 left-1/4 w-[36rem] h-[36rem] bg-red-500/15 rounded-full blur-3xl"
    ></div>
    <div
      class="absolute -bottom-32 right-1/4 w-[36rem] h-[36rem] bg-pink-500/10 rounded-full blur-3xl"
    ></div>
    <div
      class="absolute inset-0 opacity-[0.025]"
      style="background-image: radial-gradient(circle at 1px 1px, white 1px, transparent 0); background-size: 24px 24px;"
    ></div>
  </div>

  <div class="container mx-auto max-w-5xl p-6 space-y-6 relative">
    <!-- Header -->
    <header class="flex items-center justify-between pt-10 pb-4">
      <div class="flex items-center gap-4">
        <div class="relative">
          <div
            class="absolute inset-0 bg-gradient-to-r from-red-500 to-pink-600 rounded-2xl blur-2xl opacity-50"
          ></div>
          <div
            class="relative rounded-2xl bg-gradient-to-br from-red-500 to-pink-600 p-3 shadow-xl shadow-red-500/30"
          >
            <Music class="h-7 w-7 text-white" />
          </div>
        </div>
        <div>
          <h1 class="text-3xl font-black tracking-tight leading-none">
            <span
              class="bg-gradient-to-r from-red-400 via-pink-400 to-red-500 bg-clip-text text-transparent"
            >
              YouTube Converter
            </span>
          </h1>
          <p class="text-sm text-gray-400 mt-1.5">Extrait l'audio de vidéos YouTube en MP3 / M4A / FLAC</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        {#if detecting}
          <span class="flex items-center gap-2 text-xs text-gray-400 px-3 py-1.5 rounded-lg bg-white/5">
            <Loader2 class="h-3.5 w-3.5 animate-spin" />
            Détection…
          </span>
        {:else if ytdlpStatus.available}
          <span
            class="flex items-center gap-2 text-xs font-medium text-emerald-300 px-3 py-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20"
            title={ytdlpStatus.source ?? ""}
          >
            <span class="h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.7)]"></span>
            yt-dlp prêt
          </span>
        {:else}
          <button
            type="button"
            onclick={redetect}
            class="flex items-center gap-2 text-xs font-medium text-red-300 px-3 py-1.5 rounded-lg bg-red-500/10 border border-red-500/20 hover:bg-red-500/20 transition-colors"
          >
            <AlertCircle class="h-3.5 w-3.5" />
            yt-dlp indisponible
          </button>
        {/if}
        <Button variant="outline" size="icon" onclick={redetect} title="Re-détecter yt-dlp">
          {#snippet children()}<RefreshCw class="h-4 w-4 {detecting ? 'animate-spin' : ''}" />{/snippet}
        </Button>
        <Button variant="outline" size="icon" onclick={() => (settingsOpen = true)} title="Paramètres">
          {#snippet children()}<Settings class="h-4 w-4" />{/snippet}
        </Button>
      </div>
    </header>

    <!-- URL input -->
    <Card>
      <CardHeader>
        <CardTitle>
          {#snippet children()}
            <Sparkles class="h-5 w-5 text-red-500" />
            Ajouter une URL
          {/snippet}
        </CardTitle>
        <CardDescription>
          {#snippet children()}
            Une URL par ligne, ou plusieurs séparées par espaces / virgules
          {/snippet}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div class="flex gap-3">
          <div class="relative flex-1">
            <Input
              bind:value={urlInput}
              placeholder="https://www.youtube.com/watch?v=…"
              disabled={isDownloading}
              onkeydown={(e: KeyboardEvent) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  addUrl();
                }
              }}
              class={!inputIsValid ? "border-red-500/50 focus-visible:ring-red-500/50" : ""}
            />
            {#if !inputIsValid}
              <span
                class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-red-400"
                transition:fade={{ duration: 120 }}
              >
                URL invalide
              </span>
            {/if}
          </div>
          <Button variant="outline" size="icon" onclick={pasteFromClipboard} title="Coller">
            {#snippet children()}<Clipboard class="h-4 w-4" />{/snippet}
          </Button>
          <Button onclick={addUrl} disabled={isDownloading || !urlInput.trim() || !inputIsValid}>
            {#snippet children()}
              <ListPlus class="h-4 w-4" />
              Ajouter
            {/snippet}
          </Button>
        </div>
      </CardContent>
    </Card>

    <!-- Queue -->
    {#if videos.length > 0}
      <Card>
        <CardHeader>
          {#snippet children()}
            <div class="flex items-center justify-between gap-4">
              <div class="space-y-1 min-w-0">
                <CardTitle>
                  {#snippet children()}
                    File de téléchargement
                    <span class="ml-1 text-sm font-medium text-gray-400">
                      ({videos.length})
                    </span>
                  {/snippet}
                </CardTitle>
                <CardDescription>
                  {#snippet children()}
                    {completedCount} terminé{completedCount > 1 ? "s" : ""} ·
                    {failedCount} échec{failedCount > 1 ? "s" : ""}
                  {/snippet}
                </CardDescription>
              </div>
              <div class="flex items-center gap-2 shrink-0">
                {#if completedCount > 0}
                  <Button
                    variant="ghost"
                    size="sm"
                    onclick={clearCompleted}
                    disabled={isDownloading}
                  >
                    {#snippet children()}
                      <Check class="h-3.5 w-3.5" />
                      Vider terminés
                    {/snippet}
                  </Button>
                {/if}
                <Button variant="outline" size="sm" onclick={clearAll} disabled={isDownloading}>
                  {#snippet children()}
                    <Trash2 class="h-3.5 w-3.5" />
                    Tout effacer
                  {/snippet}
                </Button>
              </div>
            </div>

            {#if isDownloading || totalProgress > 0}
              <div class="mt-2 flex items-center gap-3">
                <Progress value={totalProgress} class="flex-1 h-1.5" />
                <span class="text-xs font-bold text-gray-300 tabular-nums w-10 text-right">
                  {Math.round(totalProgress)}%
                </span>
              </div>
            {/if}
          {/snippet}
        </CardHeader>
        <CardContent>
          {#snippet children()}
            <ul class="space-y-3">
              {#each videos as video (video.url)}
                {@const s = statusOf(video)}
                <li
                  in:fly={{ y: 8, duration: 200 }}
                  out:fade={{ duration: 150 }}
                  animate:flip={{ duration: 250 }}
                  class="group relative rounded-2xl border bg-white/[0.03] backdrop-blur-xl p-4 transition-all duration-300 hover:bg-white/[0.06] {s === 'done'
                    ? 'border-emerald-500/30'
                    : s === 'error'
                      ? 'border-red-500/30'
                      : 'border-white/5 hover:border-white/15'}"
                >
                  <div class="flex gap-4">
                    {#if video.thumbnail}
                      <div
                        class="relative h-16 w-28 shrink-0 overflow-hidden rounded-lg bg-white/5 ring-1 ring-white/10"
                      >
                        <img
                          src={video.thumbnail}
                          alt=""
                          loading="lazy"
                          decoding="async"
                          class="h-full w-full object-cover"
                          onerror={(e) => ((e.currentTarget as HTMLImageElement).style.opacity = "0")}
                        />
                      </div>
                    {/if}

                    <div class="flex-1 min-w-0 space-y-2">
                      <div class="flex items-start justify-between gap-3">
                        <p
                          class="text-sm font-medium text-gray-200 line-clamp-1 flex-1 leading-relaxed break-all"
                          title={video.url}
                        >
                          {video.url}
                        </p>
                        <button
                          type="button"
                          onclick={() => removeUrl(video.url)}
                          disabled={isDownloading}
                          aria-label="Supprimer"
                          class="h-7 w-7 shrink-0 rounded-lg flex items-center justify-center text-gray-400 hover:bg-white/10 hover:text-white transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
                        >
                          <X class="h-4 w-4" />
                        </button>
                      </div>

                      <div class="flex items-center justify-between gap-3">
                        <div class="flex items-center gap-2 text-xs min-w-0">
                          {#if s === "error"}
                            <span class="flex items-center gap-1.5 text-red-400 font-medium truncate">
                              <AlertCircle class="h-3.5 w-3.5 shrink-0" />
                              <span class="truncate">{video.error}</span>
                            </span>
                          {:else if s === "downloading"}
                            <span class="flex items-center gap-1.5 text-blue-300 font-medium">
                              <Loader2 class="h-3.5 w-3.5 animate-spin" />
                              Téléchargement
                            </span>
                          {:else if s === "converting"}
                            <span class="flex items-center gap-1.5 text-purple-300 font-medium">
                              <Loader2 class="h-3.5 w-3.5 animate-spin" />
                              Conversion audio
                            </span>
                          {:else if s === "done"}
                            <span class="flex items-center gap-1.5 text-emerald-400 font-semibold">
                              <Check class="h-3.5 w-3.5" />
                              Terminé
                            </span>
                          {:else}
                            <span class="text-gray-400 font-medium">En attente</span>
                          {/if}

                          {#if video.speed}
                            <span class="text-gray-500 tabular-nums">· {video.speed}</span>
                          {/if}
                          {#if video.eta && s === "downloading"}
                            <span class="text-gray-500 tabular-nums">· ETA {video.eta}</span>
                          {/if}
                        </div>

                        <span class="text-xs font-bold text-gray-300 tabular-nums w-10 text-right">
                          {Math.round(video.progress)}%
                        </span>
                      </div>

                      <Progress
                        value={video.progress}
                        indeterminate={s === "converting"}
                        class={s === "done" ? "[&>div]:from-emerald-500 [&>div]:via-emerald-400 [&>div]:to-emerald-500" : s === "error" ? "[&>div]:from-red-600 [&>div]:via-red-500 [&>div]:to-red-600" : ""}
                      />
                    </div>
                  </div>
                </li>
              {/each}
            </ul>

            <Button
              onclick={downloadAll}
              disabled={isDownloading || !ytdlpStatus.available}
              size="lg"
              class="w-full mt-6"
            >
              {#snippet children()}
                {#if isDownloading}
                  <Loader2 class="h-5 w-5 animate-spin" />
                  Téléchargement en cours…
                {:else}
                  <Download class="h-5 w-5" />
                  Télécharger ({videos.length})
                {/if}
              {/snippet}
            </Button>
          {/snippet}
        </CardContent>
      </Card>
    {:else}
      <div
        class="rounded-3xl border-2 border-dashed border-white/10 bg-white/[0.02] p-12 text-center"
        in:fade={{ duration: 200 }}
      >
        <div class="relative inline-block mb-5">
          <div class="absolute inset-0 bg-red-500/20 rounded-full blur-2xl"></div>
          <div class="relative rounded-2xl bg-white/5 p-5 ring-1 ring-white/10">
            <Music class="h-10 w-10 text-gray-500" />
          </div>
        </div>
        <h3 class="text-lg font-bold mb-1.5 text-white">Aucune vidéo</h3>
        <p class="text-sm text-gray-400 max-w-sm mx-auto">
          Colle une URL YouTube ci-dessus pour commencer. Tu peux aussi en coller plusieurs d'un coup.
        </p>
      </div>
    {/if}

    <!-- Footer -->
    <div class="text-center space-y-1.5 text-xs text-gray-500 pt-4 pb-12">
      <p>Les fichiers sont enregistrés dans ton dossier <span class="font-medium text-gray-400">Téléchargements</span></p>
      {#if !ytdlpStatus.available && !detecting}
        <p class="text-red-400 font-medium flex items-center justify-center gap-1.5">
          <AlertCircle class="h-3.5 w-3.5" />
          yt-dlp introuvable —
          <a
            href="https://github.com/yt-dlp/yt-dlp#installation"
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:text-red-300 transition-colors inline-flex items-center gap-1"
          >
            installer
            <ExternalLink class="h-3 w-3" />
          </a>
        </p>
      {/if}
    </div>
  </div>

  <!-- Toast -->
  {#if toast}
    <div
      class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 px-4 py-2.5 rounded-xl bg-white/10 backdrop-blur-2xl border border-white/15 text-sm font-medium text-white shadow-2xl"
      transition:fly={{ y: 16, duration: 200 }}
    >
      {toast}
    </div>
  {/if}

  <!-- Settings dialog -->
  <Dialog open={settingsOpen} onClose={() => (settingsOpen = false)}>
    {#snippet title()}
      <Settings class="h-6 w-6 text-red-500" />
      Paramètres
    {/snippet}
    {#snippet description()}
      Personnalise tes téléchargements (qualité, format, métadonnées)
    {/snippet}
    {#snippet children()}
      <div class="space-y-6">
        <div class="grid grid-cols-2 gap-5">
          <div class="space-y-2">
            <Label for="format">{#snippet children()}Format audio{/snippet}</Label>
            <Select
              id="format"
              value={options.current.audioFormat}
              onchange={(e: Event) =>
                options.update({ audioFormat: (e.currentTarget as HTMLSelectElement).value })}
              options={[
                { value: "mp3", label: "MP3" },
                { value: "m4a", label: "M4A (AAC)" },
                { value: "opus", label: "Opus" },
                { value: "vorbis", label: "Vorbis" },
                { value: "wav", label: "WAV (sans perte)" },
                { value: "flac", label: "FLAC (sans perte)" },
              ]}
            />
          </div>

          <div class="space-y-2">
            <Label for="quality">{#snippet children()}Qualité{/snippet}</Label>
            <Select
              id="quality"
              value={options.current.audioQuality}
              onchange={(e: Event) =>
                options.update({ audioQuality: (e.currentTarget as HTMLSelectElement).value })}
              options={[
                { value: "0", label: "Maximale (320 kbps)" },
                { value: "2", label: "Haute (256 kbps)" },
                { value: "5", label: "Moyenne (192 kbps)" },
                { value: "7", label: "Basse (128 kbps)" },
                { value: "9", label: "Minimale (64 kbps)" },
              ]}
            />
          </div>
        </div>

        <div class="space-y-2">
          <Label for="template">{#snippet children()}Modèle de nom de fichier{/snippet}</Label>
          <Input
            id="template"
            value={options.current.outputTemplate}
            oninput={(e: Event) =>
              options.update({
                outputTemplate: (e.currentTarget as HTMLInputElement).value,
              })}
            placeholder="%(title)s.%(ext)s"
          />
          <p class="text-xs text-gray-500">
            Variables yt-dlp : <code class="px-1 py-0.5 rounded bg-white/10">%(title)s</code>,
            <code class="px-1 py-0.5 rounded bg-white/10">%(uploader)s</code>,
            <code class="px-1 py-0.5 rounded bg-white/10">%(ext)s</code>
          </p>
        </div>

        <div class="space-y-3 pt-3 border-t border-white/10">
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <Label for="thumb">{#snippet children()}Intégrer la miniature{/snippet}</Label>
              <p class="text-xs text-gray-500">Embed la cover dans le fichier audio</p>
            </div>
            <Switch
              id="thumb"
              aria-label="Intégrer la miniature"
              checked={options.current.embedThumbnail}
              onCheckedChange={(v) => options.update({ embedThumbnail: v })}
            />
          </div>
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <Label for="meta">{#snippet children()}Ajouter les métadonnées{/snippet}</Label>
              <p class="text-xs text-gray-500">Titre, artiste, album dans les tags ID3</p>
            </div>
            <Switch
              id="meta"
              aria-label="Ajouter les métadonnées"
              checked={options.current.addMetadata}
              onCheckedChange={(v) => options.update({ addMetadata: v })}
            />
          </div>
        </div>

        {#if ytdlpStatus.source}
          <div class="pt-3 border-t border-white/10">
            <p class="text-xs text-gray-500">
              <span class="font-semibold text-gray-400">yt-dlp :</span>
              {ytdlpStatus.source}
            </p>
          </div>
        {/if}
      </div>

      <div class="flex justify-end gap-3 pt-6 mt-6 border-t border-white/10">
        <Button variant="outline" onclick={() => (settingsOpen = false)}>
          {#snippet children()}Fermer{/snippet}
        </Button>
      </div>
    {/snippet}
  </Dialog>
</div>
