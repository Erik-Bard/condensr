import { AppHeader } from "./AppHeader";
import { ErrorBanner } from "./ErrorBanner";
import { LinkTable } from "./LinkTable";
import { ShortenForm } from "./ShortenForm";
import { SuccessBanner } from "./SuccessBanner";
import { useLinks } from "./useLinks";
import styles from "./ShortenerPage.module.scss";

export function ShortenerPage() {
  const { url, setUrl, result, links, error, busy, shorten } = useLinks();
  const countLabel = `${links.length} ${links.length === 1 ? "link" : "links"}`;

  return (
    <>
      <AppHeader countLabel={countLabel} />
      <div className={styles.body}>
        <div className={styles.column}>
          <ShortenForm
            url={url}
            onUrlChange={setUrl}
            busy={busy}
            hasError={Boolean(error)}
            onSubmit={shorten}
          />
          {error && <ErrorBanner message={error} />}
          {result && <SuccessBanner key={result.code} result={result} />}
          <LinkTable
            links={links}
            newCode={result?.code ?? null}
            countLabel={countLabel}
          />
        </div>
      </div>
    </>
  );
}
