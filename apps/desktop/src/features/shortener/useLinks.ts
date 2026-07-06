import { useCallback, useEffect, useState } from "react";
import { listLinks, shortenUrl } from "../../api/links";
import type { LinkItem, ShortenResponse } from "../../api/types";
import { errorText } from "../../lib/errors";

export function useLinks() {
  const [url, setUrl] = useState("");
  const [result, setResult] = useState<ShortenResponse | null>(null);
  const [links, setLinks] = useState<LinkItem[]>([]);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setLinks(await listLinks());
    } catch (e) {
      setError(errorText(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function shorten() {
    setError("");
    setResult(null);
    setBusy(true);
    try {
      const res = await shortenUrl(url);
      setResult(res);
      setUrl("");
      await refresh();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  const ordered = [...links].sort(
    (a, b) => Date.parse(b.created_at) - Date.parse(a.created_at),
  );

  return { url, setUrl, result, links: ordered, error, busy, shorten };
}
