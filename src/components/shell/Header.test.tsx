import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it } from 'vitest';
import { Header } from './Header';

describe('the theme icon', () => {
  afterEach(() => {
    window.localStorage.clear();
    delete document.documentElement.dataset.skin;
  });

  it('opens a menu of every theme, applies the pick and remembers it', async () => {
    render(
      <Header
        tray={null}
        settingsOpen={false}
        errorCount={0}
        onRefresh={() => {}}
        onToggleSettings={() => {}}
        onQuit={() => {}}
      />,
    );
    expect(document.documentElement.dataset.skin).toBeUndefined();
    await userEvent.click(screen.getByRole('button', { name: /^Tema:/ }));
    expect(screen.getAllByRole('menuitemradio')).toHaveLength(6);
    await userEvent.click(screen.getByRole('menuitemradio', { name: 'Padrão' }));
    expect(document.documentElement.dataset.skin).toBe('padrao');
    expect(window.localStorage.getItem('aism.skin')).toBe('padrao');
    expect(screen.queryByRole('menu')).toBeNull();
  });
});
