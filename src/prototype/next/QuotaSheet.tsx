// The quota sheet: one ficha per window, grouped by the pool it belongs to. It reuses the meter,
// the available line and the pace sentence the current interface already shows, so the numbers and
// their rules are identical — only the layout is new.
//
// A stale or expired window has no current number and no pace: the percentage, the amount and the
// projection belong to a reading, and an old reading has none. The card says so and stops there.
import { tq } from '../../app/quota';
import {
  durationFormat,
  resetMoment,
  type ViewContext,
  type WindowModel,
} from '../../app/windowModel';
import { EstimateMark } from '../../components/panel/EstimateMark';
import { Gauge } from '../../components/panel/Gauge';
import { PaceLine } from '../../components/panel/PaceLine';
import { t } from '../../i18n';
import { durationParts, formatAmountValue, formatPercent, spokenDuration } from '../../i18n/format';
import type { HarnessView } from '../../lib/contract';
import { sheetOf } from './quotas';

const STATUS_WORD: Record<string, string> = {
  ok: 'tranquilo',
  warn: 'no limite',
  crit: 'crítico',
  out: 'esgotado',
};

function AmountLine({ model }: { model: WindowModel }) {
  const amount = model.window.amount;
  if (amount === null) return null;
  const used = formatAmountValue(amount.used, amount.unit);
  if (amount.limit === null) return <p className="nx-chip-amount">{used} gastos, sem teto</p>;
  return (
    <p className="nx-chip-amount">
      {used} de {formatAmountValue(amount.limit, amount.unit)}
      <span className="muted">
        {' '}
        · {formatAmountValue(Math.max(0, amount.limit - amount.used), amount.unit)} livres
      </span>
    </p>
  );
}

function RenewLine({ model, ctx }: { model: WindowModel; ctx: ViewContext }) {
  const when = resetMoment(model, ctx);
  if (model.expired) {
    return (
      <p className="nx-chip-renew">{when === null ? 'reiniciou' : `reiniciou às ${when.text}`}</p>
    );
  }
  if (model.leftMin === null) {
    return <p className="nx-chip-renew">{t('window.reset_unknown_short')}</p>;
  }
  const parts = durationParts(model.leftMin, durationFormat(ctx.settings), model.long);
  return (
    <p className="nx-chip-renew">
      renova em{' '}
      {parts.map((part) => (
        <span key={part.u || 'n'}>
          <b>{part.n}</b>
          {part.u}
          {part.spaced ? ' ' : ''}
        </span>
      ))}
      {when !== null && <span className="muted"> · {when.label}</span>}
    </p>
  );
}

function WindowCard({
  model,
  ctx,
  stale,
  depth,
}: {
  model: WindowModel;
  ctx: ViewContext;
  stale: boolean;
  /** 3 above a pool, 4 inside one, so the document never skips a level. */
  depth: 3 | 4;
}) {
  // A stale reading cannot be printed as a current one, and an expired window has no status.
  const silent = stale || model.expired;
  const display: WindowModel = silent ? { ...model, shown: null, status: null, over: null } : model;
  const status = display.status ?? null;
  const Heading = depth === 4 ? 'h4' : 'h3';
  return (
    <article className="nx-chip" data-status={status ?? 'none'}>
      <header className="nx-chip-head">
        <Heading className="nx-chip-title">
          {model.cadenceGuessed && <EstimateMark />}
          {model.title}
        </Heading>
        {status !== null && <span className="nx-chip-status">{STATUS_WORD[status]}</span>}
        <span className="nx-chip-pct" data-status={status ?? 'none'}>
          {formatPercent(display.shown)}
        </span>
      </header>
      <Gauge model={display} ctx={ctx} />
      <div className="nx-chip-foot">
        <p className="nx-chip-avail">
          {display.shown === null ? (
            <span className="muted">{t('window.unknown_pct')}</span>
          ) : (
            <>
              <b>{formatPercent(display.shown)}</b> {tq('window.available', display.showUsed)}
            </>
          )}
        </p>
        <RenewLine model={model} ctx={ctx} />
      </div>
      {!silent && <AmountLine model={model} />}
      {!silent && <PaceLine model={model} ctx={ctx} className="pace nx-chip-pace" />}
      <span className="vh">
        {model.leftMin === null ? '' : `${spokenDuration(model.leftMin)} até renovar.`}
      </span>
    </article>
  );
}

/** Every window the provider reports, pools first, with nothing merged and nothing hidden. */
export function QuotaSheet({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const sheet = sheetOf(view, ctx);
  if (sheet.total === 0) {
    return <p className="hint">{t('window.none')}</p>;
  }
  return (
    <div className="nx-sheet">
      {sheet.groups.map((group) => (
        <section className="nx-sheet-group" key={group.id ?? 'plain'}>
          {group.title !== null && <h3 className="nx-sheet-pool">{group.title}</h3>}
          {group.windows.map((model) => (
            <WindowCard
              key={model.window.id}
              model={model}
              ctx={ctx}
              stale={view.stale}
              depth={group.title === null ? 3 : 4}
            />
          ))}
        </section>
      ))}
    </div>
  );
}
