/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        border: "hsl(240 3.7% 15.9%)",
        input: "hsl(240 3.7% 15.9%)",
        ring: "hsl(0 84.2% 60.2%)",
        background: "hsl(0 0% 0%)",
        foreground: "hsl(0 0% 100%)",
        primary: {
          DEFAULT: "hsl(0 84.2% 60.2%)",
          foreground: "hsl(0 0% 100%)",
        },
        secondary: {
          DEFAULT: "hsl(240 3.7% 15.9%)",
          foreground: "hsl(0 0% 100%)",
        },
        destructive: {
          DEFAULT: "hsl(0 84.2% 60.2%)",
          foreground: "hsl(0 0% 100%)",
        },
        muted: {
          DEFAULT: "hsl(240 3.7% 15.9%)",
          foreground: "hsl(240 5% 64.9%)",
        },
        accent: {
          DEFAULT: "hsl(240 3.7% 15.9%)",
          foreground: "hsl(0 0% 100%)",
        },
        popover: {
          DEFAULT: "hsl(0 0% 0%)",
          foreground: "hsl(0 0% 100%)",
        },
        card: {
          DEFAULT: "hsl(0 0% 0%)",
          foreground: "hsl(0 0% 100%)",
        },
      },
      borderRadius: {
        lg: "0.5rem",
        md: "calc(0.5rem - 2px)",
        sm: "calc(0.5rem - 4px)",
      },
    },
  },
  plugins: [],
}

