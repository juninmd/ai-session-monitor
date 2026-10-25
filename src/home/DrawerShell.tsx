// The lateral layer: a scrim over the whole flyout and a drawer against its trailing edge. A 27 rem
// flyout has no room for two columns, so secondary content covers the frame instead of stacking
// under it. Both the provider detail and one settings section ride on this shell; it owns the bar,
// the scrolling body, and the two things every drawer does — take the keyboard on open, close on
// the scrim.
import { type KeyboardEvent, type ReactNode, useEffect, useRef } from 'react';
import { Icon } from '../components/icons';

const FOCUSABLE =
  'button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex="0"]';

export interface DrawerShellProps {
  /** The drawer's own heading id: the dialog and the shell focus it through it. */
  titleId: string;
  title: ReactNode;
  /** A focusable heading lets the shell park the keyboard there when the flyout comes back. */
  titleFocusable?: boolean;
  /** The close button's id, when something outside has to find it. */
  closeId?: string;
  closeLabel: string;
  /** `wide` leaves a narrow reveal of the content behind a 20 px scrim instead of a column. */
  size?: 'wide';
  onClose: () => void;
  /** Controls of the bar, after the title. */
  actions?: ReactNode;
  children: ReactNode;
}

export function DrawerShell({
  titleId,
  title,
  titleFocusable,
  closeId,
  closeLabel,
  size,
  onClose,
  actions,
  children,
}: DrawerShellProps) {
  const closeRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLElement>(null);
  // Focus moves into the drawer on open; the stage puts it back where it came from.
  useEffect(() => {
    closeRef.current?.focus();
  }, []);
  // The scrim covers the flyout but leaves its controls in the tab order; Tab wraps inside the drawer.
  const trapTab = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key !== 'Tab') return;
    const controls = panelRef.current?.querySelectorAll<HTMLElement>(FOCUSABLE) ?? [];
    const first = controls[0];
    const last = controls[controls.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  };
  return (
    <div className="nx-drawer-host">
      <button type="button" className="nx-drawer-scrim" aria-label={closeLabel} onClick={onClose} />
      <aside
        ref={panelRef}
        className="nx-drawer"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        data-size={size}
        onKeyDown={trapTab}
      >
        <header className="nx-drawer-bar">
          <button
            ref={closeRef}
            id={closeId}
            type="button"
            className="icon-btn"
            aria-label={closeLabel}
            onClick={onClose}
          >
            <Icon name="back" />
          </button>
          <h2 className="nx-panel-title" id={titleId} tabIndex={titleFocusable ? -1 : undefined}>
            {title}
          </h2>
          {actions}
        </header>
        <div className="nx-drawer-body">{children}</div>
      </aside>
    </div>
  );
}
