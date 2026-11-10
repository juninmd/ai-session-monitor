// The catalogs load on demand. English is always there; another locale reads English until its
// chunk arrives. The suite's setup preloads every locale, so this file takes a fresh module graph.
import { beforeEach, describe, expect, it, vi } from 'vitest';

async function fresh() {
  vi.resetModules();
  const catalogs = await import('./catalogs');
  const locale = await import('./locale');
  const i18n = await import('./index');
  return { ...catalogs, ...locale, t: i18n.t };
}

describe('lazy catalogs', () => {
  beforeEach(() => vi.resetModules());

  it('starts with English only', async () => {
    const { hasCatalog, catalogOf } = await fresh();
    expect(hasCatalog('en')).toBe(true);
    expect(hasCatalog('fr')).toBe(false);
    expect(catalogOf('fr')).toBeUndefined();
  });

  it('reads English for a locale whose chunk has not arrived, then its own words', async () => {
    const { loadCatalog, setLocale, releaseLocale, t, ENGLISH } = await fresh();
    releaseLocale();
    setLocale('fr');
    expect(t('action.retry')).toBe(ENGLISH['action.retry']);
    await loadCatalog('fr');
    expect(t('action.retry')).not.toBe(ENGLISH['action.retry']);
  });

  it('fetches a chunk once, however many callers ask', async () => {
    const { loadCatalog, hasCatalog } = await fresh();
    const first = loadCatalog('es');
    expect(loadCatalog('es')).toBe(first);
    await first;
    expect(hasCatalog('es')).toBe(true);
  });

  it('loads every locale the picker offers', async () => {
    const { loadAllCatalogs, hasCatalog, LOCALES } = await fresh();
    await loadAllCatalogs();
    expect(LOCALES.filter((locale) => !hasCatalog(locale.id))).toEqual([]);
  });
});
