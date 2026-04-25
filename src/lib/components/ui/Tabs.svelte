<script lang="ts" module>
  export interface Tab {
    value: string;
    label: string;
    icon?: typeof import("lucide-svelte").Settings;
  }
</script>

<script lang="ts">
  import { cn } from "$lib/utils/cn";
  import type { Snippet } from "svelte";

  interface Props {
    tabs: Tab[];
    value: string;
    onValueChange: (v: string) => void;
    children: Snippet;
    class?: string;
  }

  let { tabs, value, onValueChange, children, class: className = "" }: Props = $props();
</script>

<div class={cn("flex flex-col gap-4 min-h-0", className)}>
  <div role="tablist" class="flex gap-1 p-1 rounded-xl bg-white/5 border border-white/10 overflow-x-auto">
    {#each tabs as tab (tab.value)}
      {@const active = tab.value === value}
      <button
        type="button"
        role="tab"
        aria-selected={active}
        onclick={() => onValueChange(tab.value)}
        class={cn(
          "relative flex items-center gap-2 px-3.5 py-2 rounded-lg text-sm font-medium transition-all whitespace-nowrap",
          active
            ? "bg-gradient-to-br from-red-500 to-pink-600 text-white shadow-lg shadow-red-500/20"
            : "text-gray-400 hover:text-white hover:bg-white/5",
        )}
      >
        {#if tab.icon}
          {@const Icon = tab.icon}
          <Icon class="h-3.5 w-3.5" />
        {/if}
        {tab.label}
      </button>
    {/each}
  </div>
  <div class="min-h-0 overflow-y-auto pr-1 -mr-1 max-h-[60vh]">
    {@render children()}
  </div>
</div>
