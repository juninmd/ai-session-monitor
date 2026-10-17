// Triage: the proposed home. One headline answers "do I need to act?", then the providers are
// listed by urgency instead of in a fixed order, with the projection — the most valuable signal —
// printed on the row that owns it instead of hidden inside its detail panel.
import { type KeyboardEvent, useMemo } from 'react';
import { harnessName } from '../../app/harness';
import { showsUsed } from '../../app/quota';
import type { ViewContext } from '../../app/windowModel';
import { Icon } from '../../components/icons';
import { formatPercent } from '../../i18n/format';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { GroupHead, Logo, ShareBar, type Tone } from './Bits';
import { nxNameId, nxTabId } from './ids';
import { headlineOf, type TriageBoard, type TriageRow, triageOf } from './triageModel';
import { useRovingTabs } from './useRovingTabs';
import { renewalText } from './wording';

export interface TriageProps {
  views: readonly HarnessView[];
  ctx: ViewContext;
  selected: HarnessId;
  onSelect: (id: HarnessId) => void;
  onConfigure: () => void;
}

function readingOf(row: TriageRow): { pct: number | null; tone: Tone } {
  if (row.model === null) return { pct: null, tone: 'none' };
  return { pct: row.model.shown, tone: row.model.status ?? 'none' };
}

function RowOption({
  row,
  ctx,
  selected,
  onSelect,
  onKeyDown,
  compact,
}: {
  row: TriageRow;
  ctx: ViewContext;
  selected: boolean;
  onSelect: (id: HarnessId) => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  compact: boolean;
}) {
  const { id } = row.view;
  const { pct, tone } = readingOf(row);
  const name = harnessName(id);
  const renewal = renewalText(row.model, ctx);
  // A comfortable row is one line about its quota; anything else leads with why it is not fine.
  const parts = compact
    ? row.tier === 'fine'
      ? [row.windowName, renewal]
      : [row.verdict, row.windowName]
    : [row.windowName, renewal];
  const meta = parts.filter((part) => part !== null && part !== '').join(' · ');
  return (
    <li>
      <button
        type="button"
        id={nxTabId(id)}
        className={compact ? 'nx-card nx-line' : 'nx-card'}
        aria-pressed={selected}
        tabIndex={selected ? 0 : -1}
        data-tier={row.tier}
        onKeyDown={onKeyDown}
        onClick={() => onSelect(id)}
      >
        <span className="nx-card-top">
          <Logo id={id} size={compact ? 16 : 18} />
          <span className="nx-name" id={nxNameId(id)}>
            {name}
          </span>
          {compact && <span className="nx-meta">{meta}</span>}
          <span className="nx-pct" data-status={tone}>
            {formatPercent(pct)}
          </span>
        </span>
        <ShareBar pct={pct} tone={tone} label={`${name}: ${formatPercent(pct)}`} />
        {!compact && (
          <>
            <span className="nx-verdict" data-tier={row.tier} title={row.detail}>
              {row.verdict}
            </span>
            <span className="nx-meta">
              {meta}
              <span className="vh"> {row.detail}</span>
            </span>
          </>
        )}
      </button>
    </li>
  );
}

export function Triage({ views, ctx, selected, onSelect, onConfigure }: TriageProps) {
  const triage: TriageBoard = useMemo(() => triageOf(views, ctx), [views, ctx]);
  const headline = useMemo(() => headlineOf(triage), [triage]);
  const onKeyDown = useRovingTabs(
    triage.rows.map((row) => row.view.id),
    selected,
    onSelect,
  );
  const shown = showsUsed(ctx.settings);
  const groups = [
    { key: 'attention', title: 'Precisa de atenção', rows: triage.attention, compact: false },
    { key: 'loading', title: 'Lendo', rows: triage.loading, compact: true },
    { key: 'fine', title: 'Em dia', rows: triage.fine, compact: true },
    { key: 'unknown', title: 'Sem leitura', rows: triage.unknown, compact: true },
  ].filter((group) => group.rows.length > 0);

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
          <p className="nx-headline-text">{headline.text}</p>
          <p className="nx-headline-sub">
            {triage.rows.length} {triage.rows.length === 1 ? 'provider' : 'providers'} ·{' '}
            {shown ? 'cota utilizada' : 'cota disponível'}
          </p>
          {headline.ids.length > 0 && (
            <p className="nx-headline-jump">
              {headline.ids.map((id) => (
                <button key={id} type="button" className="link-btn" onClick={() => onSelect(id)}>
                  Ver {harnessName(id)}
                </button>
              ))}
            </p>
          )}
        </div>
      </section>
      <div className="nx-rows">
        {groups.map((group) => (
          <section key={group.key} className="nx-section">
            <GroupHead title={group.title} count={group.rows.length} />
            <ul className="nx-list">
              {group.rows.map((row) => (
                <RowOption
                  key={row.view.id}
                  row={row}
                  ctx={ctx}
                  selected={row.view.id === selected}
                  onSelect={onSelect}
                  onKeyDown={onKeyDown}
                  compact={group.compact}
                />
              ))}
            </ul>
          </section>
        ))}
      </div>
    </div>
  );
}
