// The triage's health strip: one segmented bar with the share of providers in each state, and the
// same states as filter chips. The bar is a picture of the counts the chips print, so it is hidden
// from the accessibility tree; a chip is a toggle, and pressing the active one shows everything.
import { t } from '../i18n';
import { HEALTH_BUCKETS, type HealthShare, type TriageFilter } from './triageFilter';

export interface HealthStripProps {
  health: readonly HealthShare[];
  total: number;
  filter: TriageFilter;
  onFilter: (filter: TriageFilter) => void;
}

export function HealthStrip({ health, total, filter, onFilter }: HealthStripProps) {
  const countOf = (bucket: TriageFilter) =>
    bucket === 'all' ? total : (health.find((share) => share.bucket === bucket)?.count ?? 0);
  const chips: readonly TriageFilter[] = [
    'all',
    ...HEALTH_BUCKETS.filter((bucket) => countOf(bucket) > 0),
  ];
  return (
    <div className="nx-health">
      <span className="nx-health-bar" aria-hidden="true">
        {health.map((share) => (
          <i
            key={share.bucket}
            data-bucket={share.bucket}
            data-dim={(filter !== 'all' && filter !== share.bucket) || undefined}
            style={{ flexGrow: share.count }}
          />
        ))}
      </span>
      <fieldset className="nx-health-chips" aria-label={t('triage.filter.label')}>
        {chips.map((bucket) => (
          <button
            key={bucket}
            type="button"
            className="nx-health-chip"
            data-bucket={bucket}
            aria-pressed={filter === bucket}
            onClick={() => onFilter(filter === bucket ? 'all' : bucket)}
          >
            <i aria-hidden="true" />
            {t(`triage.filter.${bucket}`)}
            <b>{countOf(bucket)}</b>
          </button>
        ))}
      </fieldset>
    </div>
  );
}
