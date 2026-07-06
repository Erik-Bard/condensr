import { LinkGlyph } from "../../components/icons";
import styles from "./EmptyState.module.scss";

export function EmptyState() {
  return (
    <div className={styles.card}>
      <div className={styles.tile}>
        <LinkGlyph size={22} />
      </div>
      <div className={styles.heading}>No links yet</div>
      <div className={styles.hint}>
        Paste a URL above and hit Shorten. Your condensed links will collect
        here.
      </div>
    </div>
  );
}
