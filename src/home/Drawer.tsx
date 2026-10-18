// The detail, opened laterally instead of stacked under the grid. A 27 rem flyout has no room for
// two columns, so this is a drawer against the trailing edge with the grid dimmed behind it: the
// list keeps its full height and the audit gets the full width.
import { useEffect, useRef } from 'react';
import { harnessName } from '../app/harness';
import { NX_DRAWER_CLOSE_ID, nxTitleId } from '../app/homeIds';
import { useNavigation } from '../app/navigation';
import type { ViewContext } from '../app/windowModel';
import { Icon } from '../components/icons';
import type { HarnessView } from '../lib/contract';
import { Panel } from './Panel';

export function DetailDrawer({
  view,
  ctx,
  onClose,
  onRefresh,
}: {
  view: HarnessView;
  ctx: ViewContext;
  onClose: () => void;
  onRefresh: () => void;
}) {
  const navigation = useNavigation();
  const closeRef = useRef<HTMLButtonElement>(null);
  const name = harnessName(view.id);
  // Focus moves into the drawer on open; the stage puts it back on the card that opened it.
  useEffect(() => {
    closeRef.current?.focus();
  }, []);
  return (
    <div className="nx-drawer-host">
      <button
        type="button"
        className="nx-drawer-scrim"
        aria-label="Fechar os detalhes"
        onClick={onClose}
      />
      <aside className="nx-drawer" role="dialog" aria-labelledby={nxTitleId(view.id)}>
        <header className="nx-drawer-bar">
          <button
            ref={closeRef}
            id={NX_DRAWER_CLOSE_ID}
            type="button"
            className="icon-btn"
            aria-label="Fechar os detalhes"
            onClick={onClose}
          >
            <Icon name="back" />
          </button>
          <h2 className="nx-panel-title" id={nxTitleId(view.id)}>
            {name}
          </h2>
          <button
            type="button"
            className="icon-btn"
            aria-label={`Atualizar ${name}`}
            onClick={onRefresh}
          >
            <Icon name="refresh" />
          </button>
          <button
            type="button"
            className="icon-btn"
            aria-label={`Configurar ${name}`}
            onClick={() => navigation.openSettings(view.id)}
          >
            <Icon name="sliders" />
          </button>
        </header>
        <div className="nx-drawer-body">
          <Panel view={view} ctx={ctx} />
        </div>
      </aside>
    </div>
  );
}
