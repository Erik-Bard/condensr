import type { LinkItem } from "../../api/types";
import { OpenIcon } from "../../components/icons";
import { openExternal } from "../../lib/platform";
import { relativeTime } from "../../lib/time";
import { stripScheme } from "../../lib/urls";
import styles from "./LinkTable.module.scss";

export function LinkRow({ link, isNew }: { link: LinkItem; isNew: boolean }) {
  const shortUrl = link.short_url;

  return (
    <div className={isNew ? `${styles.row} ${styles.rowNew}` : styles.row}>
      <span className={styles.cellShort}>{stripScheme(shortUrl)}</span>
      <span className={styles.cellDest}>{link.long_url}</span>
      <span className={styles.cellCreated}>
        {isNew ? "just now" : relativeTime(link.created_at)}
      </span>
      <span className={styles.cellOpen}>
        <button
          type="button"
          className={styles.openBtn}
          onClick={() => openExternal(shortUrl)}
          aria-label={`Open ${stripScheme(shortUrl)}`}
        >
          <OpenIcon />
        </button>
      </span>
    </div>
  );
}
