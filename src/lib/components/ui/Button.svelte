<script lang="ts" module>
  export type ButtonVariant =
    | "default"
    | "destructive"
    | "outline"
    | "secondary"
    | "ghost"
    | "link";
  export type ButtonSize = "default" | "sm" | "lg" | "icon";

  const variants: Record<ButtonVariant, string> = {
    default:
      "bg-gradient-to-r from-red-500 to-pink-600 text-white shadow-lg shadow-red-500/25 hover:shadow-xl hover:shadow-red-500/40 hover:scale-[1.02] active:scale-[0.98]",
    destructive:
      "bg-gradient-to-r from-red-600 to-red-700 text-white shadow-lg hover:shadow-xl hover:scale-[1.02] active:scale-[0.98]",
    outline:
      "border-2 border-white/10 bg-white/5 backdrop-blur-xl hover:bg-white/10 hover:border-white/20 text-white",
    secondary:
      "bg-white/10 backdrop-blur-xl text-white hover:bg-white/15 hover:scale-[1.02] active:scale-[0.98]",
    ghost: "hover:bg-white/10 hover:text-white text-gray-300",
    link: "text-red-500 underline-offset-4 hover:underline",
  };

  const sizes: Record<ButtonSize, string> = {
    default: "h-11 px-6 py-2.5",
    sm: "h-9 rounded-lg px-4 text-xs",
    lg: "h-12 rounded-xl px-8 text-base",
    icon: "h-10 w-10",
  };
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { cn } from "$lib/utils/cn";

  interface Props extends HTMLButtonAttributes {
    variant?: ButtonVariant;
    size?: ButtonSize;
    class?: string;
    children: Snippet;
  }

  let {
    variant = "default",
    size = "default",
    class: className = "",
    children,
    type = "button",
    ...rest
  }: Props = $props();

  const base =
    "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-semibold transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-black disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0";
</script>

<button {type} class={cn(base, variants[variant], sizes[size], className)} {...rest}>
  {@render children()}
</button>
