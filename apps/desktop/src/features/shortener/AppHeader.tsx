import { ChevronMark } from "../../components/icons";
import styles from "./AppHeader.module.scss";

export function AppHeader({ countLabel }: { countLabel: string }) {
  return (
    <header className={styles.header}>
      <div className={styles.brand}>
        <div className={styles.logoTile}>
          <ChevronMark size={15} strokeWidth={2.1} />
        </div>
        <span className={styles.wordmark}>condensr</span>
      </div>
      <div className={styles.status}>
        <span className={styles.statusDot} />
        <span>{countLabel}</span>
      </div>
    </header>
  );
}
