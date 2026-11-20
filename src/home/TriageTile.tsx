// One provider's card on the triage grid: its reading, every quota window it reports and, when the
// source is not simply fine, the reason in words — why it failed, why it is disconnected, how old
// the numbers are. A stale view keeps its last numbers, muted, instead of going blank.
import { type CSSProperties, Fragment, type KeyboardEvent } from 'react';
import { harnessName } from '../app/model/harness';
import { nxNameId, nxTabId } from '../app/model/homeIds';
import { summaryOf, tabChip } from '../app/model/tabModel';
import type { ViewContext, WindowModel } from '../app/model/windowModel';
import { Icon } from '../components/icons';
import { t, tn } from '../i18n';
import { formatAmountValue, formatPercent } from '../i18n/format';
import type { HarnessId, UsageWindow } from '../lib/contract';
import { resetCreditCount } from './actionModel';
import { Logo, Ring, ShareBar, type Tone } from './Bits';
import { compactQuota, gridQuotas } from './quotaModel';
import { brokenHint, readAge } from './tileNote';
import type { TriageRow } from './triageModel';
import { quotaName } from './wording';

/** The number in the card's head: the current reading, or the last one of a stale view, toneless. */
function readingOf(row: TriageRow): { pct: number | null; tone: Tone } {
  if (row.model !== null) return { pct: row.model.shown, tone: row.model.status ?? 'none' };
  return { pct: row.last?.shown ?? null, tone: 'none' };
}

function Amount({ window }: { window: UsageWindow }) {
  const amount = window.amount;
  if (amount === null) return null;
  const used = formatAmountValue(amount.used, amount.unit);
  return (
    <span className="nx-q-amount">
      {amount.limit === null
        ? used
        : t('triage.amount_of', { used, limit: formatAmountValue(amount.limit, amount.unit) })}
    </span>
  );
}

/** Under three hours to the renewal the badge lights up: it is the news a person came for. */
const SOON_MIN = 180;
const isSoon = (model: WindowModel): boolean =>
  !model.expired && model.leftMin !== null && model.leftMin < SOON_MIN;

/** The renewal of one window as a badge; a window with no renewal prints a plain dash. */
function Renewal({ text, soon }: { text: string; soon: boolean }) {
  if (text === '—') return <span className="nx-q-renew">{text}</span>;
  return (
    <span className="nx-q-renew nx-badge" data-soon={soon || undefined}>
      <Icon name="refresh" />
      {text}
    </span>
  );
}

function QuotaRows({ row, ctx }: { row: TriageRow; ctx: ViewContext }) {
  const groups = gridQuotas(row.view);
  if (groups.length === 0) {
    // A broken source explains itself in the hint below; "failed" twice would say nothing.
    if (brokenHint(row.view) !== null) return null;
    const chip = tabChip(row.view, summaryOf(row.view, ctx), ctx);
    return (
      <span className="nx-meta">{chip.kind === 'text' ? chip.text : t('triage.loading')}</span>
    );
  }
  const limiting = (row.model ?? row.last)?.window.id;
  return (
    <>
      {groups.map(({ group, windows }) => (
        <Fragment key={group ?? ''}>
          {group !== null && (
            <span className="nx-tile-group" title={group}>
              {group}
            </span>
          )}
          {windows.map((window) => {
            const quota = compactQuota(row.view, window, ctx);
            return (
              <span
                className="nx-q"
                key={window.id}
                data-limiting={window.id === limiting || undefined}
                data-old={quota.old || undefined}
              >
                <span className="nx-q-text">
                  <span className="nx-q-name" title={quotaName(window)}>
                    {quota.name}
                  </span>
                  <Renewal text={quota.renewal} soon={isSoon(quota.model)} />
                  <span className="nx-q-pct" data-status={quota.status}>
                    {quota.reading}
                  </span>
                </span>
                <ShareBar pct={quota.shown} tone={quota.status ?? 'none'} label={quota.spoken} />
                {!row.view.stale && !quota.model.expired && <Amount window={window} />}
              </span>
            );
          })}
        </Fragment>
      ))}
    </>
  );
}

export function Tile({
  row,
  ctx,
  index,
  selected,
  tabStop,
  onOpen,
  onKeyDown,
}: {
  row: TriageRow;
  ctx: ViewContext;
  /** Position in the grid: the cards come in one after the other. */
  index: number;
  selected: boolean;
  /** The one tile Tab lands on; the arrows move between the rest. */
  tabStop: boolean;
  onOpen: (id: HarnessId) => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
}) {
  const { id } = row.view;
  const pinned = ctx.settings.general.pinned.includes(id);
  const { pct, tone } = readingOf(row);
  const name = harnessName(id);
  const over = row.model?.over ?? null;
  // With quota rows on the card the reason beside the dot is enough; the detail has the rest.
  const hint = row.view.windows.length === 0 ? brokenHint(row.view) : null;
  const age = readAge(row.view, ctx);
  const resets = resetCreditCount(row.view, ctx);
  return (
    <button
      type="button"
      id={nxTabId(id)}
      className="nx-tile"
      aria-pressed={selected}
      tabIndex={tabStop ? 0 : -1}
      data-tier={row.tier}
      data-old={row.view.stale || undefined}
      style={{ '--i': index } as CSSProperties}
      title={row.detail}
      onKeyDown={onKeyDown}
      onClick={() => onOpen(id)}
    >
      <span className="nx-tile-head">
        <span className="nx-logo-ring">
          <Ring pct={pct} tone={tone} />
          <Logo id={id} size={14} />
        </span>
        <span className="nx-tile-id">
          <span className="nx-name" id={nxNameId(id)}>
            {name}
          </span>
          {row.view.plan && <span className="nx-plan">{row.view.plan}</span>}
        </span>
        {pinned && (
          <span className="nx-tile-pin" role="img" aria-label={t('triage.pinned')}>
            <Icon name="pin" />
          </span>
        )}
        {over !== null && (
          <span className="nx-tile-over">
            {'+'}
            {formatPercent(over - 100)}
          </span>
        )}
        <span className="nx-pct" data-status={tone}>
          {formatPercent(pct)}
        </span>
      </span>
      {(row.short !== '' || (resets ?? 0) > 0) && (
        <span className="nx-tile-note" data-tier={row.tier}>
          {row.short !== '' && (
            <>
              <i aria-hidden="true" />
              {row.short}
            </>
          )}
          {resets !== null && resets > 0 && (
            <span
              className="nx-badge nx-badge-reset"
              role="img"
              aria-label={tn('provider_actions.resets', resets)}
              title={tn('provider_actions.resets', resets)}
            >
              <Icon name="bolt" />
              {resets}
            </span>
          )}
        </span>
      )}
      <QuotaRows row={row} ctx={ctx} />
      {hint !== null && <span className="nx-tile-hint">{hint}</span>}
      {age !== null && <span className="nx-tile-age">{age}</span>}
      {row.verdict !== hint && <span className="vh">{row.verdict}</span>}
    </button>
  );
}
