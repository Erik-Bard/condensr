import type { LinkItem } from "../../api/types";
import { EmptyState } from "./EmptyState";
import { LinkRow } from "./LinkRow";
import styles from "./LinkTable.module.scss";

export function LinkTable({
  links,
  newCode,
  countLabel,
}: {
  links: LinkItem[];
  newCode: string | null;
  countLabel: string;
}) {
  return (
    <section className={styles.section}>
      <div className={styles.sectionHeader}>
        <span className={styles.title}>Recent links</span>
        <span className={styles.count}>{countLabel}</span>
      </div>

      {links.length === 0 ? (
        <EmptyState />
      ) : (
        <div className={styles.table}>
          <div className={styles.head}>
            <span>Short link</span>
            <span>Destination</span>
            <span>Created</span>
            <span className={styles.headOpen}>Open</span>
          </div>
          {links.map((link) => (
            <LinkRow key={link.id} link={link} isNew={newCode === link.code} />
          ))}
        </div>
      )}
    </section>
  );
}
