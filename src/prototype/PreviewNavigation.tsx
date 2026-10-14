import { useEffect, useRef, useState } from 'react';
import { harnessName } from '../app/harness';
import { summaryOf } from '../app/tabModel';
import type { ViewContext } from '../app/windowModel';
import { HarnessPanel } from '../components/HarnessPanel';
import { Icon } from '../components/icons';
import { PANEL_ID, tabNameId } from '../components/shell/TabStrip';
import { formatPercent } from '../i18n/format';
import type { HarnessId, HarnessView } from '../lib/contract';
import { criticalViews } from './model';
import { ProviderDetails } from './ProviderDetails';
import { ProviderGrid } from './ProviderGrid';
import { ProviderOverview } from './ProviderOverview';

export function PreviewNavigation({
  views,
  active,
  selected,
  ctx,
  onSelect,
  loaded,
  onSettings,
}: {
  views: readonly HarnessView[];
  active: readonly HarnessView[];
  selected: HarnessId;
  ctx: ViewContext;
  onSelect: (id: HarnessId) => void;
  loaded: boolean;
  onSettings: () => void;
}) {
  const critical = criticalViews(active, ctx);
  return (
    <div className="preview-nav">
      <div className="preview-section-head">
        <div>
          <h2>
            Providers ativos <span>{loaded ? active.length : '…'}</span>
          </h2>
          <p>{ctx.settings.general.showUsed ? 'Cota utilizada' : 'Cota disponível'}</p>
        </div>
        <ProviderOverview views={views} ctx={ctx} />
      </div>
      {critical.length > 0 && (
        <section className="preview-critical" role="status" aria-label="Uso crítico">
          <Icon name="alert" />
          <div>
            <strong>Uso crítico</strong>
            <div className="preview-critical-actions">
              {critical.map((view) => (
                <button
                  type="button"
                  key={view.id}
                  className="link-btn"
                  onClick={() => onSelect(view.id)}
                >
                  {harnessName(view.id)} ·{' '}
                  {formatPercent(100 - (summaryOf(view, ctx)?.availablePct ?? 100))} usado
                </button>
              ))}
            </div>
          </div>
        </section>
      )}
      {!loaded ? (
        <output className="hint">Carregando providers…</output>
      ) : active.length > 0 ? (
        <div className="preview-grid">
          <ProviderGrid views={active} selected={selected} ctx={ctx} onSelect={onSelect} />
        </div>
      ) : (
        <section className="preview-empty">
          <Icon name="plug" />
          <h3>Conecte seu primeiro provider</h3>
          <p>Suas fontes ativas aparecerão aqui. Escolha quais ferramentas deseja acompanhar.</p>
          <button className="btn primary" type="button" onClick={onSettings}>
            Configurar providers
          </button>
        </section>
      )}
    </div>
  );
}

function DetailAccordion({
  view,
  ctx,
  epoch,
}: {
  view: HarnessView;
  ctx: ViewContext;
  epoch?: number;
}) {
  const [open, setOpen] = useState(true);
  const detailsRef = useRef<HTMLDetailsElement>(null);

  useEffect(() => {
    setOpen(true);
    if (epoch && epoch > 0) {
      detailsRef.current?.scrollIntoView?.({ block: 'nearest', behavior: 'smooth' });
    }
  }, [epoch]);

  return (
    <details
      ref={detailsRef}
      className="preview-details"
      open={open}
      onToggle={(event) => setOpen(event.currentTarget.open)}
    >
      <summary>
        Detalhes de {harnessName(view.id)}
        <span className="preview-detail-count">
          {view.windows.length} {view.windows.length === 1 ? 'cota' : 'cotas'}
        </span>
      </summary>
      {open && <ProviderDetails view={view} ctx={ctx} />}
    </details>
  );
}

export function PreviewPanel({
  view,
  ctx,
  epoch,
}: {
  view: HarnessView;
  ctx: ViewContext;
  epoch?: number;
}) {
  return (
    <section role="tabpanel" id={PANEL_ID} aria-labelledby={tabNameId(view.id)}>
      {view.state === 'ready' ? (
        <DetailAccordion key={view.id} view={view} ctx={ctx} epoch={epoch} />
      ) : (
        <HarnessPanel view={view} ctx={ctx} />
      )}
    </section>
  );
}
