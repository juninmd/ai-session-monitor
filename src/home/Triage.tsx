// Triage: the proposed home. One headline answers "do I need to act?"; below it every provider sits
// in one grid of two per row, ordered by urgency rather than by registration. The wider card is
// spent on detail — each provider's own quota windows with their reading and renewal — while the
// alarm stays in the headline and the card itself only states the facts.
import { type KeyboardEvent, useMemo } from 'react';
import { harnessName } from '../app/harness';
import { nxNameId, nxTabId } from '../app/homeIds';
import { showsUsed } from '../app/quota';
import { summaryOf, tabChip } from '../app/tabModel';
import type { ViewContext } from '../app/windowModel';
import { Icon } from '../components/icons';
import { formatAmountValue, formatPercent } from '../i18n/format';
import type { HarnessId, HarnessView, UsageWindow } from '../lib/contract';
import { GroupHead, Logo, ShareBar, type Tone } from './Bits';
import { compactQuota, gridQuotas } from './quotaModel';
import { headlineOf, type TriageBoard, type TriageRow, triageOf } from './triageModel';
import { useRovingTabs } from './useRovingTabs';
import { quotaName } from './wording';

export interface TriageProps {
  views: readonly HarnessView[];
  ctx: ViewContext;
  selected: HarnessId;
  /** Move the highlight, as an arrow key does. */
  onSelect: (id: HarnessId) => void;
  /** Activate one provider and open its lateral detail. */
  onOpen: (id: HarnessId) => void;
  onConfigure: () => void;
}

/** How many windows one card opens with; the rest stay one click away in the detail. */
function readingOf(row: TriageRow): { pct: number | null; tone: Tone } {
  if (row.model === null) return { pct: null, tone: 'none' };
  return { pct: row.model.shown, tone: row.model.status ?? 'none' };
}

function Amount({ window }: { window: UsageWindow }) {
  const amount = window.amount;
  if (amount === null) return null;
  const used = formatAmountValue(amount.used, amount.unit);
  return (
    <span className="nx-q-amount">
      {amount.limit === null ? used : `${used} de ${formatAmountValue(amount.limit, amount.unit)}`}
    </span>
  );
}

function QuotaRows({ row, ctx }: { row: TriageRow; ctx: ViewContext }) {
  const quotas = gridQuotas(row.view, ctx);
  if (quotas.windows.length === 0) {
    const chip = tabChip(row.view, summaryOf(row.view, ctx), ctx);
    return <span className="nx-meta">{chip.kind === 'text' ? chip.text : 'Carregando'}</span>;
  }
  const limiting = row.model?.window.id;
  return (
    <>
      {quotas.group !== null && (
        <span className="nx-tile-group" title={quotas.group}>
          {quotas.group}
        </span>
      )}
      {quotas.windows.map((window) => {
        const quota = compactQuota(row.view, window, ctx);
        return (
          <span
            className="nx-q"
            key={window.id}
            data-limiting={window.id === limiting || undefined}
          >
            <span className="nx-q-text">
              <span className="nx-q-name" title={quotaName(window)}>
                {quota.name}
              </span>
              <span className="nx-q-renew">{quota.renewal}</span>
              <span className="nx-q-pct" data-status={quota.status}>
                {quota.reading}
              </span>
            </span>
            <ShareBar pct={quota.shown} tone={quota.status ?? 'none'} label={quota.spoken} />
            {!row.view.stale && !quota.model.expired && <Amount window={window} />}
          </span>
        );
      })}
    </>
  );
}

function Tile({
  row,
  ctx,
  selected,
  onOpen,
  onKeyDown,
}: {
  row: TriageRow;
  ctx: ViewContext;
  selected: boolean;
  onOpen: (id: HarnessId) => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
}) {
  const { id } = row.view;
  const { pct, tone } = readingOf(row);
  const name = harnessName(id);
  const quotas = gridQuotas(row.view, ctx);
  const over = row.model?.over ?? null;
  return (
    <button
      type="button"
      id={nxTabId(id)}
      className="nx-tile"
      aria-pressed={selected}
      tabIndex={selected ? 0 : -1}
      data-tier={row.tier}
      title={row.detail}
      onKeyDown={onKeyDown}
      onClick={() => onOpen(id)}
    >
      <span className="nx-tile-head">
        <Logo id={id} size={16} />
        <span className="nx-name" id={nxNameId(id)}>
          {name}
        </span>
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
      {row.short !== '' && (
        <span className="nx-tile-note" data-tier={row.tier}>
          <i aria-hidden="true" />
          {row.short}
        </span>
      )}
      <QuotaRows row={row} ctx={ctx} />
      {quotas.extra > 0 && (
        <span className="nx-tile-more">
          +{quotas.extra} {quotas.extra === 1 ? 'cota' : 'cotas'} nos detalhes
        </span>
      )}
      <span className="vh">{row.verdict}</span>
    </button>
  );
}

export function Triage({ views, ctx, selected, onSelect, onOpen, onConfigure }: TriageProps) {
  const triage: TriageBoard = useMemo(() => triageOf(views, ctx), [views, ctx]);
  const headline = useMemo(() => headlineOf(triage), [triage]);
  const onKeyDown = useRovingTabs(
    triage.rows.map((row) => row.view.id),
    selected,
    onSelect,
  );
  const shown = showsUsed(ctx.settings);

  if (triage.rows.length === 0) {
    return (
      <section className="nx-note">
        <h3>Nenhuma fonte monitorada</h3>
        <p className="nx-meta">
          Nada foi lido ainda. Escolha o que acompanhar e o painel passa a abrir ordenado por
          urgência, com a previsão de cada cota no próprio cartão.
        </p>
        <div className="actions">
          <button type="button" className="btn primary" onClick={onConfigure}>
            Escolher fontes
          </button>
        </div>
      </section>
    );
  }

  return (
    <div className="nx-triage">
      <section className="nx-headline" data-tone={headline.tone}>
        <Icon name={headline.tone === 'attention' ? 'alert' : 'check'} />
        <div>
          <p className="nx-headline-text">
            {headline.text}
            <span className="nx-headline-sub">
              {triage.rows.length} {triage.rows.length === 1 ? 'provider' : 'providers'} ·{' '}
              {shown ? 'cota utilizada' : 'cota disponível'}
            </span>
            {headline.ids.length > 0 && (
              <span className="nx-headline-jump">
                {headline.ids.map((id) => (
                  <button key={id} type="button" className="link-btn" onClick={() => onOpen(id)}>
                    Ver {harnessName(id)}
                  </button>
                ))}
              </span>
            )}
          </p>
        </div>
      </section>
      <section className="nx-section">
        <GroupHead title="Por urgência" count={triage.rows.length} />
        <div className="nx-tiles">
          {triage.rows.map((row) => (
            <Tile
              key={row.view.id}
              row={row}
              ctx={ctx}
              selected={row.view.id === selected}
              onOpen={onOpen}
              onKeyDown={onKeyDown}
            />
          ))}
        </div>
      </section>
    </div>
  );
}
