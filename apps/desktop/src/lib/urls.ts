import { isTauri } from "@tauri-apps/api/core";
import { apiBase } from "../api/client";

export function shortOrigin(): string {
  if (apiBase) return apiBase.replace(/\/+$/, "");
  return window.location.origin;
}

export function shortUrlFor(code: string): string {
  return `${shortOrigin()}/${code}`;
}

export function stripScheme(url: string): string {
  return url.replace(/^[a-z][a-z0-9+.-]*:\/\//i, "");
}

export function matchRedirectCode(): string | null {
  if (isTauri()) return null;
  const match = /^\/([A-Za-z0-9_-]{1,64})$/.exec(window.location.pathname);
  return match ? match[1] : null;
}
