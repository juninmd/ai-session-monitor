import { leftText, resetMoment, type ViewContext, type WindowModel } from '../../app/windowModel';
import { t } from '../../i18n';
import { formatAmountValue, formatPercent, spokenDuration } from '../../i18n/format';
import { Icon } from '../icons';
import { AvailableLine } from './AvailableLine';
import { EstimateMark } from './EstimateMark';
import { Gauge } from './Gauge';
import { PaceLine } from './PaceLine';

interface FlatWindowProps {
  model: WindowModel;
  ctx: ViewContext;
  stale?: boolean;
}

function RowTime({ model, ctx }: Omit<FlatWindowProps, 'stale'>) {
  if (model.expired) return null; // ResetFoot says it, with the time
  const text = leftText(model, ctx);
  if (text === null || model.leftMin === null) return null;
  return (
    <p className="row-time">
      <span className="vh">{t('window.spoken_left', { dur: spokenDuration(model.leftMin) })}</span>
      <span aria-hidden="true">{text}</span>
    </p>
  );
}

function ResetFoot({ model, ctx }: Omit<FlatWindowProps, 'stale'>) {
  const when = resetMoment(model, ctx);
  if (when === null) return <p className="muted">{t('window.reset_unknown_short')}</p>;
  const prefix = model.expired ? t('window.expired_short') : null;
  const key = model.status === 'out' ? 'window.back_at' : 'window.reset_at';
  return (
    <p className="muted">{prefix ? `${prefix} ${when.label}` : t(key, { when: when.label })}</p>
  );
}

/** A secondary window: title and time on one line, then gauge and percentage. */
export function FlatWindow({ model, ctx, stale = false }: FlatWindowProps) {
  const amount = model.window.amount;
  return (
    <section className="flat" data-status={model.status ?? 'none'}>
      <div className="win-head">
        <h3 className="win-title">
          {model.cadenceGuessed && <EstimateMark />}
          {model.title}
        </h3>
        <RowTime model={model} ctx={ctx} />
      </div>
      <Gauge model={model} ctx={ctx} />
      <div className="meter-foot">
        <AvailableLine model={model} />
        <ResetFoot model={model} ctx={ctx} />
      </div>
      {amount !== null && amount.limit !== null && (
        <p className="muted">
          {t('window.used_of', {
            used: formatAmountValue(amount.used, amount.unit),
            limit: formatAmountValue(amount.limit, amount.unit),
          })}
        </p>
      )}
      {!stale && <PaceLine model={model} ctx={ctx} onlyUrgent className="pace pace-row" />}
      {model.over !== null && (
        <p className="pace pace-row">
          <Icon name="warn" />
          <span>{t('window.over', { pct: formatPercent(model.over) })}</span>
        </p>
      )}
    </section>
  );
}
