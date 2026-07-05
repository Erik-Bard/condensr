import { invoke, isTauri } from "@tauri-apps/api/core";

export interface ShortenResponse {
  code: string;
  short_url: string;
  long_url: string;
}

export interface LinkItem {
  id: number;
  long_url: string;
  created_at: string;
  code: string;
}

const apiBase: string = import.meta.env.VITE_API_URL ?? "";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiBase}${path}`, init);
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new Error(
      body?.description ?? `request failed with status ${response.status}`,
    );
  }
  return response.json();
}

export function shortenUrl(url: string): Promise<ShortenResponse> {
  if (isTauri()) {
    return invoke<ShortenResponse>("shorten", { url });
  }
  return request<ShortenResponse>("/api/shorten", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ url }),
  });
}

export function listLinks(): Promise<LinkItem[]> {
  if (isTauri()) {
    return invoke<LinkItem[]>("list_links");
  }
  return request<LinkItem[]>("/api/links");
}
