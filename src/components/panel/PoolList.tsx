import type { Pool, PoolRow } from '../../app/layout';
import { leftText, type ViewContext } from '../../app/windowModel';
import { t } from '../../i18n';
import { formatPercent } from '../../i18n/format';
import { Gauge } from './Gauge';

interface RowsProps {
  rows: readonly PoolRow[];
  ctx: ViewContext;
  /** Prefix for the gauge's accessible name ("Gemini, 5 horas"). */
  scope?: string;
  withTime?: boolean;
}

function Row({ row, ctx, scope, withTime }: { row: PoolRow } & Omit<RowsProps, 'rows'>) {
  const { model, name } = row;
  const pct = model.expired || model.available === null ? '—' : formatPercent(model.available);
  const time = model.expired ? t('window.expired_short') : leftText(model, ctx);
  return (
    <li className={withTime ? 'li with-time' : 'li'} data-status={model.status ?? 'none'}>
      <span className="li-name" title={name}>
        {name}
      </span>
      <Gauge model={model} ctx={ctx} size="sm" title={scope ? `${scope}, ${name}` : name} />
      <span className="li-pct">{pct}</span>
      {withTime && <span className="li-time">{time ?? ''}</span>}
    </li>
  );
}

function Rows({ rows, ctx, scope, withTime }: RowsProps) {
  return (
    <ul className="list">
      {rows.map((row) => (
        <Row key={row.model.window.id} row={row} ctx={ctx} scope={scope} withTime={withTime} />
      ))}
    </ul>
  );
}

/** One quota pool (a model family): its windows as compact rows with their time left. */
export function PoolCard({ pool, ctx }: { pool: Pool; ctx: ViewContext }) {
  return (
    <section className="card pool">
      <h2 className="win-title">{pool.title}</h2>
      {pool.note && <p className="hint">{pool.note}</p>}
      <Rows rows={pool.rows} ctx={ctx} scope={pool.title} withTime />
    </section>
  );
}

/** Single-window groups side by side ("Semana por modelo"). */
export function GroupList({
  title,
  rows,
  ctx,
}: {
  title: string;
  rows: readonly PoolRow[];
  ctx: ViewContext;
}) {
  return (
    <section className="flat">
      <div className="win-head">
        <h3 className="win-title">{title}</h3>
      </div>
      <Rows rows={rows} ctx={ctx} scope={title} />
    </section>
  );
}
