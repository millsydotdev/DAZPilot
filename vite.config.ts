import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  root: "src",
  base: "./",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["react", "react-dom", "lucide-react", "zustand"],
          tauri: ["@tauri-apps/api", "@tauri-apps/plugin-process", "@tauri-apps/plugin-updater"],
        },
      },
    },
  },
});