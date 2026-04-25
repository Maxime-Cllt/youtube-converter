<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils/cn";

  let { class: className = "", children }: { class?: string; children: Snippet } = $props();
</script>

<div class={cn("flex flex-col space-y-2 p-6 pb-4", className)}>
  {@render children()}
</div>
