import { t } from '../../i18n';
import { formatAge } from '../../i18n/format';

interface FooterProps {
  /** Epoch seconds of the newest read, null before the first one. */
  readAt: number | null;
  nowSecs: number;
  onRefresh: () => void;
  onQuit: () => void;
}

export function Footer({ readAt, nowSecs, onRefresh, onQuit }: FooterProps) {
  const text =
    readAt === null
      ? t('footer.never')
      : t('footer.read_ago', { age: formatAge(Math.max(0, nowSecs - readAt)) });
  return (
    <footer className="fly-foot">
      <span>{text}</span>
      <span className="foot-actions">
        <button type="button" className="link-btn" onClick={onRefresh}>
          {t('footer.refresh')}
        </button>
        <button type="button" className="link-btn" onClick={onQuit}>
          {t('footer.quit')}
        </button>
      </span>
    </footer>
  );
}
