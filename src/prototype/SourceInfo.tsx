import { useCallback, useEffect, useId, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { Icon } from '../components/icons';
import { t } from '../i18n';

/** An inline source indicator; the owning tab provides focus without nested controls. */
export function SourceInfo({
  keyboardFocus,
  onDismiss,
}: {
  keyboardFocus: boolean;
  onDismiss: () => void;
}) {
  const anchor = useRef<HTMLSpanElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hadKeyboardFocus = useRef(false);
  const [position, setPosition] = useState<{ top: number; left: number } | null>(null);
  const id = useId();
  const cancelClose = useCallback(() => {
    if (timer.current !== null) clearTimeout(timer.current);
    timer.current = null;
  }, []);
  const show = useCallback(() => {
    cancelClose();
    const rect = anchor.current?.getBoundingClientRect();
    if (!rect) return;
    const width = Math.min(280, window.innerWidth - 32);
    setPosition({
      top: Math.max(16, Math.min(rect.bottom + 6, window.innerHeight - 100)),
      left: Math.max(16, Math.min(rect.right - width, window.innerWidth - width - 16)),
    });
  }, [cancelClose]);
  const close = useCallback(() => {
    cancelClose();
    timer.current = setTimeout(() => setPosition(null), 140);
  }, [cancelClose]);
  useEffect(() => {
    if (keyboardFocus) show();
    else if (hadKeyboardFocus.current) {
      cancelClose();
      setPosition(null);
    }
    hadKeyboardFocus.current = keyboardFocus;
  }, [keyboardFocus, show, cancelClose]);
  useEffect(() => cancelClose, [cancelClose]);
  useEffect(() => {
    if (position === null) return;
    const dismiss = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return;
      event.preventDefault();
      event.stopPropagation();
      cancelClose();
      setPosition(null);
      onDismiss();
    };
    const reposition = () => show();
    window.addEventListener('keydown', dismiss, true);
    window.addEventListener('resize', reposition);
    window.addEventListener('scroll', reposition, true);
    return () => {
      window.removeEventListener('keydown', dismiss, true);
      window.removeEventListener('resize', reposition);
      window.removeEventListener('scroll', reposition, true);
    };
  }, [position, cancelClose, onDismiss, show]);
  return (
    <>
      <span
        ref={anchor}
        role="img"
        aria-label={t('meta.unofficial')}
        className="preview-source-info"
        title={t('meta.unofficial')}
        onMouseEnter={show}
        onMouseLeave={close}
        aria-describedby={position ? id : undefined}
      >
        <Icon name="info" />
        <span className="vh"> {t('meta.unofficial')}</span>
      </span>
      {position &&
        createPortal(
          <div
            className="preview-source-tooltip"
            id={id}
            role="tooltip"
            style={position}
            onMouseEnter={cancelClose}
            onMouseLeave={close}
          >
            <strong>{t('meta.unofficial')}</strong>
            <p>{t('meta.unofficial_tip')}</p>
          </div>,
          document.body,
        )}
    </>
  );
}
