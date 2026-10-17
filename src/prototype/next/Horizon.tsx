// Horizon: every provider on the same question. Each bar spans from now to that window's reset,
// so a shortfall is a coloured stretch with a size in time, not a percentage to interpret.
import { type KeyboardEvent, useMemo } from 'react';
import { harnessName } from '../../app/harness';
import { durationFormat, type ViewContext } from '../../app/windowModel';
import { Icon } from '../../components/icons';
import { formatDuration } from '../../i18n/format';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { GroupHead, Logo, ShareBar, type Tone } from './Bits';
import { exhaustionLabel, type HorizonRow, horizonOf } from './horizonModel';
import { nxNameId, nxTabId } from './ids';
import { Spark } from './Spark';
import { useRovingTabs } from './useRovingTabs';
import { renewalText } from './wording';

interface HorizonProps {
  views: readonly HarnessView[];
  ctx: ViewContext;
  selected: HarnessId;
  onSelect: (id: HarnessId) => void;
  /** History is only fetched while this is on: one request per provider is not free. */
  trend: boolean;
  onTrend: (on: boolean) => void;
}

function headFigure(row: HorizonRow, ctx: ViewContext): string {
  const fmt = durationFormat(ctx.settings);
  switch (row.kind) {
    case 'out':
      return 'esgotado';
    case 'short':
      return `${formatDuration(row.darkMin, fmt, row.model?.long ?? false)} sem cota`;
    case 'lasts':
      return 'chega ao fim';
    default:
      return '—';
  }
}

function HorizonOption({
  row,
  ctx,
  selected,
  onSelect,
  onKeyDown,
  trend,
}: {
  row: HorizonRow;
  ctx: ViewContext;
  selected: boolean;
  onSelect: (id: HarnessId) => void;
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
        onClick={() => onSelect(id)}
      >
        <span className="nx-h-head">
          <Logo id={id} size={16} />
          <span className="nx-name" id={nxNameId(id)}>
            {name}
          </span>
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
            label={`Tendência de ${name}`}
          />
        )}
      </button>
    </li>
  );
}

export function Horizon({ views, ctx, selected, onSelect, trend, onTrend }: HorizonProps) {
  const horizon = useMemo(() => horizonOf(views, ctx), [views, ctx]);
  const onKeyDown = useRovingTabs(
    horizon.rows.map((row) => row.view.id),
    selected,
    onSelect,
  );
  const covers = horizon.rows.filter((row) => row.kind === 'lasts').length;
  const next = horizon.soonest?.model;
  return (
    <div className="nx-horizon">
      <section className="nx-note">
        <h3>Chego até a renovação?</h3>
        <p>
          Cada barra vai de agora até a renovação da cota que limita o provider. O trecho colorido é
          o tempo que a cota ainda cobre; o resto é tempo sem cota.
        </p>
        <p className="nx-meta">
          {covers} de {horizon.rows.length} cobrem todo o período
          {next
            ? ` · próxima renovação em ${renewalText(next, ctx).replace('renova em ', '')}`
            : ''}
        </p>
        <p className="nx-trend">
          <label>
            <input
              type="checkbox"
              checked={trend}
              onChange={(event) => onTrend(event.target.checked)}
            />
            Mostrar tendência (busca o histórico de cada provider)
          </label>
        </p>
      </section>
      <div className="nx-rows">
        <section className="nx-section">
          <GroupHead title="Por tempo até a renovação" count={horizon.rows.length} />
          <ul className="nx-list">
            {horizon.rows.map((row) => (
              <HorizonOption
                key={row.view.id}
                row={row}
                ctx={ctx}
                selected={row.view.id === selected}
                onSelect={onSelect}
                onKeyDown={onKeyDown}
                trend={trend}
              />
            ))}
          </ul>
          {horizon.rows.length === 0 && (
            <p className="hint">
              <Icon name="clock" /> Nada em leitura ainda.
            </p>
          )}
        </section>
      </div>
    </div>
  );
}
