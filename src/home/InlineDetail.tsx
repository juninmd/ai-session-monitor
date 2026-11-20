// The provider detail of the list view: it opens under the provider's own row instead of over the
// frame, with the same controls and the same panel as the sheet over the cards.
import { harnessName } from '../app/model/harness';
import { nxTitleId } from '../app/model/homeIds';
import type { ViewContext } from '../app/model/windowModel';
import type { HarnessView } from '../lib/contract';
import { DetailActions } from './DetailActions';
import { Panel } from './Panel';

export function InlineDetail({
  view,
  ctx,
  onRefresh,
}: {
  view: HarnessView;
  ctx: ViewContext;
  onRefresh: () => void;
}) {
  return (
    <section className="nx-inline" aria-labelledby={nxTitleId(view.id)}>
      <header className="nx-inline-bar">
        <h2 className="nx-panel-title" id={nxTitleId(view.id)}>
          {harnessName(view.id)}
        </h2>
        <DetailActions view={view} ctx={ctx} onRefresh={onRefresh} />
      </header>
      <Panel view={view} ctx={ctx} />
    </section>
  );
}
