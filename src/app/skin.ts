// Skins: the colour palettes of the Canto widget. Presentation only, so the choice stays in this
// webview (localStorage) and never reaches the settings contract or the backend.
import { t } from '../i18n';

export type SkinId = 'sistema' | 'padrao' | 'hueco-mundo' | 'dracula' | 'glass' | 'claro';

export const SKINS: readonly { id: SkinId; label: string }[] = [
  { id: 'sistema', label: t('skin.system') },
  { id: 'padrao', label: t('skin.default') },
  { id: 'hueco-mundo', label: t('skin.hueco_mundo') },
  { id: 'dracula', label: t('skin.dracula') },
  { id: 'glass', label: t('skin.glass') },
  { id: 'claro', label: t('skin.light') },
];

const KEY = 'aism.skin';

const isSkin = (value: unknown): value is SkinId => SKINS.some((skin) => skin.id === value);

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
