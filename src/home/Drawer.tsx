// The provider detail, opened laterally over the grid. The shell owns the layer; this file decides
// what the bar carries and puts the audit in the body.
import { harnessName } from '../app/harness';
import { NX_DRAWER_CLOSE_ID, nxTitleId } from '../app/homeIds';
import { useNavigation } from '../app/navigation';
import type { ViewContext } from '../app/windowModel';
import { Icon } from '../components/icons';
import type { HarnessView } from '../lib/contract';
import { DrawerShell } from './DrawerShell';
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
  const name = harnessName(view.id);
  return (
    <DrawerShell
      titleId={nxTitleId(view.id)}
      title={name}
      closeId={NX_DRAWER_CLOSE_ID}
      closeLabel="Fechar os detalhes"
      onClose={onClose}
      actions={
        <>
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
        </>
      }
    >
      <Panel view={view} ctx={ctx} />
    </DrawerShell>
  );
}
