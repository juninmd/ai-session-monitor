import { render } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { pinLocale, releaseLocale, setLocale } from '../i18n/locale';
import { CommandPalette } from './CommandPalette';
import type { PaletteCommand } from './paletteModel';

const refresh: PaletteCommand = {
  id: 'refresh',
  group: 'actions',
  label: 'Refresh',
  hint: '',
  keywords: '',
  run: vi.fn(),
};

function footerKeys() {
  return Array.from(document.querySelectorAll('kbd')).map((key) => key.textContent);
}

describe('the palette key labels', () => {
  beforeEach(() => releaseLocale());
  // The suite pins pt-BR for its other tests; pin it again so they keep reading Portuguese.
  afterEach(() => pinLocale('pt-BR'));

  it('names the keys in the language of the flyout', () => {
    setLocale('fr');
    render(<CommandPalette commands={[refresh]} onClose={vi.fn()} />);
    expect(footerKeys()).toEqual(expect.arrayContaining(['Échap', 'Entrée']));
  });

  it('names the keys in Spanish, where Enter is Intro', () => {
    setLocale('es');
    render(<CommandPalette commands={[refresh]} onClose={vi.fn()} />);
    expect(footerKeys()).toEqual(expect.arrayContaining(['Esc', 'Intro']));
  });
});
