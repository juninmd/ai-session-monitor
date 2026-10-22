// Mirrors general.language into the i18n module and the document. Returns the active locale so the
// caller can re-key what it renders: texts computed at mount (and memoised ones) must be rebuilt.
import { useEffect, useLayoutEffect } from 'react';
import {
  applyDocumentLocale,
  type Locale,
  rememberLocale,
  setLocale,
  useLocale,
} from '../i18n/locale';

export function useLocaleSync(language: Locale, loaded: boolean): Locale {
  // Until the stored settings are in, `language` is the default: do not flash English at someone
  // who chose another language. The last choice is remembered by the document itself (recalledLocale, read at start).
  const wanted = loaded ? language : null;
  useLayoutEffect(() => {
    if (wanted === null) return;
    setLocale(wanted);
    rememberLocale(wanted);
  }, [wanted]);
  const locale = useLocale();
  useEffect(() => applyDocumentLocale(locale), [locale]);
  return locale;
}
