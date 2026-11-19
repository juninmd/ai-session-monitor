import { harnessName } from '../app/model/harness';
import { t } from '../i18n';
import { formatClock } from '../i18n/format';
import { Logo } from './Bits';
import type { Renewal } from './horizonModel';
import { quotaName } from './wording';

export function RenewalSchedule({
  renewals,
  nowSecs,
}: {
  renewals: readonly Renewal[];
  nowSecs: number;
}) {
  return (
    <details className="nx-renewals">
      <summary>{t('horizon.renewals')}</summary>
      {renewals.length === 0 ? (
        <p className="hint">{t('horizon.no_renewals')}</p>
      ) : (
        <ol>
          {renewals.map(({ id, window, resetsAt }) => (
            <li key={`${id}:${window.id}`}>
              <Logo id={id} size={16} />
              <span>
                <strong>{harnessName(id)}</strong>
                <span className="nx-meta">{quotaName(window)}</span>
              </span>
              <time dateTime={new Date(resetsAt * 1000).toISOString()}>
                {formatClock(resetsAt, nowSecs).label}
              </time>
            </li>
          ))}
        </ol>
      )}
    </details>
  );
}
