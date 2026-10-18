// The detail content of one provider: every window it reports as a ficha, the stored trend, and the
// extra information. The frame around it — the lateral drawer and its heading — lives in Drawer.tsx.
import type { ViewContext } from '../app/windowModel';
import { HarnessPanel } from '../components/HarnessPanel';
import { MetaChips } from '../components/panel/MetaChips';
import { StaleBanner } from '../components/panel/StaleBanner';
import type { HarnessView } from '../lib/contract';
import { HistoryBoard } from './HistoryBoard';
import { InfoBoard } from './InfoBoard';
import { ProviderActions } from './ProviderActions';
import { QuotaSheet } from './QuotaSheet';

export function Panel({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  if (view.state !== 'ready') return <HarnessPanel view={view} ctx={ctx} />;
  return (
    <div className="nx-panel">
      {(view.stale || view.error !== null) && <StaleBanner view={view} ctx={ctx} />}
      <MetaChips view={view} ctx={ctx} />
      <ProviderActions key={view.id} view={view} ctx={ctx} />
      <QuotaSheet view={view} ctx={ctx} />
      {ctx.settings.general.historyEnabled && (
        <HistoryBoard key={`history-${view.id}`} view={view} ctx={ctx} />
      )}
      <InfoBoard view={view} ctx={ctx} />
    </div>
  );
}
