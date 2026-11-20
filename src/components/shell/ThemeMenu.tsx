// The theme control of the header: one icon that opens a menu with every skin, instead of cycling
// through them blindly. Picking one applies it and floods the frame with the new colours from the
// icon outward, so the change reads as an event and not as a flicker.
import { type CSSProperties, type KeyboardEvent, useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { applySkin, loadSkin, SKIN_IDS, type SkinId, skinOptions } from '../../app/model/skin';
import { t } from '../../i18n';
import { Icon } from '../icons';

interface Wash {
  x: number;
  y: number;
  key: number;
}

export function ThemeMenu() {
  const [skin, setSkin] = useState<SkinId>(loadSkin);
  const [open, setOpen] = useState(false);
  const [wash, setWash] = useState<Wash | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const options = skinOptions();
  const skinLabel = options.find((option) => option.id === skin)?.label ?? '';
  const themeLabel = t('header.theme', { name: skinLabel });

  // The checked skin takes the keyboard when the menu opens.
  useEffect(() => {
    if (!open) return;
    menuRef.current?.querySelector<HTMLElement>('[aria-checked="true"]')?.focus();
  }, [open]);

  // A press anywhere else closes the menu without picking.
  useEffect(() => {
    if (!open) return;
    const onPointer = (event: PointerEvent) => {
      if (event.target instanceof Node && !rootRef.current?.contains(event.target)) setOpen(false);
    };
    document.addEventListener('pointerdown', onPointer);
    return () => document.removeEventListener('pointerdown', onPointer);
  }, [open]);

  const choose = (next: SkinId) => {
    const frame = rootRef.current?.closest('.nx-fly');
    const button = triggerRef.current;
    if (next !== skin && frame && button) {
      const f = frame.getBoundingClientRect();
      const b = button.getBoundingClientRect();
      setWash({
        x: b.left + b.width / 2 - f.left,
        y: b.top + b.height / 2 - f.top,
        key: Date.now(),
      });
    }
    setSkin(next);
    applySkin(next);
    setOpen(false);
    button?.focus();
  };

  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key === 'Escape') {
      // Closes the menu only: the flyout's own Escape would hide the whole window.
      event.preventDefault();
      setOpen(false);
      triggerRef.current?.focus();
      return;
    }
    const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
    if (step === 0) return;
    event.preventDefault();
    const items = [
      ...(menuRef.current?.querySelectorAll<HTMLElement>('[role="menuitemradio"]') ?? []),
    ];
    const at = items.indexOf(document.activeElement as HTMLElement);
    items[(at + step + items.length) % items.length]?.focus();
  };

  const frame = rootRef.current?.closest('.nx-fly');
  return (
    <div className="nx-theme" ref={rootRef}>
      <button
        ref={triggerRef}
        type="button"
        className="icon-btn"
        aria-label={themeLabel}
        title={themeLabel}
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((on) => !on)}
      >
        <Icon name="palette" />
      </button>
      {open && (
        <div
          ref={menuRef}
          className="nx-theme-menu"
          role="menu"
          aria-label={t('header.theme_menu')}
          tabIndex={-1}
          onKeyDown={onKeyDown}
        >
          {SKIN_IDS.map((id) => (
            <button
              key={id}
              type="button"
              role="menuitemradio"
              aria-checked={id === skin}
              className="nx-theme-item"
              data-skin-swatch={id}
              onClick={() => choose(id)}
            >
              <i aria-hidden="true" />
              {options.find((option) => option.id === id)?.label}
              {id === skin && <Icon name="check" className="nx-theme-check" />}
            </button>
          ))}
        </div>
      )}
      {wash !== null &&
        frame &&
        createPortal(
          <span
            key={wash.key}
            className="nx-wash"
            aria-hidden="true"
            style={{ '--x': `${wash.x}px`, '--y': `${wash.y}px` } as CSSProperties}
            onAnimationEnd={() => setWash(null)}
          />,
          frame,
        )}
    </div>
  );
}
