<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { cn } from "$lib/utils/cn";

  interface Props extends Omit<HTMLInputAttributes, "value" | "class"> {
    value?: string;
    class?: string;
  }

  let {
    value = $bindable(""),
    class: className = "",
    type = "text",
    ...rest
  }: Props = $props();
</script>

<input
  {type}
  bind:value
  class={cn(
    "flex h-11 w-full rounded-xl border-2 border-white/10 bg-white/5 backdrop-blur-xl px-4 py-2.5 text-sm text-white shadow-lg transition-all placeholder:text-gray-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500/50 focus-visible:border-white/20 disabled:cursor-not-allowed disabled:opacity-50 hover:border-white/15",
    className,
  )}
  {...rest}
/>
