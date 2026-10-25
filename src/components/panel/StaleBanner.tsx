import { cooldownLeft, nextAttemptText } from '../../app/cooldown';
import { describeError } from '../../app/errorText';
import type { ViewContext } from '../../app/windowModel';
import { t } from '../../i18n';
import { formatAge } from '../../i18n/format';
import type { HarnessView } from '../../lib/contract';
import { Banner } from './Banner';
import { ActionError, useErrorAction } from './ErrorActions';

function FailedRefresh({ view }: { view: HarnessView }) {
  const text = view.error ? describeError(view.error, view.id) : null;
  const task = useErrorAction(text?.action ?? 'retry', view.id);
  if (text === null) return null;
  return (
    <>
      <p>{t('stale.failed', { title: text.title })}</p>
      <div className="actions">
        <button
          type="button"
          className="btn"
          disabled={task.pending}
          onClick={() => void task.run()}
        >
          {task.label}
        </button>
      </div>
      {task.error && <ActionError error={task.error} harness={view.id} />}
    </>
  );
}

/** Old numbers stay on screen but say how old they are and why they stopped moving. */
export function StaleBanner({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  if (!view.stale && view.error === null) return null;
  const age = view.readAt === null ? null : formatAge(ctx.nowSecs - view.readAt);
  const left = cooldownLeft(view, ctx.nowSecs);
  return (
    <Banner tone="warn" role="status">
      {view.stale && <p>{age === null ? t('stale.banner_noage') : t('stale.banner', { age })}</p>}
      <FailedRefresh view={view} />
      {left !== null && <p>{nextAttemptText(left)}</p>}
    </Banner>
  );
}
