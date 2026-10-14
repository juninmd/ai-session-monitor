import type { ViewContext } from '../app/windowModel';
import { Icon } from '../components/icons';
import type { HarnessView, UsageWindow } from '../lib/contract';
import { compactQuota } from './quotaModel';

export function QuotaSummary({
  view,
  window,
  ctx,
}: {
  view: HarnessView;
  window: UsageWindow;
  ctx: ViewContext;
}) {
  const quota = compactQuota(view, window, ctx);
  return (
    <span className="preview-quota" data-window={window.id}>
      <span className="preview-quota-text" aria-hidden="true" title={quota.name}>
        <span>{quota.name}</span>
        <strong data-status={quota.status}>
          {(quota.status === 'crit' || quota.status === 'out') && <Icon name="alert" />}
          {quota.reading}
        </strong>
      </span>
      <span
        className={`t-bar${quota.shown === null ? ' off' : ''}`}
        data-status={quota.status}
        aria-hidden="true"
      >
        {quota.shown !== null && <i style={{ inlineSize: `${quota.shown}%` }} />}
      </span>
      <span className="vh"> {quota.spoken}</span>
    </span>
  );
}
