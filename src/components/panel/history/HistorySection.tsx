import { useId, useState } from 'react';
import { burndownOf } from '../../../app/burndown';
import { summarySentences } from '../../../app/burndownText';
import { useHistory } from '../../../app/useHistory';
import { type ViewContext, windowTitle } from '../../../app/windowModel';
import { t } from '../../../i18n';
import type { HarnessView, UsageWindow } from '../../../lib/contract';
import { Note } from '../Banner';
import { BurndownChart } from './BurndownChart';
import { HistoryTable } from './HistoryTable';

const hasReading = (window: UsageWindow) => window.usedPct !== null && window.resetsAt !== null;

/** Collapsed by default, so the stored readings are only fetched once somebody asks to see them. */
export function HistorySection({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const base = useId();
  const [open, setOpen] = useState(false);
  const [picked, setPicked] = useState<string | null>(null);
  const state = useHistory(view.id, open, view.readAt);
  const choices = view.windows.filter(hasReading);
  const window = choices.find((candidate) => candidate.id === picked) ?? choices[0];
  if (!ctx.settings.general.historyEnabled || window === undefined) return null;

  const bodyId = `${base}-body`;
  const title = windowTitle(view.id, window);
  const fresh = state.status === 'ready' && state.history.harness === view.id;
  const burndown =
    state.status === 'ready' && fresh ? burndownOf(window, state.history.epochs) : null;
  return (
    <section className="flat hist" aria-labelledby={`${base}-h`}>
      <h3 className="win-title" id={`${base}-h`}>
        <button
          type="button"
          className="hist-toggle"
          aria-expanded={open}
          aria-controls={bodyId}
          onClick={() => setOpen((current) => !current)}
        >
          {t('history.title')}
        </button>
      </h3>
      <div id={bodyId} hidden={!open} className="stack">
        {choices.length > 1 && (
          <label className="hist-pick">
            <span>{t('history.window')}</span>
            <select value={window.id} onChange={(event) => setPicked(event.target.value)}>
              {choices.map((choice) => (
                <option key={choice.id} value={choice.id}>
                  {windowTitle(view.id, choice)}
                </option>
              ))}
            </select>
          </label>
        )}
        {open && state.status === 'failed' && <Note icon="warn">{t('history.failed')}</Note>}
        {open && !fresh && state.status !== 'failed' && (
          <p className="note" role="status">
            {t('history.loading')}
          </p>
        )}
        {open && fresh && burndown === null && <Note>{t('history.empty')}</Note>}
        {open && burndown !== null && (
          <>
            <BurndownChart burndown={burndown} title={title} />
            <p className="hist-sum">{summarySentences(burndown, ctx.nowSecs).join(' ')}</p>
            <HistoryTable burndown={burndown} nowSecs={ctx.nowSecs} />
          </>
        )}
      </div>
    </section>
  );
}
