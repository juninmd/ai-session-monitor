import { t } from '../../i18n';
import { formatAge } from '../../i18n/format';

interface FooterProps {
  /** Epoch seconds of the newest read, null before the first one. */
  readAt: number | null;
  nowSecs: number;
}

export function Footer({ readAt, nowSecs }: FooterProps) {
  const text =
    readAt === null
      ? t('footer.never')
      : t('footer.read_ago', { age: formatAge(Math.max(0, nowSecs - readAt)) });
  return (
    <footer className="fly-foot">
      <span>{text}</span>
    </footer>
  );
}
