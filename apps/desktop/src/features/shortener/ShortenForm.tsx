import type { FormEvent } from "react";
import { LinkGlyph } from "../../components/icons";
import styles from "./ShortenForm.module.scss";

export function ShortenForm({
  url,
  onUrlChange,
  busy,
  hasError,
  onSubmit,
}: {
  url: string;
  onUrlChange: (url: string) => void;
  busy: boolean;
  hasError: boolean;
  onSubmit: () => void;
}) {
  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    onSubmit();
  }

  return (
    <form className={styles.form} onSubmit={handleSubmit}>
      <div
        className={
          hasError ? `${styles.field} ${styles.fieldError}` : styles.field
        }
      >
        <LinkGlyph size={17} />
        <input
          className={styles.input}
          value={url}
          onChange={(e) => onUrlChange(e.currentTarget.value)}
          placeholder="Paste a long URL to condense…"
          spellCheck={false}
          autoCorrect="off"
          autoCapitalize="off"
        />
        {busy && <span className={styles.spinner} />}
      </div>
      <button
        type="submit"
        className={
          busy ? `${styles.submit} ${styles.submitLoading}` : styles.submit
        }
        disabled={busy || !url}
      >
        {busy ? "Shortening…" : "Shorten"}
      </button>
    </form>
  );
}
