<script lang="ts">
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils/cn";

  interface Props {
    class?: string;
    children: Snippet;
  }

  let { class: className = "", children }: Props = $props();
</script>

<div
  class={cn(
    "rounded-2xl border border-white/10 bg-white/[0.04] backdrop-blur-2xl text-white shadow-xl transition-colors duration-300 hover:border-white/20",
    className,
  )}
>
  {@render children()}
</div>
