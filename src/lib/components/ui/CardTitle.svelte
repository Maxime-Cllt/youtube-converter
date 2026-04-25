<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils/cn";

  let { class: className = "", children }: { class?: string; children: Snippet } = $props();
</script>

<h3 class={cn("text-xl font-bold leading-none tracking-tight flex items-center gap-2", className)}>
  {@render children()}
</h3>
