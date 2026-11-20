// Decides whether an Alt+digit keydown is the surface shortcut or just a key typed for something else.
export interface SurfaceKeyEvent {
  key: string;
  code: string;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  repeat: boolean;
  isComposing: boolean;
  target: EventTarget | null;
  getModifierState: (key: string) => boolean;
}

function isEditable(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el || typeof el.tagName !== 'string') return false;
  return ['INPUT', 'TEXTAREA', 'SELECT'].includes(el.tagName) || el.isContentEditable === true;
}

/** The surface Alt+digit points at, or undefined when the key must pass through untouched. */
export function surfaceFromKey<T>(event: SurfaceKeyEvent, surfaces: readonly T[]): T | undefined {
  if (!event.altKey || event.ctrlKey || event.metaKey) return undefined;
  // Alt+numpad types a character code, AltGr composes one, a held key repeats, a text field owns its keys.
  if (event.repeat || event.isComposing || event.code.startsWith('Numpad')) return undefined;
  if (event.getModifierState('AltGraph') || isEditable(event.target)) return undefined;
  if (!/^[1-9]$/.test(event.key)) return undefined;
  return surfaces[Number(event.key) - 1];
}
