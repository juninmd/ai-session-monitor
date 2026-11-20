// The provider detail, opened as a sheet over the cards. The shell owns the layer; this file
// decides what the bar carries and puts the audit in the body.

import { harnessName } from '../app/model/harness';
import { NX_DRAWER_CLOSE_ID, nxTitleId } from '../app/model/homeIds';
import type { ViewContext } from '../app/model/windowModel';
import { t } from '../i18n';
import type { HarnessView } from '../lib/contract';
import { DetailActions } from './DetailActions';
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
  return (
    <DrawerShell
      titleId={nxTitleId(view.id)}
      title={harnessName(view.id)}
      closeId={NX_DRAWER_CLOSE_ID}
      closeLabel={t('drawer.close')}
      size="sheet"
      onClose={onClose}
      actions={<DetailActions view={view} ctx={ctx} onRefresh={onRefresh} />}
    >
      <Panel view={view} ctx={ctx} />
    </DrawerShell>
  );
}
