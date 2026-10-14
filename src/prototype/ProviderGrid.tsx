import { type KeyboardEvent, useEffect, useState } from 'react';
import { harnessName } from '../app/harness';
import { summaryOf, tabChip } from '../app/tabModel';
import type { ViewContext } from '../app/windowModel';
import { HarnessLogo } from '../components/brand/HarnessLogo';
import { focusTab, PANEL_ID, tabId, tabNameId } from '../components/shell/TabStrip';
import { t } from '../i18n';
import type { HarnessId, HarnessView } from '../lib/contract';
import { QuotaSummary } from './QuotaSummary';
import { gridQuotas, unofficialSource } from './quotaModel';
import { SourceInfo } from './SourceInfo';

interface GridProps {
  views: readonly HarnessView[];
  selected: HarnessId;
  ctx: ViewContext;
  onSelect: (id: HarnessId) => void;
}

function ProviderCard({
  view,
  selected,
  ctx,
  onSelect,
}: Omit<GridProps, 'views'> & { view: HarnessView }) {
  const [sourceFocus, setSourceFocus] = useState(false);
  const chip = tabChip(view, summaryOf(view, ctx), ctx);
  const quotas = gridQuotas(view, ctx);
  const hasQuotas = view.state === 'ready' && quotas.windows.length > 0;
  return (
    <button
      type="button"
      role="tab"
      id={tabId(view.id)}
      className="tab"
      aria-selected={view.id === selected}
      aria-controls={PANEL_ID}
      tabIndex={view.id === selected ? 0 : -1}
      onClick={() => onSelect(view.id)}
      onFocus={(event) => setSourceFocus(event.currentTarget.matches(':focus-visible'))}
      onBlur={() => setSourceFocus(false)}
    >
      <span className="preview-provider-name">
        <HarnessLogo id={view.id} size={16} />
        <span id={tabNameId(view.id)}>{harnessName(view.id)}</span>
        {unofficialSource(view, ctx) && (
          <SourceInfo keyboardFocus={sourceFocus} onDismiss={() => setSourceFocus(false)} />
        )}
      </span>
      {hasQuotas ? (
        <span className="preview-quotas">
          {quotas.group !== null && (
            <span className="preview-group-name" title={quotas.group}>
              {quotas.group}
            </span>
          )}
          {quotas.windows.map((window) => (
            <QuotaSummary key={window.id} view={view} window={window} ctx={ctx} />
          ))}
        </span>
      ) : (
        <span className="preview-provider-state">
          {chip.kind === 'text' ? chip.text : 'Carregando…'}
        </span>
      )}
      {hasQuotas && quotas.extra > 0 && (
        <span className="preview-extra-count">
          +{quotas.extra} {quotas.extra === 1 ? 'cota nos detalhes' : 'cotas nos detalhes'}
        </span>
      )}
      <span className="vh"> {chip.sr}</span>
    </button>
  );
}

export function ProviderGrid({ views, selected, ctx, onSelect }: GridProps) {
  useEffect(() => {
    document
      .getElementById(tabId(selected))
      ?.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
  }, [selected]);
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const ids = views.map((view) => view.id);
    const index = ids.indexOf(selected);
    const columns = getComputedStyle(event.currentTarget).gridTemplateColumns.split(' ').length;
    let target: number;
    switch (event.key) {
      case 'ArrowRight':
        target = (index + 1) % ids.length;
        break;
      case 'ArrowLeft':
        target = (index - 1 + ids.length) % ids.length;
        break;
      case 'ArrowDown':
        target = (index + columns) % ids.length;
        break;
      case 'ArrowUp':
        target = (index - columns + ids.length) % ids.length;
        break;
      case 'Home':
        target = 0;
        break;
      case 'End':
        target = ids.length - 1;
        break;
      default:
        return;
    }
    const id = ids[target];
    if (id === undefined) return;
    event.preventDefault();
    onSelect(id);
    focusTab(id);
  };
  return (
    <div className="tabs" role="tablist" aria-label={t('tabs.label')} onKeyDown={onKeyDown}>
      {views.map((view) => (
        <ProviderCard key={view.id} view={view} selected={selected} ctx={ctx} onSelect={onSelect} />
      ))}
    </div>
  );
}
