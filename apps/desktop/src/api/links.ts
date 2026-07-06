import { invoke, isTauri } from "@tauri-apps/api/core";
import { request } from "./client";
import type { LinkItem, ShortenResponse } from "./types";

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
