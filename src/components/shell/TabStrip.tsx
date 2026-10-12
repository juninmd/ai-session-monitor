// Five tabs share a 432 px strip: icon and a short reading on top, the name below, then the bar.
// Roving tabindex with automatic activation; arrows wrap, Home and End jump to the ends.
import type { KeyboardEvent } from 'react';
import { harnessName, TAB_ORDER } from '../../app/harness';
import { STATUS_GLYPH, summaryOf, type TabChip, tabChip } from '../../app/tabModel';
import type { ViewContext } from '../../app/windowModel';
import { t } from '../../i18n';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { HarnessLogo } from '../brand/HarnessLogo';
import { Icon } from '../icons';

export const tabId = (id: HarnessId) => `tab-${id}`;
/** The tab's visible name; the panel is labelled by it so its name stays short. */
export const tabNameId = (id: HarnessId) => `tab-name-${id}`;
export const PANEL_ID = 'panel';

export function focusTab(id: HarnessId): void {
  document.getElementById(tabId(id))?.focus();
}

function Reading({ chip }: { chip: TabChip }) {
  switch (chip.kind) {
    case 'pct':
      return (
        <span className="t-read" data-status={chip.status} aria-hidden="true">
          {chip.status !== 'ok' && <Icon name={STATUS_GLYPH[chip.status]} />}
          {chip.text}
        </span>
      );
    case 'loading':
      return <span className="t-read sk sk-pct" aria-hidden="true" />;
    case 'text':
      return null;
  }
}

function Foot({ chip }: { chip: TabChip }) {
  switch (chip.kind) {
    case 'pct':
      return (
        <span className="t-bar" data-status={chip.status} aria-hidden="true">
          <i style={{ inlineSize: `${Math.min(100, Math.max(0, chip.pct))}%` }} />
        </span>
      );
    case 'loading':
      return <span className="t-bar off" aria-hidden="true" />;
    case 'text':
      return (
        <span className="t-state" data-warn={chip.warn || undefined} aria-hidden="true">
          {chip.glyph && <Icon name={chip.glyph} />}
          {chip.text}
        </span>
      );
  }
}

interface TabProps {
  view: HarnessView;
  selected: boolean;
  ctx: ViewContext;
  onSelect: (id: HarnessId) => void;
}

function Tab({ view, selected, ctx, onSelect }: TabProps) {
  const chip = tabChip(view, summaryOf(view, ctx), ctx);
  const name = harnessName(view.id);
  return (
    <button
      type="button"
      role="tab"
      id={tabId(view.id)}
      className="tab"
      aria-selected={selected}
      aria-controls={PANEL_ID}
      tabIndex={selected ? 0 : -1}
      onClick={() => onSelect(view.id)}
    >
      <HarnessLogo id={view.id} size={20} className="t-icon" />
      <Reading chip={chip} />
      <span className="t-name" id={tabNameId(view.id)}>
        {name}
      </span>
      <Foot chip={chip} />
      {/* The space keeps "Claude Code" and the sentence apart in the accessible name. */}{' '}
      <span className="vh">{chip.sr}</span>
    </button>
  );
}

interface TabStripProps {
  views: readonly HarnessView[];
  selected: HarnessId;
  ctx: ViewContext;
  onSelect: (id: HarnessId) => void;
}

export function TabStrip({ views, selected, ctx, onSelect }: TabStripProps) {
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const last = TAB_ORDER.length - 1;
    const index = TAB_ORDER.indexOf(selected);
    let target: number;
    switch (event.key) {
      case 'ArrowRight':
        target = index === last ? 0 : index + 1;
        break;
      case 'ArrowLeft':
        target = index === 0 ? last : index - 1;
        break;
      case 'Home':
        target = 0;
        break;
      case 'End':
        target = last;
        break;
      default:
        return;
    }
    event.preventDefault();
    const id = TAB_ORDER[target];
    if (id === undefined) return;
    onSelect(id);
    focusTab(id);
  };

  return (
    <div className="tabs" role="tablist" aria-label={t('tabs.label')} onKeyDown={onKeyDown}>
      {views.map((view) => (
        <Tab
          key={view.id}
          view={view}
          selected={view.id === selected}
          ctx={ctx}
          onSelect={onSelect}
        />
      ))}
    </div>
  );
}
