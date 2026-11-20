import { type CSSProperties, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { useAsyncAction } from '../app/hooks/useAsyncAction';
import type { ViewContext } from '../app/model/windowModel';
import { t, tn } from '../i18n';
import type { HarnessView } from '../lib/contract';
import { PreviewActionsContext, type ResetOutcome, resetCreditCount } from './actionModel';

const outcomeText = (outcome: ResetOutcome): string => t(`provider_actions.outcome.${outcome}`);

// The command stays a <code> element inside a sentence whose word order is the language's.
const COMMAND_MARK = '\u0001';

function LowPriorityHint() {
  const [before = '', after = ''] = t('provider_actions.low_priority_hint', {
    command: COMMAND_MARK,
  }).split(COMMAND_MARK);
  return (
    <p>
      {before}
      <code>/low-priority</code>
      {after}
    </p>
  );
}

const CONFETTI = 26;
const COLORS = ['var(--accent)', 'var(--warn-fill)', 'var(--crit)', 'var(--btn-bg)'];

/** A burst after a reset goes through: pure decoration, so it stays out of the accessibility tree. */
function Confetti() {
  const bits = useMemo(
    () =>
      Array.from({ length: CONFETTI }, (_, index) => {
        const angle = Math.random() * Math.PI * 2;
        const reach = 60 + Math.random() * 110;
        return {
          background: COLORS[index % COLORS.length],
          '--dx': `${Math.round(Math.cos(angle) * reach)}px`,
          '--dy': `${Math.round(Math.sin(angle) * reach - 40)}px`,
          '--r': `${Math.round(Math.random() * 720 - 360)}deg`,
        } as CSSProperties;
      }),
    [],
  );
  return (
    <span className="nx-confetti" aria-hidden="true">
      {bits.map((style, index) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: the pieces are anonymous and never reorder.
        <i key={index} className="nx-confetti-bit" style={style} />
      ))}
    </span>
  );
}

/** Compact, explicit actions; this component is only mounted inside the fabricated preview. */
export function ProviderActions({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const actions = useContext(PreviewActionsContext);
  const [confirming, setConfirming] = useState(false);
  const [result, setResult] = useState<ResetOutcome | null>(null);
  const [copied, setCopied] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const feedback = useRef<HTMLParagraphElement>(null);
  const wasConfirming = useRef(false);
  useEffect(() => {
    if (result !== null) feedback.current?.focus();
    else if (wasConfirming.current && !confirming) trigger.current?.focus();
    wasConfirming.current = confirming;
  }, [result, confirming]);
  const reset = useAsyncAction(async () => {
    if (!actions) return;
    setResult(await actions.consumeReset());
    setConfirming(false);
  });
  const copy = useAsyncAction(async () => {
    await navigator.clipboard.writeText('/low-priority');
    setCopied(true);
  });
  const count = resetCreditCount(view, ctx);
  const canReset = actions && count !== null && count > 0;
  const canCopy =
    actions?.lowPriorityOffered &&
    view.id === 'claude' &&
    view.state === 'ready' &&
    !view.stale &&
    ctx.settings.claude.enabled;
  if (!canReset && !canCopy && result === null) return null;
  return (
    <div className="preview-provider-actions">
      {result === 'reset' && <Confetti />}
      {canReset && (
        <div className="preview-action-row">
          <span>{tn('provider_actions.resets', count)}</span>
          <button
            ref={trigger}
            className="link-btn"
            type="button"
            disabled={reset.pending || confirming}
            onClick={() => {
              reset.clear();
              setResult(null);
              setConfirming(true);
            }}
          >
            {t('provider_actions.use_reset')}
          </button>
        </div>
      )}
      {confirming && canReset && (
        <fieldset className="preview-reset-confirm" aria-label={t('provider_actions.confirm_aria')}>
          <p>{t('provider_actions.confirm_body')}</p>
          <div className="preview-action-row">
            <button
              className="btn"
              type="button"
              disabled={reset.pending}
              onClick={() => void reset.run()}
            >
              {t('provider_actions.confirm')}
            </button>
            <button
              className="link-btn"
              type="button"
              disabled={reset.pending}
              onClick={() => setConfirming(false)}
            >
              {t('provider_actions.cancel')}
            </button>
          </div>
        </fieldset>
      )}
      {result && (
        <p ref={feedback} role="status" tabIndex={-1}>
          {outcomeText(result)}
        </p>
      )}
      {reset.error && <p role="alert">{t('provider_actions.reset_failed')}</p>}
      {canCopy && (
        <>
          <div className="preview-action-row">
            <span>{t('provider_actions.low_priority_offer')}</span>
            <button
              className="link-btn"
              type="button"
              disabled={copy.pending}
              onClick={() => void copy.run()}
            >
              {t('provider_actions.copy_command')}
            </button>
          </div>
          <LowPriorityHint />
          {copied && <p role="status">{t('provider_actions.copied')}</p>}
          {copy.error && <p role="alert">{t('provider_actions.copy_failed')}</p>}
        </>
      )}
    </div>
  );
}
