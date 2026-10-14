import { useEffect, useRef } from 'react';
import { t } from '../../i18n';

export function PendingChanges({
  waiting,
  dirty,
  origin,
  onCancel,
  onDiscard,
}: {
  waiting: boolean;
  dirty: boolean;
  origin: HTMLElement | null;
  onCancel: () => void;
  onDiscard: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const keep = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    const node = dialog.current;
    if (!node) return;
    // The fallback also permits contract tests in DOM implementations without native dialogs.
    if (typeof node.showModal === 'function') node.showModal();
    else node.setAttribute('open', '');
    keep.current?.focus();
    return () => {
      if (typeof node.close === 'function') node.close();
      else node.removeAttribute('open');
      // Restore after removing the modal: focusing the underlying page before close is ignored.
      if (origin?.isConnected) origin.focus();
    };
  }, [origin]);
  return (
    <dialog
      ref={dialog}
      className="pending-dialog"
      role="alertdialog"
      aria-labelledby="pending-title"
      aria-describedby="pending-body"
      onCancel={(event) => {
        event.preventDefault();
        onCancel();
      }}
      onKeyDown={(event) => {
        if (event.key === 'Tab') {
          const buttons =
            event.currentTarget.querySelectorAll<HTMLButtonElement>('button:not(:disabled)');
          const first = buttons[0];
          const last = buttons[buttons.length - 1];
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last?.focus();
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first?.focus();
          }
        }
        if (event.key === 'Escape') {
          event.preventDefault();
          event.stopPropagation();
          onCancel();
        }
      }}
    >
      <h2 id="pending-title">
        {t(waiting ? 'draft.wait_title' : dirty ? 'draft.title' : 'draft.done_title')}
      </h2>
      <p id="pending-body">{t(waiting ? 'draft.wait' : dirty ? 'draft.body' : 'draft.done')}</p>
      <div className="actions">
        <button ref={keep} type="button" className="btn primary" onClick={onCancel}>
          {t('draft.keep')}
        </button>
        <button type="button" className="btn" disabled={waiting} onClick={onDiscard}>
          {t(dirty ? 'draft.discard' : 'draft.continue')}
        </button>
      </div>
    </dialog>
  );
}
