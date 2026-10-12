import { useMemo } from 'react';
import { cooldownLeft, nextAttemptText } from '../../app/cooldown';
import { layoutWindows } from '../../app/layout';
import type { ViewContext } from '../../app/windowModel';
import { t } from '../../i18n';
import { formatAge } from '../../i18n/format';
import type { HarnessError, HarnessView } from '../../lib/contract';
import { Banner } from './Banner';
import { ErrorBlock } from './ErrorBlock';
import { WindowsBlock } from './ReadyView';

const UNKNOWN_FAILURE: HarnessError = { code: 'internal', retryable: false, params: {} };

/** A failed read: the reason and one action first, then whatever numbers were last known. */
export function ErrorView({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const layout = useMemo(() => layoutWindows(view, ctx), [view, ctx]);
  const age = view.readAt === null ? null : formatAge(ctx.nowSecs - view.readAt);
  const left = cooldownLeft(view, ctx.nowSecs);
  return (
    <>
      <ErrorBlock error={view.error ?? UNKNOWN_FAILURE} harness={view.id} />
      {left !== null && (
        <Banner tone="warn">
          <p>{nextAttemptText(left)}</p>
        </Banner>
      )}
      {view.windows.length > 0 && (
        <>
          <Banner tone="warn">
            <p>{age === null ? t('stale.banner_noage') : t('stale.banner', { age })}</p>
          </Banner>
          <WindowsBlock layout={layout} ctx={ctx} stale />
        </>
      )}
    </>
  );
}
