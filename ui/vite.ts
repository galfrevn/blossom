import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: "safari16.4",
  },
  test: {
    environment: "happy-dom",
    include: ["**/tests/**/*.{ts,tsx}"],
  },
});
