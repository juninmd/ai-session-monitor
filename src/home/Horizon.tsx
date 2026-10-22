// Horizon: every provider on the same question. Each bar spans from now to that window's reset,
// so a shortfall is a coloured stretch with a size in time, not a percentage to interpret.
import { type KeyboardEvent, useId, useMemo } from 'react';
import { harnessName } from '../app/harness';
import { nxNameId, nxTabId } from '../app/homeIds';
import { durationFormat, leftText, type ViewContext } from '../app/windowModel';
import { Icon } from '../components/icons';
import { t } from '../i18n';
import { formatDuration } from '../i18n/format';
import type { HarnessId, HarnessView } from '../lib/contract';
import { GroupHead, Logo, ShareBar, type Tone } from './Bits';
import { exhaustionLabel, type HorizonRow, horizonOf } from './horizonModel';
import { Spark } from './Spark';
import { useRovingTabs } from './useRovingTabs';
import { renewalText } from './wording';

interface HorizonProps {
  views: readonly HarnessView[];
  ctx: ViewContext;
  selected: HarnessId;
  /** Move the highlight, as an arrow key does. */
  onSelect: (id: HarnessId) => void;
  /** Activate one provider and open its lateral detail. */
  onOpen: (id: HarnessId) => void;
  /** History is only fetched while this is on: one request per provider is not free. */
  trend: boolean;
  onTrend: (on: boolean) => void;
}

function headFigure(row: HorizonRow, ctx: ViewContext): string {
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

function HorizonOption({
  row,
  ctx,
  selected,
  onOpen,
  onKeyDown,
  trend,
}: {
  row: HorizonRow;
  ctx: ViewContext;
  selected: boolean;
  onOpen: (id: HarnessId) => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  trend: boolean;
}) {
  const { id } = row.view;
  const name = harnessName(id);
  const cover = row.cover === null ? null : row.cover * 100;
  const exhaust = exhaustionLabel(row, ctx);
  // The fill carries the same status the rest of the app prints; the uncovered stretch of the
  // track is what says "no quota here", in CSS, from `kind`.
  const tone: Tone = row.model?.status ?? 'none';
  const renewal = renewalText(row.model, ctx);
  return (
    <li>
      <button
        type="button"
        id={nxTabId(id)}
        className="nx-h-row"
        aria-pressed={selected}
        tabIndex={selected ? 0 : -1}
        data-kind={row.kind}
        onKeyDown={onKeyDown}
        onClick={() => onOpen(id)}
      >
        <span className="nx-h-head">
          <Logo id={id} size={16} />
          <span className="nx-name" id={nxNameId(id)}>
            {name}
          </span>
          {row.view.plan && <span className="nx-plan">{row.view.plan}</span>}
          {row.windowName !== null && <span className="nx-h-quota">{row.windowName}</span>}
          <span className="nx-h-figure" data-kind={row.kind}>
            {headFigure(row, ctx)}
          </span>
        </span>
        <span className="nx-h-track">
          <ShareBar pct={cover} tone={tone} label={`${name}: ${row.label}`} />
        </span>
        <span className="nx-h-foot">
          <span className="nx-meta">{row.label}</span>
          <span className="nx-meta nx-h-renew">
            {exhaust === null ? renewal : `${exhaust} · ${renewal}`}
          </span>
        </span>
        {trend && (
          <Spark
            view={row.view}
            window={row.model?.window ?? null}
            label={t('horizon.trend_of', { name })}
          />
        )}
      </button>
    </li>
  );
}

export function Horizon({ views, ctx, selected, onSelect, onOpen, trend, onTrend }: HorizonProps) {
  const tipId = useId();
  const horizon = useMemo(() => horizonOf(views, ctx), [views, ctx]);
  const onKeyDown = useRovingTabs(
    horizon.rows.map((row) => row.view.id),
    selected,
    onSelect,
  );
  const covers = horizon.rows.filter((row) => row.kind === 'lasts').length;
  const next = horizon.soonest?.model;
  const nextLeft = next ? leftText(next, ctx) : null;
  const total = horizon.rows.length;
  // The per-row trend costs one history request per provider, so it also waits for the setting.
  const showTrend = trend && ctx.settings.general.historyEnabled;
  return (
    <div className="nx-horizon">
      <div className="nx-h-bar">
        <span className="nx-info">
          <button
            type="button"
            className="icon-btn"
            aria-label={t('horizon.about')}
            aria-describedby={tipId}
          >
            <Icon name="info" />
          </button>
          <span className="nx-tip" role="tooltip" id={tipId}>
            <strong>{t('horizon.tip.title')}</strong>
            <span>{t('horizon.tip.body')}</span>
            <span className="nx-meta">
              {nextLeft === null
                ? t('horizon.tip.covers', { covers, total })
                : t('horizon.tip.covers_next', { covers, total, left: nextLeft })}
            </span>
          </span>
        </span>
        <label className="nx-trend">
          <input
            type="checkbox"
            checked={showTrend}
            onChange={(event) => onTrend(event.target.checked)}
          />
          {t('horizon.trend_toggle')}
        </label>
      </div>
      <div className="nx-rows">
        <section className="nx-section">
          <GroupHead title={t('horizon.by_renewal')} count={total} />
          <ul className="nx-list">
            {horizon.rows.map((row) => (
              <HorizonOption
                key={row.view.id}
                row={row}
                ctx={ctx}
                selected={row.view.id === selected}
                onOpen={onOpen}
                onKeyDown={onKeyDown}
                trend={showTrend}
              />
            ))}
          </ul>
          {horizon.rows.length === 0 && (
            <p className="hint">
              <Icon name="clock" /> {t('horizon.empty')}
            </p>
          )}
        </section>
      </div>
    </div>
  );
}
