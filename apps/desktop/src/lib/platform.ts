import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

export function openExternal(url: string) {
  if (isTauri()) {
    void openUrl(url);
    return;
  }
  window.open(url, "_blank", "noopener,noreferrer");
}
