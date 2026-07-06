import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error
const host = process.env.TAURI_DEV_HOST;
// @ts-expect-error
const apiProxyTarget = process.env.CONDENSR_API_URL || "http://localhost:8080";

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    proxy: {
      "/api": apiProxyTarget,
      "/health": apiProxyTarget,
    },
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
