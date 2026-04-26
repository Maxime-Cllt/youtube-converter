<script lang="ts">
  import { cn } from "$lib/utils/cn";

  interface Props {
    checked?: boolean;
    disabled?: boolean;
    id?: string;
    "aria-label"?: string;
    class?: string;
    onCheckedChange?: (next: boolean) => void;
  }

  let {
    checked = $bindable(false),
    disabled = false,
    id,
    "aria-label": ariaLabel,
    class: className = "",
    onCheckedChange,
  }: Props = $props();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onCheckedChange?.(checked);
  }
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={ariaLabel}
  {id}
  {disabled}
  onclick={toggle}
  class={cn(
    "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-black disabled:cursor-not-allowed disabled:opacity-50",
    checked ? "bg-gradient-to-r from-red-500 to-pink-600" : "bg-white/10",
    className,
  )}
>
  <span
    class={cn(
      "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-lg ring-0 transition-transform duration-200",
      checked ? "translate-x-5" : "translate-x-0",
    )}
  ></span>
</button>
