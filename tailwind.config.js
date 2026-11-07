/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        surface: {
          0: "#0d0d0d",
          1: "#1a1a1a",
          2: "#242424",
          3: "#2e2e2e",
          4: "#383838",
        },
        accent: {
          DEFAULT: "#d4a574",
          hover: "#e0b585",
          muted: "#b8956a",
        },
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "monospace"],
      },
    },
  },
  plugins: [],
};
