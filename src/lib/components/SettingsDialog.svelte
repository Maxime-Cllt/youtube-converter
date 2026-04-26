<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    Settings as SettingsIcon,
    FileVideo,
    Music2,
    Tag,
    Subtitles,
    ShieldOff,
    Network,
    SlidersHorizontal,
    Folder,
    FileText,
    RotateCcw,
    Info,
  } from "lucide-svelte";

  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Switch from "$lib/components/ui/Switch.svelte";
  import Tabs, { type Tab } from "$lib/components/ui/Tabs.svelte";
  import Field from "$lib/components/ui/Field.svelte";
  import CheckboxChip from "$lib/components/ui/CheckboxChip.svelte";

  import { options } from "$lib/stores/options.svelte";
  import type { SponsorBlockCategory, YtDlpStatus } from "$lib/types";

  interface Props {
    open: boolean;
    onClose: () => void;
    ytdlpStatus: YtDlpStatus;
  }

  let { open: isOpen, onClose, ytdlpStatus }: Props = $props();

  let activeTab = $state("format");

  const tabs: Tab[] = [
    { value: "format", label: "Format", icon: FileVideo },
    { value: "embed", label: "Métadonnées", icon: Tag },
    { value: "subs", label: "Sous-titres", icon: Subtitles },
    { value: "sponsor", label: "SponsorBlock", icon: ShieldOff },
    { value: "network", label: "Réseau", icon: Network },
    { value: "advanced", label: "Avancé", icon: SlidersHorizontal },
  ];

  const sbCategories: { value: SponsorBlockCategory; label: string }[] = [
    { value: "sponsor", label: "Sponsor" },
    { value: "selfpromo", label: "Auto-promo" },
    { value: "intro", label: "Intro" },
    { value: "outro", label: "Outro" },
    { value: "interaction", label: "Interaction" },
    { value: "music_offtopic", label: "Hors-sujet (musique)" },
    { value: "preview", label: "Récap / preview" },
    { value: "filler", label: "Filler" },
  ];

  function toggleSbCategory(c: SponsorBlockCategory) {
    const current = options.current.sponsorblockCategories;
    const next = current.includes(c) ? current.filter((x) => x !== c) : [...current, c];
    options.update({ sponsorblockCategories: next });
  }

  async function pickFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Dossier de sortie",
    });
    if (typeof selected === "string") {
      options.update({ outputDir: selected });
    }
  }

  async function pickCookies() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Fichier cookies",
      filters: [{ name: "Cookies", extensions: ["txt"] }],
    });
    if (typeof selected === "string") {
      options.update({ cookiesFile: selected });
    }
  }

  function resetAll() {
    if (confirm("Réinitialiser tous les paramètres aux valeurs par défaut ?")) {
      options.reset();
    }
  }
</script>

