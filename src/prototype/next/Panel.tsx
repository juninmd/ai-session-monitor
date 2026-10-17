// The detail panel of the proposed interface. Scanning lives in the lists above; this is the place
// to audit one provider, so it reuses the same detail surface the current interface already shows.
import { harnessName } from '../../app/harness';
import type { ViewContext } from '../../app/windowModel';
import { HarnessPanel } from '../../components/HarnessPanel';
import type { HarnessView } from '../../lib/contract';
import { ProviderDetails } from '../ProviderDetails';
import { nxNameId } from './ids';

export function Panel({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  return (
    <section className="nx-panel" aria-labelledby={nxNameId(view.id)}>
      <h2 className="nx-panel-title">{harnessName(view.id)}</h2>
      {view.state === 'ready' ? (
        <ProviderDetails view={view} ctx={ctx} />
      ) : (
        <HarnessPanel view={view} ctx={ctx} />
      )}
    </section>
  );
}
