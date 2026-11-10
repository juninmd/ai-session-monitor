// The message catalogs, keyed by the tag the settings store. English is the master: its keys are
// the MessageKey type, it ships in the main bundle and it is the fallback for a key another locale
// lacks (see t()). Every other locale is its own chunk, fetched the first time it is wanted: nine
// unused languages would otherwise be parsed on every start of a flyout that shows one.
import type { Locale } from './locale';
import { en } from './locales/en';

export type MessageKey = keyof typeof en;
export type Catalog = Readonly<Record<MessageKey, string>>;
export type PartialCatalog = Readonly<Partial<Record<MessageKey, string>>>;

type LazyLocale = Exclude<Locale, 'en'>;

const LOADERS: Readonly<Record<LazyLocale, () => Promise<PartialCatalog>>> = {
  'zh-CN': () => import('./locales/zh-CN').then((module) => module.zhCN),
  hi: () => import('./locales/hi').then((module) => module.hi),
  es: () => import('./locales/es').then((module) => module.es),
  fr: () => import('./locales/fr').then((module) => module.fr),
  ar: () => import('./locales/ar').then((module) => module.ar),
  bn: () => import('./locales/bn').then((module) => module.bn),
  'pt-BR': () => import('./locales/pt-BR').then((module) => module.ptBR),
  ru: () => import('./locales/ru').then((module) => module.ru),
  ur: () => import('./locales/ur').then((module) => module.ur),
};

const loaded = new Map<Locale, PartialCatalog>([['en', en]]);
const pending = new Map<Locale, Promise<void>>();

/** The English catalog: always present, the source of every key. */
export const ENGLISH: Catalog = en;

/** A locale's messages, or undefined while its chunk has not arrived (t() then reads English). */
export const catalogOf = (locale: Locale): PartialCatalog | undefined => loaded.get(locale);

export const hasCatalog = (locale: Locale): boolean => loaded.has(locale);

/**
 * Fetches a locale's chunk once. Rejects when the chunk cannot be read; the caller may still switch
 * to the locale, since every key falls back to English.
 */
export function loadCatalog(locale: Locale): Promise<void> {
  if (locale === 'en' || loaded.has(locale)) return Promise.resolve();
  const running = pending.get(locale);
  if (running) return running;
  const started = LOADERS[locale]()
    .then((catalog) => {
      loaded.set(locale, catalog);
    })
    .finally(() => pending.delete(locale));
  pending.set(locale, started);
  return started;
}

/** Every locale at once: the test suite and the catalog checks read them all. */
export async function loadAllCatalogs(): Promise<void> {
  await Promise.all((Object.keys(LOADERS) as LazyLocale[]).map(loadCatalog));
}
