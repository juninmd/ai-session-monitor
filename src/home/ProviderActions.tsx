import { useContext, useEffect, useRef, useState } from 'react';
import { useAsyncAction } from '../app/useAsyncAction';
import type { ViewContext } from '../app/windowModel';
import type { HarnessView } from '../lib/contract';
import { PreviewActionsContext, type ResetOutcome, resetCreditCount } from './actionModel';

const outcomeText: Record<ResetOutcome, string> = {
  reset: 'Reset simulado. Cotas e saldo atualizados na prévia.',
  noCredit: 'Nenhum reset disponível. O saldo mudou; atualize a leitura.',
  nothingToReset: 'Não há cota a renovar na prévia.',
  unavailable: 'A ação não está disponível nesta leitura.',
};

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
      {canReset && (
        <div className="preview-action-row">
          <span>
            {count} {count === 1 ? 'reset disponível' : 'resets disponíveis'}
          </span>
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
            Usar reset
          </button>
        </div>
      )}
      {confirming && canReset && (
        <fieldset className="preview-reset-confirm" aria-label="Confirmar reset de cota">
          <p>Usar 1 reset do Codex? O consumo é definitivo. Nesta prévia, a ação é simulada.</p>
          <div className="preview-action-row">
            <button
              className="btn"
              type="button"
              disabled={reset.pending}
              onClick={() => void reset.run()}
            >
              Confirmar reset
            </button>
            <button
              className="link-btn"
              type="button"
              disabled={reset.pending}
              onClick={() => setConfirming(false)}
            >
              Cancelar
            </button>
          </div>
        </fieldset>
      )}
      {result && (
        <p ref={feedback} role="status" tabIndex={-1}>
          {outcomeText[result]}
        </p>
      )}
      {reset.error && <p role="alert">Não foi possível simular o reset. Tente novamente.</p>}
      {canCopy && (
        <>
          <div className="preview-action-row">
            <span>Oferta de baixa prioridade · prévia</span>
            <button
              className="link-btn"
              type="button"
              disabled={copy.pending}
              onClick={() => void copy.run()}
            >
              Copiar /low-priority
            </button>
          </div>
          <p>
            Use <code>/low-priority</code> na sessão interativa que ofereceu esse modo.
          </p>
          {copied && (
            <p role="status">Comando copiado. O modo ainda precisa ser ativado no Claude Code.</p>
          )}
          {copy.error && (
            <p role="alert">
              Não foi possível copiar. Selecione o comando acima e copie manualmente.
            </p>
          )}
        </>
      )}
    </div>
  );
}
