import { useCallback, useEffect, useId, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { harnessName } from '../app/harness';
import type { ViewContext } from '../app/windowModel';
import { HarnessLogo } from '../components/brand/HarnessLogo';
import { Icon } from '../components/icons';
import { TrayRing } from '../components/shell/TrayRing';
import { t } from '../i18n';
import type { HarnessView } from '../lib/contract';
import { criticalViews, usageChip } from './model';

/** Portal keeps the complete tooltip outside the flyout's clipped scroll area. */
export function ProviderOverview({
  views,
  ctx,
  tray = false,
}: {
  views: readonly HarnessView[];
  ctx: ViewContext;
  tray?: boolean;
}) {
  const id = useId();
  const trigger = useRef<HTMLButtonElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [position, setPosition] = useState<{ top: number; right: number } | null>(null);
  const cancelClose = useCallback(() => {
    if (timer.current) clearTimeout(timer.current);
  }, []);
  const show = () => {
    cancelClose();
    const rect = trigger.current?.getBoundingClientRect();
    if (!rect) return;
    const height = Math.min(520, window.innerHeight - 32);
    setPosition({
      top: Math.max(
        16,
        Math.min(tray ? rect.top - height : rect.bottom + 4, window.innerHeight - height - 16),
      ),
      right: Math.max(16, window.innerWidth - rect.right),
    });
  };
  const close = () => {
    cancelClose();
    timer.current = setTimeout(() => setPosition(null), 140);
  };
  useEffect(() => {
    const dismiss = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || event.isComposing) return;
      event.preventDefault();
      setPosition(null);
    };
    if (position !== null) document.addEventListener('keydown', dismiss, true);
    return () => {
      cancelClose();
      document.removeEventListener('keydown', dismiss, true);
    };
  }, [position, cancelClose]);
  const critical = criticalViews(views, ctx).length;
  return (
    <>
      <button
        ref={trigger}
        className="preview-overview-trigger"
        type="button"
        aria-label="Resumo de todos os providers"
        aria-describedby={position ? id : undefined}
        onMouseEnter={show}
        onMouseLeave={close}
        onFocus={show}
        onBlur={close}
        onClick={show}
      >
        <TrayRing pct={null} status={critical > 0 ? 'crit' : null} />
        <span>Visão geral</span>
        <Icon name="info" />
      </button>
      {position &&
        createPortal(
          <div
            id={id}
            role="tooltip"
            className="preview-tooltip"
            style={{ top: position.top, right: position.right }}
            onMouseEnter={cancelClose}
            onMouseLeave={close}
          >
            <div className="preview-tooltip-head">
              <strong>Todos os providers</strong>
              <span>{views.length} fontes</span>
            </div>
            <p className="hint">Uso da janela mais limitada</p>
            <ul className="preview-tooltip-list">
              {views.map((view) => {
                const chip = usageChip(view, ctx);
                return (
                  <li key={view.id} data-status={chip.kind === 'pct' ? chip.status : undefined}>
                    <HarnessLogo id={view.id} size={16} />
                    <span>{harnessName(view.id)}</span>
                    <small>
                      {chip.kind === 'pct'
                        ? `${chip.text} usado · ${t(`status.${chip.status}`)}`
                        : chip.kind === 'text'
                          ? chip.text
                          : 'Carregando'}
                    </small>
                  </li>
                );
              })}
            </ul>
            <div className="preview-tooltip-foot">
              {critical > 0
                ? `${critical} em uso crítico (≥ 90%)`
                : 'Nenhum provider com uso ≥ 90%'}
            </div>
          </div>,
          document.body,
        )}
    </>
  );
}
