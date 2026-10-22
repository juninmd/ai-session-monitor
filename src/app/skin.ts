// Skins: the colour palettes of the Canto widget. Presentation only, so the choice stays in this
// webview (localStorage) and never reaches the settings contract or the backend.
import { type MessageKey, t } from '../i18n';

export type SkinId = 'sistema' | 'padrao' | 'hueco-mundo' | 'dracula' | 'glass' | 'claro';

export const SKIN_IDS: readonly SkinId[] = [
  'sistema',
  'padrao',
  'hueco-mundo',
  'dracula',
  'glass',
  'claro',
];

const LABELS: Readonly<Record<SkinId, MessageKey>> = {
  sistema: 'skin.system',
  padrao: 'skin.default',
  'hueco-mundo': 'skin.hueco_mundo',
  dracula: 'skin.dracula',
  glass: 'skin.glass',
  claro: 'skin.light',
};

/** Read at call time: the language can change while the app runs. */
export const skinOptions = (): { id: SkinId; label: string }[] =>
  SKIN_IDS.map((id) => ({ id, label: t(LABELS[id]) }));

const KEY = 'aism.skin';

const isSkin = (value: unknown): value is SkinId => SKIN_IDS.some((id) => id === value);

/** Storage can be blocked or empty (private window, cleared data): the system look is the default. */
export function loadSkin(): SkinId {
  try {
    const saved = window.localStorage.getItem(KEY);
    return isSkin(saved) ? saved : 'sistema';
  } catch {
    return 'sistema';
  }
}

export function applySkin(skin: SkinId): void {
  const root = document.documentElement;
  if (skin === 'sistema') delete root.dataset.skin;
  else root.dataset.skin = skin;
  try {
    window.localStorage.setItem(KEY, skin);
  } catch {
    // The skin still applies for this run; it just will not be remembered.
  }
}
