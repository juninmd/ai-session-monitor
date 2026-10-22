// Every locale's catalog, keyed by the tag the settings store. English is the master: its keys are
// the MessageKey type, and a locale that lacks a key shows the English text (see t()).
import type { Locale } from './locale';
import { ar } from './locales/ar';
import { bn } from './locales/bn';
import { en } from './locales/en';
import { es } from './locales/es';
import { fr } from './locales/fr';
import { hi } from './locales/hi';
import { ptBR } from './locales/pt-BR';
import { ru } from './locales/ru';
import { ur } from './locales/ur';
import { zhCN } from './locales/zh-CN';

export type MessageKey = keyof typeof en;
export type Catalog = Readonly<Record<MessageKey, string>>;

export const CATALOGS: Readonly<
  Partial<Record<Locale, Readonly<Partial<Record<MessageKey, string>>>>>
> = {
  en,
  'zh-CN': zhCN,
  hi,
  es,
  fr,
  ar,
  bn,
  'pt-BR': ptBR,
  ru,
  ur,
};
