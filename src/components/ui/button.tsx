import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-semibold transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500/50 focus-visible:ring-offset-2 focus-visible:ring-offset-black disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default:
          "bg-gradient-to-r from-red-500 to-pink-600 text-white shadow-lg shadow-red-500/25 hover:shadow-xl hover:shadow-red-500/40 hover:scale-[1.02] active:scale-[0.98]",
        destructive:
          "bg-gradient-to-r from-red-600 to-red-700 text-white shadow-lg hover:shadow-xl hover:scale-[1.02] active:scale-[0.98]",
        outline:
          "border-2 border-white/10 bg-white/5 backdrop-blur-xl hover:bg-white/10 hover:border-white/20",
        secondary:
          "bg-white/10 backdrop-blur-xl text-white hover:bg-white/15 hover:scale-[1.02] active:scale-[0.98]",
        ghost: "hover:bg-white/10 hover:text-white",
        link: "text-red-500 underline-offset-4 hover:underline",
      },
      size: {
        default: "h-11 px-6 py-2.5",
        sm: "h-9 rounded-lg px-4 text-xs",
        lg: "h-12 rounded-xl px-8 text-base",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <button
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button, buttonVariants }
