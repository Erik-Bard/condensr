import { RedirectPage } from "./features/redirect/RedirectPage";
import { ShortenerPage } from "./features/shortener/ShortenerPage";
import { matchRedirectCode } from "./lib/urls";
import styles from "./App.module.scss";

function App() {
  const redirectCode = matchRedirectCode();
  return (
    <div className={styles.app}>
      {redirectCode ? <RedirectPage code={redirectCode} /> : <ShortenerPage />}
    </div>
  );
}

export default App;
