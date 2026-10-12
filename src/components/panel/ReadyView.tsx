import { useMemo } from 'react';
import { layoutWindows, type WindowLayout } from '../../app/layout';
import type { ViewContext } from '../../app/windowModel';
import { t } from '../../i18n';
import type { HarnessView } from '../../lib/contract';
import { Note } from './Banner';
import { Facts } from './Facts';
import { FlatWindow } from './FlatWindow';
import { ClaudeLocked, HarnessFooter } from './HarnessExtras';
import { Hero } from './Hero';
import { HistorySection } from './history/HistorySection';
import { MetaChips } from './MetaChips';
import { Notices } from './Notices';
import { GroupList, PoolCard } from './PoolList';
import { StaleBanner } from './StaleBanner';

interface WindowsProps {
  layout: WindowLayout;
  ctx: ViewContext;
  stale: boolean;
}

/** Stale numbers stay readable but are desaturated as a block, and stop extrapolating a pace. */
export function WindowsBlock({ layout, ctx, stale }: WindowsProps) {
  const { hero, flats, pools, groupList } = layout;
  return (
    <div className="stack" data-stale={stale ? 'true' : undefined}>
      {hero && <Hero model={hero} ctx={ctx} stale={stale} />}
      {flats.map((model) => (
        <FlatWindow key={model.window.id} model={model} ctx={ctx} stale={stale} />
      ))}
      {pools.map((pool) => (
        <PoolCard key={pool.id} pool={pool} ctx={ctx} />
      ))}
      {groupList && <GroupList title={groupList.title} rows={groupList.rows} ctx={ctx} />}
    </div>
  );
}

export function ReadyView({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const layout = useMemo(() => layoutWindows(view, ctx), [view, ctx]);
  const hasNumbers = view.windows.length > 0;
  const claudeLocked = view.id === 'claude' && !ctx.settings.claude.oauthUsage;
  return (
    <>
      <MetaChips view={view} ctx={ctx} />
      <StaleBanner view={view} ctx={ctx} />
      <Notices notices={view.notices} harness={view.id} placement="top" />
      {hasNumbers ? (
        <WindowsBlock layout={layout} ctx={ctx} stale={view.stale} />
      ) : (
        <Note>{t('window.none')}</Note>
      )}
      {claudeLocked && <ClaudeLocked />}
      <Facts view={view} nowSecs={ctx.nowSecs} />
      <HistorySection key={view.id} view={view} ctx={ctx} />
      <Notices notices={view.notices} harness={view.id} placement="bottom" />
      <HarnessFooter harness={view.id} />
    </>
  );
}