<Dialog open={isOpen} {onClose} class="max-w-3xl">
  {#snippet title()}
    <SettingsIcon class="h-6 w-6 text-red-500" />
    Paramètres
  {/snippet}
  {#snippet description()}
    Configure finement yt-dlp : format, qualité, sous-titres, réseau…
  {/snippet}
  {#snippet children()}
    <Tabs {tabs} value={activeTab} onValueChange={(v) => (activeTab = v)}>
      {#snippet children()}
        <!-- ============= FORMAT TAB ============= -->
        {#if activeTab === "format"}
          <div class="space-y-5">
            <div class="grid grid-cols-2 gap-2 p-1 rounded-xl bg-white/5 border border-white/10">
              <button
                type="button"
                onclick={() => options.update({ mode: "audio" })}
                class={[
                  "flex items-center justify-center gap-2 py-3 rounded-lg text-sm font-semibold transition-all",
                  options.current.mode === "audio"
                    ? "bg-gradient-to-br from-red-500 to-pink-600 text-white shadow-lg shadow-red-500/20"
                    : "text-gray-400 hover:text-white hover:bg-white/5",
                ]}
              >
                <Music2 class="h-4 w-4" />
                Audio uniquement
              </button>
              <button
                type="button"
                onclick={() => options.update({ mode: "video" })}
                class={[
                  "flex items-center justify-center gap-2 py-3 rounded-lg text-sm font-semibold transition-all",
                  options.current.mode === "video"
                    ? "bg-gradient-to-br from-red-500 to-pink-600 text-white shadow-lg shadow-red-500/20"
                    : "text-gray-400 hover:text-white hover:bg-white/5",
                ]}
              >
                <FileVideo class="h-4 w-4" />
                Vidéo + audio
              </button>
            </div>

            {#if options.current.mode === "audio"}
              <div class="grid grid-cols-2 gap-4">
                <Field label="Format audio" hint="MP3 = compatibilité max · FLAC = sans perte">
                  {#snippet children()}
                    <Select
                      value={options.current.audioFormat}
                      onchange={(e: Event) =>
                        options.update({ audioFormat: (e.currentTarget as HTMLSelectElement).value })}
                      options={[
                        { value: "mp3", label: "MP3" },
                        { value: "m4a", label: "M4A (AAC)" },
                        { value: "opus", label: "Opus" },
                        { value: "vorbis", label: "Vorbis (OGG)" },
                        { value: "wav", label: "WAV (sans perte)" },
                        { value: "flac", label: "FLAC (sans perte)" },
                        { value: "aac", label: "AAC" },
                      ]}
                    />
                  {/snippet}
                </Field>

                <Field label="Qualité" hint="0 = max VBR (~245 kbps) · 9 = min (~64 kbps)">
                  {#snippet children()}
                    <Select
                      value={options.current.audioQuality}
                      onchange={(e: Event) =>
                        options.update({ audioQuality: (e.currentTarget as HTMLSelectElement).value })}
                      options={[
                        { value: "0", label: "0 — Maximale (~245 kbps)" },
                        { value: "2", label: "2 — Très haute (~190 kbps)" },
                        { value: "5", label: "5 — Moyenne (~130 kbps)" },
                        { value: "7", label: "7 — Basse (~100 kbps)" },
                        { value: "9", label: "9 — Minimale (~64 kbps)" },
                      ]}
                    />
                  {/snippet}
                </Field>
              </div>
            {:else}
              <div class="grid grid-cols-2 gap-4">
                <Field label="Résolution max" hint="yt-dlp choisira la meilleure ≤ cette valeur">
                  {#snippet children()}
                    <Select
                      value={options.current.videoResolution}
                      onchange={(e: Event) =>
                        options.update({ videoResolution: (e.currentTarget as HTMLSelectElement).value })}
                      options={[
                        { value: "best", label: "Meilleure disponible" },
                        { value: "2160", label: "2160p (4K)" },
                        { value: "1440", label: "1440p (2K)" },
                        { value: "1080", label: "1080p (Full HD)" },
                        { value: "720", label: "720p (HD)" },
                        { value: "480", label: "480p" },
                        { value: "360", label: "360p" },
                      ]}
                    />
                  {/snippet}
                </Field>

                <Field label="Conteneur" hint="MP4 = compatibilité · MKV = couvre-tout · WebM = libre">
                  {#snippet children()}
                    <Select
                      value={options.current.videoContainer}
                      onchange={(e: Event) =>
                        options.update({ videoContainer: (e.currentTarget as HTMLSelectElement).value })}
                      options={[
                        { value: "default", label: "Par défaut" },
                        { value: "mp4", label: "MP4" },
                        { value: "mkv", label: "MKV" },
                        { value: "webm", label: "WebM" },
                      ]}
                    />
                  {/snippet}
                </Field>

                <Field label="Codec préféré" hint="H.264 = compatibilité max · AV1 = compression++">
                  {#snippet children()}
                    <Select
                      value={options.current.videoCodec}
                      onchange={(e: Event) =>
                        options.update({ videoCodec: (e.currentTarget as HTMLSelectElement).value })}
                      options={[
                        { value: "any", label: "Aucune préférence" },
                        { value: "h264", label: "H.264 (AVC)" },
                        { value: "h265", label: "H.265 (HEVC)" },
                        { value: "vp9", label: "VP9" },
                        { value: "av1", label: "AV1" },
                      ]}
                    />
                  {/snippet}
                </Field>

                <div class="flex items-center justify-between pt-7">
                  <div class="space-y-0.5">
                    <p class="text-sm font-semibold text-gray-200">Formats libres</p>
                    <p class="text-xs text-gray-500">Préférer VP9/Opus à H.264/AAC</p>
                  </div>
                  <Switch
                    aria-label="Préférer formats libres"
                    checked={options.current.preferFreeFormats}
                    onCheckedChange={(v) => options.update({ preferFreeFormats: v })}
                  />
                </div>
              </div>
            {/if}

            <div class="border-t border-white/10 pt-5 space-y-4">
              <Field
                label="Dossier de sortie"
                hint="Vide = dossier Téléchargements de l'utilisateur"
              >
                {#snippet children()}
                  <div class="flex gap-2">
                    <Input
                      value={options.current.outputDir ?? ""}
                      oninput={(e: Event) =>
                        options.update({
                          outputDir:
                            (e.currentTarget as HTMLInputElement).value || null,
                        })}
                      placeholder="~/Downloads"
                    />
                    <Button variant="outline" size="icon" onclick={pickFolder} title="Parcourir">
                      {#snippet children()}<Folder class="h-4 w-4" />{/snippet}
                    </Button>
                  </div>
                {/snippet}
              </Field>

              <Field
                label="Modèle de nom de fichier"
                hint="Variables yt-dlp : %(title)s, %(uploader)s, %(upload_date)s, %(playlist_index)s, %(ext)s"
              >
                {#snippet children()}
                  <Input
                    value={options.current.outputTemplate}
                    oninput={(e: Event) =>
                      options.update({
                        outputTemplate: (e.currentTarget as HTMLInputElement).value,
                      })}
                    placeholder="%(title)s.%(ext)s"
                  />
                {/snippet}
              </Field>
            </div>
          </div>
        {/if}

        <!-- ============= EMBED TAB ============= -->
        {#if activeTab === "embed"}
          <div class="space-y-3">
            {#each [
              { key: "embedThumbnail", label: "Intégrer la miniature", hint: "Embed la cover dans le fichier" },
              { key: "addMetadata", label: "Ajouter les métadonnées", hint: "Titre, artiste, album, année dans les tags" },
              { key: "embedChapters", label: "Intégrer les chapitres", hint: "Embed les marqueurs de chapitre dans MP4/MKV" },
              { key: "embedSubs", label: "Intégrer les sous-titres", hint: "Embed les pistes de sous-titres dans le fichier (vidéo)" },
              { key: "writeInfoJson", label: "Sauvegarder le JSON d'info", hint: "Écrit un .info.json avec toutes les métadonnées brutes" },
              { key: "writeDescription", label: "Sauvegarder la description", hint: "Écrit un .description avec le texte de la vidéo" },
            ] as item (item.key)}
              <div class="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/5 hover:bg-white/[0.06] transition-colors">
                <div class="space-y-0.5 pr-4">
                  <p class="text-sm font-semibold text-gray-200">{item.label}</p>
                  <p class="text-xs text-gray-500">{item.hint}</p>
                </div>
                <Switch
                  aria-label={item.label}
                  checked={options.current[item.key as keyof typeof options.current] as boolean}
                  onCheckedChange={(v) => options.update({ [item.key]: v })}
                />
              </div>
            {/each}
          </div>
        {/if}

        <!-- ============= SUBTITLES TAB ============= -->
        {#if activeTab === "subs"}
          <div class="space-y-4">
            <div class="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/5">
              <div class="space-y-0.5">
                <p class="text-sm font-semibold text-gray-200">Télécharger les sous-titres</p>
                <p class="text-xs text-gray-500">Pistes manuelles fournies par l'auteur</p>
              </div>
              <Switch
                aria-label="Télécharger sous-titres"
                checked={options.current.writeSubs}
                onCheckedChange={(v) => options.update({ writeSubs: v })}
              />
            </div>

            <div class="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/5">
              <div class="space-y-0.5">
                <p class="text-sm font-semibold text-gray-200">Sous-titres auto-générés</p>
                <p class="text-xs text-gray-500">Inclure la transcription auto YouTube</p>
              </div>
              <Switch
                aria-label="Sous-titres auto"
                checked={options.current.writeAutoSubs}
                onCheckedChange={(v) => options.update({ writeAutoSubs: v })}
              />
            </div>

            <Field
              label="Langues (codes ISO, séparés par virgule)"
              hint='Exemples : "en" · "en,fr" · "all" · "en.*" pour toutes les variantes anglaises'
            >
              {#snippet children()}
                <Input
                  value={options.current.subLangs}
                  oninput={(e: Event) =>
                    options.update({ subLangs: (e.currentTarget as HTMLInputElement).value })}
                  placeholder="en,fr"
                />
              {/snippet}
            </Field>

            <div
              class="flex items-start gap-3 p-3 rounded-xl bg-blue-500/10 border border-blue-500/20"
            >
              <Info class="h-4 w-4 text-blue-300 shrink-0 mt-0.5" />
              <p class="text-xs text-blue-200/80 leading-relaxed">
                Pour intégrer les sous-titres directement dans le fichier vidéo, active aussi
                <span class="font-semibold">Intégrer les sous-titres</span> dans l'onglet Métadonnées.
              </p>
            </div>
          </div>
        {/if}

        <!-- ============= SPONSORBLOCK TAB ============= -->
        {#if activeTab === "sponsor"}
          <div class="space-y-5">
            <div class="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/5">
              <div class="space-y-0.5">
                <p class="text-sm font-semibold text-gray-200">Activer SponsorBlock</p>
                <p class="text-xs text-gray-500">
                  Coupe automatiquement les segments référencés sur sponsor.ajay.app
                </p>
              </div>
              <Switch
                aria-label="Activer SponsorBlock"
                checked={options.current.sponsorblockRemove}
                onCheckedChange={(v) => options.update({ sponsorblockRemove: v })}
              />
            </div>

            <div class="space-y-2">
              <p class="text-sm font-semibold text-gray-200">Catégories à supprimer</p>
              <div class="flex flex-wrap gap-2">
                {#each sbCategories as cat (cat.value)}
                  <CheckboxChip
                    label={cat.label}
                    checked={options.current.sponsorblockCategories.includes(cat.value)}
                    onToggle={() => toggleSbCategory(cat.value)}
                    disabled={!options.current.sponsorblockRemove}
                  />
                {/each}
              </div>
            </div>
          </div>
        {/if}

        <!-- ============= NETWORK TAB ============= -->
        {#if activeTab === "network"}
          <div class="space-y-4">
            <div class="grid grid-cols-2 gap-4">
              <Field
                label="Limite de débit"
                hint='Vide = illimité · "1M" · "500K" · "2.5M"'
              >
                {#snippet children()}
                  <Input
                    value={options.current.limitRate}
                    oninput={(e: Event) =>
                      options.update({ limitRate: (e.currentTarget as HTMLInputElement).value })}
                    placeholder="1M"
                  />
                {/snippet}
              </Field>

              <Field
                label="Fragments parallèles"
                hint="2-16 · accélère sur connexions rapides"
              >
                {#snippet children()}
                  <Input
                    type="number"
                    min={1}
                    max={16}
                    value={String(options.current.concurrentFragments)}
                    oninput={(e: Event) => {
                      const v = parseInt((e.currentTarget as HTMLInputElement).value, 10);
                      options.update({ concurrentFragments: isNaN(v) ? 1 : Math.min(16, Math.max(1, v)) });
                    }}
                  />
                {/snippet}
              </Field>

              <Field label="Tentatives" hint="Nombre de retries par fragment (défaut yt-dlp : 10)">
                {#snippet children()}
                  <Input
                    type="number"
                    min={0}
                    max={100}
                    value={String(options.current.retries)}
                    oninput={(e: Event) => {
                      const v = parseInt((e.currentTarget as HTMLInputElement).value, 10);
                      options.update({ retries: isNaN(v) ? 10 : Math.min(100, Math.max(0, v)) });
                    }}
                  />
                {/snippet}
              </Field>

              <Field label="Proxy" hint='Format URL : "socks5://127.0.0.1:1080"'>
                {#snippet children()}
                  <Input
                    value={options.current.proxy ?? ""}
                    oninput={(e: Event) =>
                      options.update({
                        proxy: (e.currentTarget as HTMLInputElement).value || null,
                      })}
                    placeholder="http://… ou socks5://…"
                  />
                {/snippet}
              </Field>
            </div>

            <Field
              label="User-Agent personnalisé"
              hint="Vide = User-Agent par défaut yt-dlp"
            >
              {#snippet children()}
                <Input
                  value={options.current.userAgent ?? ""}
                  oninput={(e: Event) =>
                    options.update({
                      userAgent: (e.currentTarget as HTMLInputElement).value || null,
                    })}
                  placeholder="Mozilla/5.0 …"
                />
              {/snippet}
            </Field>

            <Field
              label="Fichier cookies"
              hint="Format Netscape ou cookies.txt — utile pour vidéos privées / restreintes"
            >
              {#snippet children()}
                <div class="flex gap-2">
                  <Input
                    value={options.current.cookiesFile ?? ""}
                    oninput={(e: Event) =>
                      options.update({
                        cookiesFile: (e.currentTarget as HTMLInputElement).value || null,
                      })}
                    placeholder="/chemin/vers/cookies.txt"
                  />
                  <Button variant="outline" size="icon" onclick={pickCookies} title="Parcourir">
                    {#snippet children()}<FileText class="h-4 w-4" />{/snippet}
                  </Button>
                </div>
              {/snippet}
            </Field>
          </div>
        {/if}

        <!-- ============= ADVANCED TAB ============= -->
        {#if activeTab === "advanced"}
          <div class="space-y-4">
            <div class="flex items-center justify-between p-3 rounded-xl bg-white/[0.03] border border-white/5">
              <div class="space-y-0.5">
                <p class="text-sm font-semibold text-gray-200">Ignorer les playlists</p>
                <p class="text-xs text-gray-500">
                  Sur une URL avec <code>?list=</code>, télécharger uniquement la vidéo
                </p>
              </div>
              <Switch
                aria-label="Ignorer playlist"
                checked={options.current.noPlaylist}
                onCheckedChange={(v) => options.update({ noPlaylist: v })}
              />
            </div>

            {#if !options.current.noPlaylist}
              <Field
                label="Plage d'éléments de playlist"
                hint='Exemples : "1-5" · "1,3,5" · "1-3,7,10-15" · vide = tous'
              >
                {#snippet children()}
                  <Input
                    value={options.current.playlistItems}
                    oninput={(e: Event) =>
                      options.update({
                        playlistItems: (e.currentTarget as HTMLInputElement).value,
                      })}
                    placeholder="1-10"
                  />
                {/snippet}
              </Field>
            {/if}

            <Field
              label="Arguments yt-dlp supplémentaires"
              hint="Ajoutés bruts à la commande. Séparés par espaces. Pour utilisateurs avancés."
            >
              {#snippet children()}
                <Input
                  value={options.current.customArgs}
                  oninput={(e: Event) =>
                    options.update({
                      customArgs: (e.currentTarget as HTMLInputElement).value,
                    })}
                  placeholder="--no-mtime --geo-bypass"
                />
              {/snippet}
            </Field>

            <div
              class="flex items-start gap-3 p-3 rounded-xl bg-amber-500/10 border border-amber-500/20"
            >
              <Info class="h-4 w-4 text-amber-300 shrink-0 mt-0.5" />
              <p class="text-xs text-amber-200/80 leading-relaxed">
                Documentation complète :
                <a
                  href="https://github.com/yt-dlp/yt-dlp#usage-and-options"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="underline hover:text-amber-100"
                >
                  github.com/yt-dlp/yt-dlp
                </a>
              </p>
            </div>

            {#if ytdlpStatus.source}
              <div
                class="flex items-start gap-3 p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20"
              >
                <Info class="h-4 w-4 text-emerald-300 shrink-0 mt-0.5" />
                <div class="text-xs leading-relaxed flex-1 min-w-0">
                  <p class="font-semibold text-emerald-200 mb-0.5">yt-dlp</p>
                  <p class="text-emerald-200/70 break-all">{ytdlpStatus.source}</p>
                </div>
              </div>
            {/if}

            <div
              class={[
                "flex items-start gap-3 p-3 rounded-xl border",
                ytdlpStatus.ffmpegAvailable
                  ? "bg-emerald-500/10 border-emerald-500/20"
                  : "bg-red-500/10 border-red-500/20",
              ]}
            >
              <Info
                class="h-4 w-4 shrink-0 mt-0.5 {ytdlpStatus.ffmpegAvailable
                  ? 'text-emerald-300'
                  : 'text-red-300'}"
              />
              <div class="text-xs leading-relaxed flex-1 min-w-0">
                <p
                  class={[
                    "font-semibold mb-0.5",
                    ytdlpStatus.ffmpegAvailable ? "text-emerald-200" : "text-red-200",
                  ]}
                >
                  ffmpeg + ffprobe
                </p>
                {#if ytdlpStatus.ffmpegAvailable}
                  <p class="text-emerald-200/70 break-all">{ytdlpStatus.ffmpegSource}</p>
                {:else}
                  <p class="text-red-200/70">
                    Non détecté. yt-dlp ne pourra pas extraire l'audio, intégrer thumbnails ni convertir les formats.
                    Installe ffmpeg via <code class="px-1 py-0.5 rounded bg-white/10">brew install ffmpeg</code>
                    (macOS), ton gestionnaire de paquets (Linux) ou
                    <a
                      href="https://ffmpeg.org/download.html"
                      target="_blank"
                      rel="noopener noreferrer"
                      class="underline hover:text-red-100"
                    >
                      ffmpeg.org
                    </a>
                    (Windows).
                  </p>
                {/if}
              </div>
            </div>
          </div>
        {/if}
      {/snippet}
    </Tabs>

    <div
      class="flex items-center justify-between gap-3 pt-5 mt-4 border-t border-white/10"
    >
      <Button variant="ghost" size="sm" onclick={resetAll}>
        {#snippet children()}
          <RotateCcw class="h-3.5 w-3.5" />
          Réinitialiser
        {/snippet}
      </Button>
      <Button variant="outline" onclick={onClose}>
        {#snippet children()}Fermer{/snippet}
      </Button>
    </div>
  {/snippet}
</Dialog>
