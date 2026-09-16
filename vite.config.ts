import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri сервира фронтенда от фиксиран порт в режим на разработка.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "chrome105",
    outDir: "dist",
    emptyOutDir: true,
    sourcemap: false,
  },
});
