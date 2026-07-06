import { ArrowRightIcon, ChevronMark } from "../../components/icons";
import { shortUrlFor, stripScheme } from "../../lib/urls";
import { useRedirectTarget } from "./useRedirectTarget";
import styles from "./RedirectPage.module.scss";

export function RedirectPage({ code }: { code: string }) {
  const destination = useRedirectTarget(code);

  return (
    <div className={styles.wrap}>
      <div className={styles.inner}>
        <div className={styles.tile}>
          <ChevronMark size={26} strokeWidth={2.1} />
        </div>
        <div className={styles.title}>Taking you there…</div>
        <div className={styles.path}>
          <span className={styles.source}>{stripScheme(shortUrlFor(code))}</span>
          {destination && (
            <>
              <ArrowRightIcon />
              <span className={styles.dest}>{stripScheme(destination)}</span>
            </>
          )}
        </div>
        <div className={styles.bar}>
          <span className={styles.barFill} />
        </div>
        <div className={styles.hint}>
          Not redirected?{" "}
          <a href={destination ?? shortUrlFor(code)}>Continue manually</a>
        </div>
      </div>
    </div>
  );
}
