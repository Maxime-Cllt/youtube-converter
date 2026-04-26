<script lang="ts">
  import { Check } from "lucide-svelte";
  import { cn } from "$lib/utils/cn";

  interface Props {
    checked: boolean;
    label: string;
    onToggle: () => void;
    disabled?: boolean;
  }

  let { checked, label, onToggle, disabled = false }: Props = $props();
</script>

<button
  type="button"
  onclick={onToggle}
  {disabled}
  aria-pressed={checked}
  class={cn(
    "inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium border transition-all",
    checked
      ? "bg-gradient-to-r from-red-500/20 to-pink-500/20 border-red-500/40 text-red-200"
      : "bg-white/5 border-white/10 text-gray-400 hover:border-white/20 hover:text-gray-200",
    disabled && "opacity-50 cursor-not-allowed",
  )}
>
  {#if checked}
    <Check class="h-3 w-3" />
  {/if}
  {label}
</button>
