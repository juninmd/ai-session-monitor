// Triage: the home. One headline answers "do I need to act?"; below it every provider sits in a
// grid of two per row or in a list, as the person chose, ordered by urgency rather than by registration. The card itself is in
// TriageTile.tsx; the alarm stays in the headline and the card only states the facts.
import { Fragment, useMemo, useState } from 'react';
import type { ViewMode } from '../app/hooks/useViewMode';
import { harnessName } from '../app/model/harness';
import { showsUsed } from '../app/model/quota';
import type { ViewContext } from '../app/model/windowModel';
import { Icon } from '../components/icons';
import { t, tn } from '../i18n';
import type { HarnessId, HarnessView } from '../lib/contract';
import { GroupHead } from './Bits';
import { HealthStrip } from './HealthStrip';
import { InlineDetail } from './InlineDetail';
import { Tile } from './TriageTile';
import {
  effectiveFilter,
  filterRows,
  healthOf,
  showsHealth,
  type TriageFilter,
} from './triageFilter';
import { headlineOf, type TriageBoard, triageOf } from './triageModel';
import { useRovingTabs } from './useRovingTabs';
import { ViewToggle } from './ViewToggle';

/**
 * The headline names the count; the grid below already lists every provider in the same order.
 * More links than this turn the headline into a wall of names.
 */
const MAX_JUMPS = 3;

export interface TriageProps {
  views: readonly HarnessView[];
  ctx: ViewContext;
  selected: HarnessId;
  /** The selected provider's detail is open: inline in the list, as a sheet over the cards. */
  detailOpen: boolean;
  viewMode: ViewMode;
  onViewMode: (mode: ViewMode) => void;
  /** Move the highlight, as an arrow key does. */
  onSelect: (id: HarnessId) => void;
  /** Activate one provider and open its lateral detail. */
  onOpen: (id: HarnessId) => void;
  onRefreshView: (id: HarnessId) => void;
  onConfigure: () => void;
}

export function Triage({
  views,
  ctx,
  selected,
  detailOpen,
  viewMode,
  onViewMode,
  onSelect,
  onOpen,
  onRefreshView,
  onConfigure,
}: TriageProps) {
  const triage: TriageBoard = useMemo(() => triageOf(views, ctx), [views, ctx]);
  const headline = useMemo(() => headlineOf(triage), [triage]);
  const health = useMemo(() => healthOf(triage.rows), [triage]);
  const [wanted, setWanted] = useState<TriageFilter>('all');
  const filter = effectiveFilter(wanted, health);
  const rows = useMemo(() => filterRows(triage.rows, filter), [triage, filter]);
  const onKeyDown = useRovingTabs(
    rows.map((row) => row.view.id),
    selected,
    onSelect,
  );
  // The highlighted provider can be filtered out; the grid still needs one tile in the tab order.
  const tabStop = rows.some((row) => row.view.id === selected) ? selected : rows[0]?.view.id;
  const shown = showsUsed(ctx.settings);

  if (triage.rows.length === 0) {
    return (
      <section className="nx-note">
        <h3>{t('triage.empty.title')}</h3>
        <p className="nx-meta">{t('triage.empty.body')}</p>
        <div className="actions">
          <button type="button" className="btn primary" onClick={onConfigure}>
            {t('triage.empty.action')}
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
              {tn(shown ? 'triage.sub_used' : 'triage.sub_left', triage.rows.length)}
            </span>
            {headline.ids.length > 0 && (
              <span className="nx-headline-jump">
                {headline.ids.slice(0, MAX_JUMPS).map((id) => (
                  <button key={id} type="button" className="link-btn" onClick={() => onOpen(id)}>
                    {t('triage.see', { name: harnessName(id) })}
                  </button>
                ))}
              </span>
            )}
          </p>
        </div>
      </section>
      <section className="nx-section">
        <div className="nx-section-bar">
          <GroupHead title={t('triage.by_urgency')} count={rows.length} />
          <ViewToggle mode={viewMode} onChange={onViewMode} />
        </div>
        {showsHealth(health) && (
          <HealthStrip
            health={health}
            total={triage.rows.length}
            filter={filter}
            onFilter={setWanted}
          />
        )}
        <div className="nx-tiles" key={filter} data-view={viewMode}>
          {rows.map((row, index) => (
            <Fragment key={row.view.id}>
              <Tile
                row={row}
                ctx={ctx}
                index={index}
                selected={row.view.id === selected}
                tabStop={row.view.id === tabStop}
                onOpen={onOpen}
                onKeyDown={onKeyDown}
              />
              {viewMode === 'list' && detailOpen && row.view.id === selected && (
                <InlineDetail
                  view={row.view}
                  ctx={ctx}
                  onRefresh={() => onRefreshView(row.view.id)}
                />
              )}
            </Fragment>
          ))}
        </div>
      </section>
    </div>
  );
}
