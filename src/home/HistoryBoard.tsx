// Histórico: the stored passes of one window, drawn large enough to read, with the comparison
// against the earlier passes spelled out and the readings still available as a table. It is only
// mounted when the setting allows it, so a disabled preference never reaches the backend.
import { useMemo, useState } from 'react';
import { burndownOf } from '../app/burndown';
import { summarySentences } from '../app/burndownText';
import { sharePct, showsUsed } from '../app/quota';
import { useHistory } from '../app/useHistory';
import { modelWindow, type ViewContext, windowTitle } from '../app/windowModel';
import { Icon } from '../components/icons';
import { HistoryTable } from '../components/panel/history/HistoryTable';
import { t } from '../i18n';
import { formatPercent } from '../i18n/format';
import type { HarnessView } from '../lib/contract';
import { availablePct } from '../lib/usage';
import { historyCsv } from './historyCsv';
import { trendOf } from './historyModel';
import { readableWindows } from './quotas';
import { TrendChart } from './TrendChart';
import { windowLabel } from './wording';

const TREND_ICON = { more: 'alert', less: 'check', similar: 'clock', first: 'clock' } as const;

export function HistoryBoard({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const choices = useMemo(() => readableWindows(view), [view]);
  const [picked, setPicked] = useState<string | null>(null);
  const active = choices.find((window) => window.id === picked) ?? choices[0];
  const state = useHistory(view.id, active !== undefined, view.readAt);
  const [copyNote, setCopyNote] = useState('');
  const used = showsUsed(ctx.settings);

  if (active === undefined) {
    return (
      <section className="nx-board">
        <h3 className="nx-board-head">Histórico</h3>
        <p className="hint">Nenhuma cota desta fonte guarda uma leitura com hora de renovação.</p>
      </section>
    );
  }

  const model = modelWindow(view.id, active, ctx);
  const fresh = state.status === 'ready' && state.history.harness === view.id;
  const burndown =
    state.status === 'ready' && fresh ? burndownOf(active, state.history.epochs) : null;
  const trend = burndown === null ? null : trendOf(burndown);
  const title = windowTitle(view.id, active);

  const copyCsv = () => {
    if (state.status !== 'ready') return;
    const written = navigator.clipboard?.writeText(historyCsv(view.id, state.history.epochs));
    if (written === undefined) {
      setCopyNote(t('history.copy_failed'));
      return;
    }
    void written.then(
      () => setCopyNote(t('history.copied')),
      () => setCopyNote(t('history.copy_failed')),
    );
  };

  return (
    <section className="nx-board">
      <h3 className="nx-board-head">
        Histórico
        {burndown !== null && <span className="nx-count">{burndown.past.length + 1}</span>}
        {fresh && burndown !== null && (
          <button
            type="button"
            className="icon-btn nx-board-action"
            aria-label={t('history.copy_csv')}
            title={t('history.copy_csv')}
            onClick={copyCsv}
          >
            <Icon name="copy" />
          </button>
        )}
      </h3>
      <p className="vh" role="status">
        {copyNote}
      </p>
      {choices.length > 1 && (
        <fieldset className="nx-chips" aria-label={t('history.window')}>
          {choices.map((window) => (
            <button
              key={window.id}
              type="button"
              className="nx-chip-btn"
              aria-pressed={window.id === active.id}
              onClick={() => setPicked(window.id)}
            >
              {windowLabel(view.id, window)}
              {window.usedPct !== null && (
                <b className="nx-chip-btn-pct">
                  {formatPercent(sharePct(availablePct(window) ?? 0, used))}
                </b>
              )}
            </button>
          ))}
        </fieldset>
      )}
      {state.status === 'failed' && <p className="hint">Não foi possível ler o histórico.</p>}
      {!fresh && state.status !== 'failed' && <p className="hint">Carregando as leituras…</p>}
      {fresh && burndown === null && (
        <p className="hint">Ainda não há leituras guardadas para esta cota.</p>
      )}
      {fresh && burndown !== null && trend !== null && (
        <>
          <p className="nx-trend-line" data-kind={trend.kind}>
            <Icon name={TREND_ICON[trend.kind]} />
            <span>{trend.label}</span>
          </p>
          <TrendChart
            burndown={burndown}
            model={model}
            ctx={ctx}
            title={title}
            live={!view.stale}
          />
          <p className="nx-meta">{summarySentences(burndown, ctx.nowSecs).join(' ')}</p>
          <HistoryTable burndown={burndown} nowSecs={ctx.nowSecs} />
        </>
      )}
    </section>
  );
}
