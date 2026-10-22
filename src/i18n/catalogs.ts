// Every locale's catalog, keyed by the tag the settings store. Keys are the pt-BR catalog's until
// English becomes the master (see locales/en).
import type { Locale } from './locale';
import { en } from './locales/en';
import { ptBR } from './locales/pt-BR';

export type MessageKey = keyof typeof ptBR;
export type Catalog = Readonly<Record<MessageKey, string>>;

/** Locales not written yet resolve to the English catalog, key by key (see t()). */
export const CATALOGS: Readonly<Partial<Record<Locale, Partial<Catalog>>>> = {
  en,
  'pt-BR': ptBR,
};
