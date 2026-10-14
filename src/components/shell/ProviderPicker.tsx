import { useEffect, useRef, useState } from 'react';
import { harnessName } from '../../app/harness';
import { summaryOf, tabChip } from '../../app/tabModel';
import type { ViewContext } from '../../app/windowModel';
import { t } from '../../i18n';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { HarnessLogo } from '../brand/HarnessLogo';
import { focusTab } from './TabStrip';

/** A searchable overview keeps every source reachable without sideways scrolling. */
export function ProviderPicker({
  views,
  selected,
  ctx,
  onSelect,
}: {
  views: readonly HarnessView[];
  selected: HarnessId;
  ctx: ViewContext;
  onSelect: (id: HarnessId) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const searchRef = useRef<HTMLInputElement>(null);
  const summaryRef = useRef<HTMLElement>(null);

  useEffect(() => {
    if (open) searchRef.current?.focus();
  }, [open]);

  const normalized = query.trim().toLocaleLowerCase('pt-BR');
  const matches = views.filter((view) =>
    harnessName(view.id).toLocaleLowerCase('pt-BR').includes(normalized),
  );
  return (
    <details
      className="provider-picker"
      open={open}
      onKeyDown={(event) => {
        if (event.key !== 'Escape' || !open || event.nativeEvent.isComposing) return;
        event.preventDefault();
        setOpen(false);
        setQuery('');
        summaryRef.current?.focus();
      }}
      onToggle={(event) => {
        setOpen(event.currentTarget.open);
        if (!event.currentTarget.open) setQuery('');
      }}
    >
      <summary ref={summaryRef}>
        {t('providers.title')} <span className="provider-count">{views.length}</span>
      </summary>
      {open && (
        <div className="provider-body">
          <label className="vh" htmlFor="provider-search">
            {t('providers.search')}
          </label>
          <input
            ref={searchRef}
            id="provider-search"
            type="search"
            value={query}
            placeholder={t('providers.search')}
            onChange={(event) => setQuery(event.target.value)}
          />
          <fieldset className="provider-grid" aria-label={t('providers.results')}>
            {matches.map((view) => {
              const chip = tabChip(view, summaryOf(view, ctx), ctx);
              return (
                <button
                  type="button"
                  className="provider-choice"
                  key={view.id}
                  aria-pressed={selected === view.id}
                  data-status={chip.kind === 'pct' ? chip.status : undefined}
                  onClick={() => {
                    onSelect(view.id);
                    setOpen(false);
                    setQuery('');
                    focusTab(view.id);
                  }}
                >
                  <HarnessLogo id={view.id} size={20} />
                  <span>
                    <strong>{harnessName(view.id)}</strong>
                    <small>{chip.sr}</small>
                  </span>
                </button>
              );
            })}
          </fieldset>
          {matches.length === 0 && <p role="status">{t('providers.empty')}</p>}
          {query && (
            <button
              className="link-btn"
              type="button"
              onClick={() => {
                setQuery('');
                searchRef.current?.focus();
              }}
            >
              {t('providers.clear')}
            </button>
          )}
        </div>
      )}
    </details>
  );
}
