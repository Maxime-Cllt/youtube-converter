<script lang="ts">
  import type { HTMLSelectAttributes } from "svelte/elements";
  import { cn } from "$lib/utils/cn";

  interface Option {
    value: string;
    label: string;
  }

  interface Props extends Omit<HTMLSelectAttributes, "value" | "class"> {
    value?: string;
    options: Option[];
    class?: string;
  }

  let {
    value = $bindable(""),
    options,
    class: className = "",
    ...rest
  }: Props = $props();
</script>

<select
  bind:value
  class={cn("ui-select", className)}
  {...rest}
>
  {#each options as opt (opt.value)}
    <option value={opt.value}>{opt.label}</option>
  {/each}
</select>

<style>
  .ui-select {
    display: flex;
    height: 2.75rem;
    width: 100%;
    border-radius: 0.75rem;
    border: 2px solid rgb(255 255 255 / 0.1);
    background-color: rgb(255 255 255 / 0.05);
    backdrop-filter: blur(16px);
    padding: 0.625rem 2.5rem 0.625rem 1rem;
    font-size: 0.875rem;
    color: white;
    box-shadow:
      0 10px 15px -3px rgb(0 0 0 / 0.1),
      0 4px 6px -4px rgb(0 0 0 / 0.1);
    transition: all 0.15s ease;
    cursor: pointer;
    appearance: none;
    background-repeat: no-repeat;
    background-position: right 1rem center;
    background-size: 12px 12px;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='12' fill='%23a1a1aa' viewBox='0 0 16 16'><path d='M7.247 11.14 2.451 5.658C1.885 5.013 2.345 4 3.204 4h9.592a1 1 0 0 1 .753 1.659l-4.796 5.48a1 1 0 0 1-1.506 0z'/></svg>");
  }

  .ui-select:hover {
    border-color: rgb(255 255 255 / 0.15);
  }

  .ui-select:focus-visible {
    outline: none;
    border-color: rgb(255 255 255 / 0.2);
    box-shadow: 0 0 0 2px rgb(239 68 68 / 0.5);
  }

  .ui-select:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .ui-select option {
    background-color: rgb(17 24 39);
    color: white;
  }
</style>
