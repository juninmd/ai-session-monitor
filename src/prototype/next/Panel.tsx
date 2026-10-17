// The detail panel of the proposed interface: who the provider is, every window it reports as a
// ficha, the stored trend, and the way into its settings. Scanning lives in the grid above; this is
// the place to audit one provider, so it is quieter and denser than the tiles.
import { harnessName } from '../../app/harness';
import { useNavigation } from '../../app/navigation';
import type { ViewContext } from '../../app/windowModel';
import { HarnessPanel } from '../../components/HarnessPanel';
import { Icon } from '../../components/icons';
import { MetaChips } from '../../components/panel/MetaChips';
import { StaleBanner } from '../../components/panel/StaleBanner';
import type { HarnessView } from '../../lib/contract';
import { ProviderActions } from '../ProviderActions';
import { HistoryBoard } from './HistoryBoard';
import { InfoBoard } from './InfoBoard';
import { nxTitleId } from './ids';
import { QuotaSheet } from './QuotaSheet';

function Head({ view }: { view: HarnessView }) {
  const navigation = useNavigation();
  return (
    <header className="nx-detail-head">
      <h2 className="nx-panel-title" id={nxTitleId(view.id)}>
        {harnessName(view.id)}
      </h2>
      <button
        type="button"
        className="icon-btn"
        aria-label={`Configurar ${harnessName(view.id)}`}
        onClick={() => navigation.openSettings(view.id)}
      >
        <Icon name="sliders" />
      </button>
    </header>
  );
}

export function Panel({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  if (view.state !== 'ready') {
    return (
      <section className="nx-panel" aria-labelledby={nxTitleId(view.id)}>
        <Head view={view} />
        <HarnessPanel view={view} ctx={ctx} />
      </section>
    );
  }
  return (
    <section className="nx-panel" aria-labelledby={nxTitleId(view.id)}>
      <Head view={view} />
      {(view.stale || view.error !== null) && <StaleBanner view={view} ctx={ctx} />}
      <MetaChips view={view} ctx={ctx} />
      <ProviderActions key={view.id} view={view} ctx={ctx} />
      <QuotaSheet view={view} ctx={ctx} />
      {ctx.settings.general.historyEnabled && (
        <HistoryBoard key={`history-${view.id}`} view={view} ctx={ctx} />
      )}
      <InfoBoard view={view} ctx={ctx} />
    </section>
  );
}
