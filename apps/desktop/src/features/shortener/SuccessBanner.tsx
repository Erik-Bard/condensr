import { useState } from "react";
import type { ShortenResponse } from "../../api/types";
import { CheckCircleIcon, CopyIcon } from "../../components/icons";
import { stripScheme } from "../../lib/urls";
import styles from "./SuccessBanner.module.scss";

export function SuccessBanner({ result }: { result: ShortenResponse }) {
  const [copied, setCopied] = useState(false);

  async function copyShortUrl() {
    await navigator.clipboard.writeText(result.short_url);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  return (
    <div className={styles.banner}>
      <CheckCircleIcon />
      <div className={styles.text}>
        <div className={styles.label}>Short link ready</div>
        <div className={styles.url}>{stripScheme(result.short_url)}</div>
      </div>
      <button type="button" className={styles.copyBtn} onClick={copyShortUrl}>
        <CopyIcon />
        <span>{copied ? "Copied" : "Copy"}</span>
      </button>
    </div>
  );
}
