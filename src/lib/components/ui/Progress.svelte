<script lang="ts">
  import { cn } from "$lib/utils/cn";

  interface Props {
    value?: number;
    max?: number;
    class?: string;
    indeterminate?: boolean;
  }

  let {
    value = 0,
    max = 100,
    class: className = "",
    indeterminate = false,
  }: Props = $props();

  const percentage = $derived(Math.min(Math.max((value / max) * 100, 0), 100));
</script>

<div
  class={cn("relative h-2 w-full overflow-hidden rounded-full bg-white/5", className)}
  role="progressbar"
  aria-valuenow={value}
  aria-valuemin={0}
  aria-valuemax={max}
>
  {#if indeterminate}
    <div
      class="h-full w-1/3 rounded-full bg-gradient-to-r from-red-500 via-pink-500 to-red-500 absolute animate-[shimmer_1.5s_ease-in-out_infinite]"
      style="background-size: 200% 100%;"
    ></div>
  {:else}
    <div
      class="h-full rounded-full bg-gradient-to-r from-red-500 via-pink-500 to-red-500 transition-[width] duration-300 ease-out"
      style="width: {percentage}%; background-size: 200% 100%;"
    ></div>
  {/if}
</div>
