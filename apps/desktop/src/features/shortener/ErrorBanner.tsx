import { AlertCircleIcon } from "../../components/icons";
import styles from "./ErrorBanner.module.scss";

export function ErrorBanner({ message }: { message: string }) {
  return (
    <div className={styles.banner}>
      <AlertCircleIcon />
      <span>{message}</span>
    </div>
  );
}
