import { Fragment } from 'react';
import {
  durationFormat,
  leftText,
  resetMoment,
  type ViewContext,
  type WindowModel,
} from '../../app/windowModel';
import { t } from '../../i18n';
import { durationParts, formatAmountValue, formatPercent, spokenDuration } from '../../i18n/format';
import { Icon } from '../icons';
import { AvailableLine } from './AvailableLine';
import { Gauge } from './Gauge';
import { PaceLine } from './PaceLine';

interface WindowProps {
  model: WindowModel;
  ctx: ViewContext;
  /** Stale data keeps its numbers but drops the pace extrapolation. */
  stale?: boolean;
}

function ResetText({ model, ctx }: WindowProps) {
  if (model.expired) return null; // HeroBody says it, with the time
  const when = resetMoment(model, ctx);
  if (when === null) return null;
  if (model.window.amount) {
    const left = leftText(model, ctx);
    return left === null ? null : (
      <p className="win-reset">{t('window.reset_in', { dur: left })}</p>
    );
  }
  const key = model.status === 'out' ? 'window.back_at' : 'window.reset_at';
  return <p className="win-reset">{t(key, { when: when.label })}</p>;
}

function TimeLeft({ model, ctx }: WindowProps) {
  if (model.leftMin === null) {
    return (
      <p className="hero-time">
        <span className="n">{'—'}</span>
        <span className="cap">{t('window.reset_unknown')}</span>
      </p>
    );
  }
  const parts = durationParts(model.leftMin, durationFormat(ctx.settings), model.long);
  return (
    <p className="hero-time">
      <span className="vh">{t('window.spoken_left', { dur: spokenDuration(model.leftMin) })}</span>
      <span aria-hidden="true">
        {parts.map((p) => (
          <Fragment key={p.u || 'n'}>
            <span className="n">{p.n}</span>
            {p.u && <span className="u">{p.u}</span>}
          </Fragment>
        ))}
      </span>{' '}
      <span className="cap">{t('window.to_reset')}</span>
    </p>
  );
}

function AmountLeft({ model }: { model: WindowModel }) {
  const amount = model.window.amount;
  if (amount === null) return null;
  const used = formatAmountValue(amount.used, amount.unit);
  if (amount.limit === null) {
    return (
      <p className="hero-time">
        <span className="n">{used}</span>
        <span className="cap">{t('window.spent_no_cap')}</span>
      </p>
    );
  }
  const limit = formatAmountValue(amount.limit, amount.unit);
  const available = formatAmountValue(Math.max(0, amount.limit - amount.used), amount.unit);
  return (
    <p className="hero-time">
      <span className="vh">{t('window.spoken_amount', { available, limit })}</span>
      <span className="n" aria-hidden="true">
        {available}
      </span>
      <span className="cap" aria-hidden="true">
        {t('window.available_of', { limit })}
      </span>
    </p>
  );
}

function HeroBody({ model, ctx }: WindowProps) {
  if (model.expired) {
    return (
      <>
        <p className="hero-out">
          <Icon name="refresh" />
          <span>{t('window.expired')}</span>
        </p>
        <p className="muted">
          {t('window.expired_body', { when: resetMoment(model, ctx)?.label ?? '' })}
        </p>
      </>
    );
  }
  if (model.window.amount) return <AmountLeft model={model} />;
  if (model.status === 'out') {
    return (
      <p className="hero-out">
        <Icon name="ban" />
        <span>{t('window.out')}</span>
      </p>
    );
  }
  return <TimeLeft model={model} ctx={ctx} />;
}

/** The window that matters most: time to reset first, then the gauge and the pace sentence. */
export function Hero({ model, ctx, stale = false }: WindowProps) {
  return (
    <section className="card hero" data-status={model.status ?? 'none'}>
      <div className="win-head">
        <h2 className="win-title">{model.title}</h2>
        <ResetText model={model} ctx={ctx} />
      </div>
      <HeroBody model={model} ctx={ctx} />
      <Gauge model={model} ctx={ctx} />
      {!model.expired && (
        <div className="meter-foot">
          <AvailableLine model={model} />
          {!stale && <PaceLine model={model} ctx={ctx} />}
        </div>
      )}
      {model.over !== null && (
        <p className="pace pace-row">
          <Icon name="warn" />
          <span>{t('window.over', { pct: formatPercent(model.over) })}</span>
        </p>
      )}
    </section>
  );
}
