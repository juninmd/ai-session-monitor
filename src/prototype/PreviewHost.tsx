import { type ReactNode, useMemo, useState } from 'react';
import { harnessName } from '../app/harness';
import { viewsInTabOrder } from '../app/placeholder';
import { useAsyncAction } from '../app/useAsyncAction';
import { useNowSecs } from '../app/useNowSecs';
import { useSettingsState } from '../app/useSettingsState';
import { useSnapshotState } from '../app/useSnapshotState';
import { Icon } from '../components/icons';
import { ActionError } from '../components/panel/ErrorActions';
import { formatPercent } from '../i18n/format';
import { isExpired } from '../lib/usage';
import type { MockBackend } from '../mock';
import { PreviewActionsContext, simulateCodexReset } from './actionModel';
import { ProviderOverview } from './ProviderOverview';
import { useCriticalToast } from './useCriticalToast';

/** Preview controls and the simulated Windows tray never invoke real desktop notifications. */
export function PreviewHost({ backend, children }: { backend: MockBackend; children: ReactNode }) {
  const [lowPriorityOffered, setLowPriorityOffered] = useState(false);
  const snapshots = useSnapshotState(backend);
  const settings = useSettingsState(backend);
  const { nowSecs } = useNowSecs();
  const views = useMemo(() => viewsInTabOrder(snapshots.snapshot), [snapshots.snapshot]);
  const ctx = useMemo(
    () => ({ nowSecs, settings: settings.settings }),
    [nowSecs, settings.settings],
  );
  const notices = useCriticalToast(views, ctx);
  const claude = views.find((view) => view.id === 'claude');
  const offerAvailable =
    settings.settings.claude.enabled && claude?.state === 'ready' && !claude.stale;
  const update = useAsyncAction(async () => {
    await backend.checkUpdate();
  });
  const codex = views.find((view) => view.id === 'codex');
  const window = codex?.windows[0];
  const canSimulate =
    settings.settings.codex.enabled &&
    codex?.state === 'ready' &&
    !codex.stale &&
    window?.usedPct !== null &&
    window?.usedPct !== undefined &&
    window.usedPct < 90 &&
    !isExpired(window, nowSecs);
  const simulate = useAsyncAction(async () => {
    if (!canSimulate || !codex) return;
    backend.override({
      ...codex,
      readAt: nowSecs,
      windows: codex.windows.map((item, index) => (index === 0 ? { ...item, usedPct: 90 } : item)),
    });
  });
  return (
    <main className="preview-stage">
      <div className="preview-caption">
        <span className="preview-tag">Protótipo visual</span>
        <small>Prévia local · dados fictícios</small>
      </div>
      <PreviewActionsContext
        value={{ lowPriorityOffered, consumeReset: () => simulateCodexReset(backend, nowSecs) }}
      >
        {children}
      </PreviewActionsContext>
      <details className="preview-demo-controls">
        <summary>Controles da prévia</summary>
        <nav className="preview-scenarios" aria-label="Cenários de demonstração">
          <a href="?prototype=grid&mock=demo">Dia a dia</a>
          <a href="?prototype=grid&mock=critical">Uso crítico</a>
          <a href="?prototype=grid&mock=fresh">Sem conexões</a>
          <a href="?prototype=grid&mock=errors">Falhas</a>
          <a href="?prototype=grid&mock=stale">Desatualizado</a>
          <a href="?prototype=grid&mock=loading">Carregando</a>
        </nav>
        <button
          type="button"
          className="btn"
          disabled={!canSimulate || simulate.pending}
          onClick={() => void simulate.run()}
        >
          <Icon name="alert" />
          Simular 90% no Codex
        </button>
        <p>O aviso aparece uma vez ao cruzar 90% nesta janela.</p>
        {simulate.error && <ActionError error={simulate.error} />}
        <div className="preview-demo-actions">
          <button
            className="btn"
            type="button"
            disabled={!offerAvailable || lowPriorityOffered}
            onClick={() => setLowPriorityOffered(true)}
          >
            Simular oferta low-priority no Claude
          </button>
          <button
            className="btn"
            type="button"
            disabled={update.pending}
            onClick={() => void update.run()}
          >
            Simular nova versão do monitor
          </button>
        </div>
        {update.error && <ActionError error={update.error} />}
      </details>
      <aside className="preview-tray" aria-label="Prévia da bandeja">
        <small>Hover na bandeja</small>
        <ProviderOverview views={views} ctx={ctx} tray />
      </aside>
      <div className="preview-toasts">
        {notices.toasts.map((notice) => (
          <section key={notice.key} className="preview-toast" role="alert">
            <span className="preview-tag">Prévia de notificação</span>
            <strong>
              <Icon name="alert" />
              {harnessName(notice.id)}: uso crítico
            </strong>
            <p>
              {formatPercent(notice.used)} da cota utilizada. Consulte o provider para conferir a
              próxima renovação.
            </p>
            <div>
              <button
                type="button"
                className="link-btn"
                onClick={() => notices.dismiss(notice.key)}
              >
                Dispensar
              </button>
            </div>
          </section>
        ))}
      </div>
    </main>
  );
}
