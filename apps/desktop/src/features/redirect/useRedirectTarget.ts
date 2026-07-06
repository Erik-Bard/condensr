import { useEffect, useState } from "react";
import { listLinks } from "../../api/links";

export function useRedirectTarget(code: string): string | null {
  const [destination, setDestination] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    let timer = 0;
    listLinks()
      .then((items) => {
        if (cancelled) return;
        const match = items.find((item) => item.code === code);
        if (!match) return;
        setDestination(match.long_url);
        timer = window.setTimeout(
          () => window.location.replace(match.long_url),
          900,
        );
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [code]);

  return destination;
}
