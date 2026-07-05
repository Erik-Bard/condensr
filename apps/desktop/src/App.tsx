import { useEffect, useState, type FormEvent } from "react";
import {
  listLinks,
  shortenUrl,
  type LinkItem,
  type ShortenResponse,
} from "./api";
import "./App.css";

function App() {
  const [url, setUrl] = useState("");
  const [result, setResult] = useState<ShortenResponse | null>(null);
  const [links, setLinks] = useState<LinkItem[]>([]);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function refreshLinks() {
    try {
      setLinks(await listLinks());
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    refreshLinks();
  }, []);

  async function onShorten(e: FormEvent) {
    e.preventDefault();
    setError("");
    setBusy(true);
    try {
      const res = await shortenUrl(url);
      setResult(res);
      setUrl("");
      await refreshLinks();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="container">
      <h1>condensr</h1>

      <form className="row" onSubmit={onShorten}>
        <input
          value={url}
          onChange={(e) => setUrl(e.currentTarget.value)}
          placeholder="https://example.com/very/long/url"
        />
        <button type="submit" disabled={busy || !url}>
          {busy ? "Shortening…" : "Shorten"}
        </button>
      </form>

      {error && <p className="error">{error}</p>}

      {result && (
        <p className="result">
          <strong>{result.code}</strong> →{" "}
          <a href={result.short_url} target="_blank" rel="noreferrer">
            {result.short_url}
          </a>
        </p>
      )}

      <h2>Recent links</h2>
      <ul className="links">
        {links.length === 0 && <li className="muted">No links yet.</li>}
        {links.map((l) => (
          <li key={l.id}>
            <code>{l.code}</code> <span className="muted">{l.long_url}</span>
          </li>
        ))}
      </ul>
    </main>
  );
}

export default App;
