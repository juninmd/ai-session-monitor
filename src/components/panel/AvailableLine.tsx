import type { WindowModel } from '../../app/windowModel';
import { t } from '../../i18n';
import { formatPercent } from '../../i18n/format';

/**
 * "52% disponível", or an explicit "unknown" sentence: never a made-up 0 %. An expired window
 * prints nothing here; its card already says "reiniciou" once, with the time.
 */
export function AvailableLine({ model }: { model: WindowModel }) {
  if (model.expired) return null;
  if (model.available === null) return <p className="muted">{t('window.unknown_pct')}</p>;
  return (
    <p>
      <strong>{formatPercent(model.available)}</strong> {t('window.available')}
    </p>
  );
}
