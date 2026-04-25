<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  interface Props {
    open: boolean;
    onClose: () => void;
    children: Snippet;
    title?: Snippet;
    description?: Snippet;
    class?: string;
  }

  let {
    open,
    onClose,
    children,
    title,
    description,
    class: className = "",
  }: Props = $props();

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    role="dialog"
    aria-modal="true"
  >
    <button
      type="button"
      aria-label="Close"
      class="fixed inset-0 bg-black/80 backdrop-blur-sm cursor-default"
      onclick={onClose}
      transition:fade={{ duration: 150 }}
    ></button>

    <div
      class="relative z-10 w-full max-w-2xl rounded-3xl border border-white/15 bg-gradient-to-br from-gray-900/95 via-black/95 to-gray-900/95 backdrop-blur-3xl p-8 shadow-2xl {className}"
      transition:scale={{ duration: 200, easing: cubicOut, start: 0.96 }}
    >
      {#if title || description}
        <div class="mb-6 space-y-2">
          {#if title}
            <h2
              class="text-2xl font-bold leading-none tracking-tight bg-gradient-to-r from-white to-gray-300 bg-clip-text text-transparent flex items-center gap-3"
            >
              {@render title()}
            </h2>
          {/if}
          {#if description}
            <p class="text-sm text-gray-400">{@render description()}</p>
          {/if}
        </div>
      {/if}
      {@render children()}
    </div>
  </div>
{/if}
