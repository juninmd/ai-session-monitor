// The horizon inside a provider's detail: will each window last until its renewal? Every bar spans
// from now to that window's reset, so a shortfall is a stretch with a size in time, not a
// percentage to interpret. The projections come from horizonModel, which reads the shared pace rule.
import { durationFormat, type ViewContext } from '../app/model/windowModel';
import { t } from '../i18n';
import { formatDuration } from '../i18n/format';
import type { HarnessView } from '../lib/contract';
import { ShareBar, type Tone } from './Bits';
import { exhaustionLabel, type HorizonRow, windowRows } from './horizonModel';
import { renewalText } from './wording';

function figure(row: HorizonRow, ctx: ViewContext): string {
  const fmt = durationFormat(ctx.settings);
  switch (row.kind) {
    case 'out':
      return t('horizon.figure.out');
    case 'short':
      return t('horizon.figure.short', {
        duration: formatDuration(row.darkMin, fmt, row.model?.long ?? false),
      });
    case 'lasts':
      return t('horizon.figure.lasts');
    default:
      return '—';
  }
}

export function HorizonBoard({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const rows = windowRows(view, ctx);
  if (rows.length === 0) return null;
  return (
    <section className="nx-horizon" aria-labelledby={`nx-h-${view.id}`}>
      <h3 className="nx-section-head" id={`nx-h-${view.id}`}>
        {t('surface.horizon.label')}
      </h3>
      {rows.map((row) => {
        const exhaust = exhaustionLabel(row, ctx);
        const renewal = renewalText(row.model, ctx);
        const tone: Tone = row.model?.status ?? 'none';
        return (
          <div className="nx-h-row" data-kind={row.kind} key={row.model?.window.id ?? row.label}>
            <span className="nx-h-head">
              <span className="nx-h-quota">{row.windowName}</span>
              <span className="nx-h-figure" data-kind={row.kind}>
                {figure(row, ctx)}
              </span>
            </span>
            <span className="nx-h-track">
              <ShareBar
                pct={row.cover === null ? null : row.cover * 100}
                tone={tone}
                label={`${row.windowName ?? ''}: ${row.label}`}
              />
            </span>
            <span className="nx-h-foot">
              <span className="nx-meta">{row.label}</span>
              <span className="nx-meta nx-h-renew">
                {exhaust === null ? renewal : `${exhaust} · ${renewal}`}
              </span>
            </span>
          </div>
        );
      })}
    </section>
  );
}
