<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    label: string;
    hint?: string;
    for?: string;
    children: Snippet;
    class?: string;
  }

  let {
    label,
    hint,
    for: forId,
    children,
    class: className = "",
  }: Props = $props();
</script>

<div class="space-y-1.5 {className}">
  <label class="text-sm font-semibold text-gray-200" for={forId}>{label}</label>
  {@render children()}
  {#if hint}
    <p class="text-xs text-gray-500 leading-relaxed">{hint}</p>
  {/if}
</div>
