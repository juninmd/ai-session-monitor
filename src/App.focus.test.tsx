import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderApp } from './test/helpers';

const tab = (name: string) => screen.getByRole('tab', { name: new RegExp(`^${name}\\b`) });

// Chromium drops the keyboard on <body> when the focused control is disabled or removed. A card
// that replaces itself after a click must leave the keyboard on something that stays mounted.
describe('the keyboard when an action replaces its own card', () => {
  it('stays on the tab after Claude Code consent installs the command', async () => {
    const { user } = renderApp({ scenario: 'fresh' });
    (await screen.findByRole('button', { name: 'Instalar comando' })).focus();
    await user.keyboard('{Enter}');
    expect(
      await screen.findByRole('heading', { name: 'Aguardando a primeira leitura' }),
    ).toBeVisible();
    expect(tab('Claude Code')).toHaveFocus();
  });

  it('stays on the tab after Codex consent', async () => {
    const { user } = renderApp({ scenario: 'fresh' });
    await user.click(tab('Codex'));
    (await screen.findByRole('button', { name: 'Permitir leitura' })).focus();
    await user.keyboard('{Enter}');
    expect(await screen.findByRole('heading', { name: 'Janela de 5 horas' })).toBeVisible();
    expect(tab('Codex')).toHaveFocus();
  });

  it('stays on the tab when "Agora não" switches the source off', async () => {
    const { user } = renderApp({ scenario: 'fresh' });
    (await screen.findByRole('button', { name: 'Agora não' })).focus();
    await user.keyboard('{Enter}');
    expect(
      await screen.findByRole('heading', { name: 'Claude Code está desligado' }),
    ).toBeVisible();
    expect(tab('Claude Code')).toHaveFocus();
  });
});
