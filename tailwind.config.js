/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{svelte,ts,js}"],
  theme: {
    extend: {
      colors: {
        border: "hsl(0 0% 14%)",
        input: "hsl(0 0% 14%)",
        ring: "hsl(0 72% 51%)",
        background: "hsl(0 0% 3%)",
        foreground: "hsl(0 0% 98%)",
        primary: {
          DEFAULT: "hsl(0 72% 51%)",
          foreground: "hsl(0 0% 100%)",
        },
        secondary: {
          DEFAULT: "hsl(0 0% 14%)",
          foreground: "hsl(0 0% 98%)",
        },
        destructive: {
          DEFAULT: "hsl(0 72% 51%)",
          foreground: "hsl(0 0% 100%)",
        },
        muted: {
          DEFAULT: "hsl(0 0% 14%)",
          foreground: "hsl(0 0% 64%)",
        },
        accent: {
          DEFAULT: "hsl(0 0% 14%)",
          foreground: "hsl(0 0% 98%)",
        },
        popover: {
          DEFAULT: "hsl(0 0% 5%)",
          foreground: "hsl(0 0% 98%)",
        },
        card: {
          DEFAULT: "hsl(0 0% 5%)",
          foreground: "hsl(0 0% 98%)",
        },
      },
      borderRadius: {
        lg: "0.75rem",
        md: "0.625rem",
        sm: "0.5rem",
      },
      keyframes: {
        "fade-in": {
          from: { opacity: "0", transform: "translateY(8px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        "scale-in": {
          from: { opacity: "0", transform: "scale(0.96)" },
          to: { opacity: "1", transform: "scale(1)" },
        },
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
        float: {
          "0%, 100%": { transform: "translateY(0px) scale(1)" },
          "50%": { transform: "translateY(-20px) scale(1.03)" },
        },
        "float-reverse": {
          "0%, 100%": { transform: "translateY(0px) scale(1)" },
          "50%": { transform: "translateY(16px) scale(0.97)" },
        },
        "glow-blue": {
          "0%, 100%": { boxShadow: "0 0 0 0 rgba(59,130,246,0)" },
          "50%": { boxShadow: "0 0 28px 4px rgba(59,130,246,0.12)" },
        },
        "glow-purple": {
          "0%, 100%": { boxShadow: "0 0 0 0 rgba(168,85,247,0)" },
          "50%": { boxShadow: "0 0 24px 4px rgba(168,85,247,0.16)" },
        },
        "gradient-x": {
          "0%, 100%": { backgroundPosition: "0% 50%" },
          "50%": { backgroundPosition: "100% 50%" },
        },
        "shimmer-slide": {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(200%)" },
        },
      },
      animation: {
        "fade-in": "fade-in 0.3s ease-out",
        "scale-in": "scale-in 0.2s ease-out",
        shimmer: "shimmer 2s linear infinite",
        float: "float 7s ease-in-out infinite",
        "float-reverse": "float-reverse 9s ease-in-out infinite",
        "glow-blue": "glow-blue 2.5s ease-in-out infinite",
        "glow-purple": "glow-purple 1.8s ease-in-out infinite",
        "gradient-x": "gradient-x 4s ease infinite",
        "shimmer-slide": "shimmer-slide 1.8s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
