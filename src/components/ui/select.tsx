import * as React from "react"
import { cn } from "@/lib/utils"

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  options: { value: string; label: string }[]
}

const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, options, ...props }, ref) => {
    return (
      <select
        className={cn(
          "flex h-11 w-full rounded-xl border-2 border-white/10 bg-white/5 backdrop-blur-xl px-4 py-2.5 text-sm text-white shadow-lg transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500/50 focus-visible:border-white/20 disabled:cursor-not-allowed disabled:opacity-50 hover:border-white/15 cursor-pointer",
          className
        )}
        ref={ref}
        {...props}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value} className="bg-gray-900 text-white">
            {option.label}
          </option>
        ))}
      </select>
    )
  }
)
Select.displayName = "Select"

export { Select }
