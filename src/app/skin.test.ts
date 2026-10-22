import { afterEach, describe, expect, it } from 'vitest';
import { applySkin, loadSkin, skinOptions } from './skin';

afterEach(() => {
  window.localStorage.clear();
  delete document.documentElement.dataset.skin;
});

describe('skins', () => {
  it('offers the Canto palettes and the system look', () => {
    expect(skinOptions().map((skin) => skin.id)).toEqual([
      'sistema',
      'padrao',
      'hueco-mundo',
      'dracula',
      'glass',
      'claro',
    ]);
  });

  it('applies a skin to the root and remembers it', () => {
    applySkin('dracula');
    expect(document.documentElement.dataset.skin).toBe('dracula');
    expect(loadSkin()).toBe('dracula');
  });

  it('leaves no attribute for the system look', () => {
    applySkin('glass');
    applySkin('sistema');
    expect(document.documentElement.dataset.skin).toBeUndefined();
    expect(loadSkin()).toBe('sistema');
  });

  it('falls back to the system look for a value it does not know', () => {
    window.localStorage.setItem('aism.skin', 'neon');
    expect(loadSkin()).toBe('sistema');
  });
});
