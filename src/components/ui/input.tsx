import * as React from "react"
import { cn } from "@/lib/utils"

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          "flex h-11 w-full rounded-xl border-2 border-white/10 bg-white/5 backdrop-blur-xl px-4 py-2.5 text-sm text-white shadow-lg transition-all file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-white placeholder:text-gray-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500/50 focus-visible:border-white/20 disabled:cursor-not-allowed disabled:opacity-50 hover:border-white/15",
          className
        )}
        ref={ref}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

export { Input }
