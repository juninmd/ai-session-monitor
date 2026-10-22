// The active interface language. Settings own the choice (general.language); App mirrors it here
// so t() stays a plain function. Presentation only: a locale never changes what is measured.
import { useSyncExternalStore } from 'react';
import type { Language } from '../lib/contract';

export type Locale = Language;

export const DEFAULT_LOCALE: Locale = 'en';

export interface LocaleInfo {
  id: Locale;
  /** The language's own name: a picker must stay readable to someone who cannot read the UI. */
  name: string;
  dir: 'ltr' | 'rtl';
}

/** The ten most spoken languages (Ethnologue, total speakers), English first. */
export const LOCALES: readonly LocaleInfo[] = [
  { id: 'en', name: 'English', dir: 'ltr' },
  { id: 'zh-CN', name: '简体中文', dir: 'ltr' },
  { id: 'hi', name: 'हिन्दी', dir: 'ltr' },
  { id: 'es', name: 'Español', dir: 'ltr' },
  { id: 'fr', name: 'Français', dir: 'ltr' },
  { id: 'ar', name: 'العربية', dir: 'rtl' },
  { id: 'bn', name: 'বাংলা', dir: 'ltr' },
  { id: 'pt-BR', name: 'Português (Brasil)', dir: 'ltr' },
  { id: 'ru', name: 'Русский', dir: 'ltr' },
  { id: 'ur', name: 'اردو', dir: 'rtl' },
];

export const isLocale = (value: unknown): value is Locale =>
  LOCALES.some((locale) => locale.id === value);

let active: Locale = DEFAULT_LOCALE;
let pinned = false;
const listeners = new Set<() => void>();

export const getLocale = (): Locale => active;

export function setLocale(next: Locale): void {
  if (pinned || next === active) return;
  active = next;
  for (const listener of listeners) listener();
}

/**
 * Test seam: holds the locale at `next` and ignores `setLocale` until `releaseLocale()`. The suite
 * pins pt-BR so the existing assertions keep reading Portuguese whatever the stored setting says.
 */
export function pinLocale(next: Locale): void {
  pinned = false;
  setLocale(next);
  pinned = true;
}

export function releaseLocale(): void {
  pinned = false;
}

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

/** The active locale as React state: components that read it re-render when it changes. */
export function useLocale(): Locale {
  return useSyncExternalStore(subscribe, getLocale, getLocale);
}

/** `lang` and `dir` on the document, so the browser picks fonts and mirrors the layout. */
export function applyDocumentLocale(locale: Locale): void {
  const info = LOCALES.find((item) => item.id === locale);
  document.documentElement.lang = locale;
  document.documentElement.dir = info?.dir ?? 'ltr';
}

const STORAGE_KEY = 'aism.locale';

/** The last language the settings held, so the next start opens in it before they have loaded. */
export function recalledLocale(): Locale | null {
  try {
    const saved = window.localStorage.getItem(STORAGE_KEY);
    return isLocale(saved) ? saved : null;
  } catch {
    return null;
  }
}

export function rememberLocale(locale: Locale): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Not remembered; the settings still say it.
  }
}
