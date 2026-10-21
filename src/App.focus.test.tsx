import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { renderApp } from './test/helpers';

const tile = (name: string) => screen.getByRole('button', { name: new RegExp(`^${name}`) });
const ready = () => screen.findByRole('button', { name: /^Codex/ });
const drawer = () => screen.getByRole('dialog');

// Chromium drops the keyboard on <body> when the focused control is disabled or removed. An action
// that replaces its own card must leave the keyboard on something that stays mounted: here the row
// behind the drawer, and the settings screen an error card opens.
describe('the keyboard when an action replaces its own card', () => {
  it('moves into the drawer, and back to the row that opened it', async () => {
    const { user } = renderApp();
    await ready();
    await user.click(tile('Codex'));
    expect(document.getElementById('nx-drawer-close')).toHaveFocus();
    await user.keyboard('{Escape}');
    expect(tile('Codex')).toHaveFocus();
  });

  it('lands on the group of the source whose error action it followed', async () => {
    const { user } = renderApp({ scenario: 'errors' });
    await screen.findByRole('button', { name: /^Triagem/ });
    await user.click(tile('Codex'));
    await user.click(await within(drawer()).findByRole('button', { name: 'Abrir configurações' }));
    const group = document.getElementById('grp-codex') as HTMLDetailsElement;
    await vi.waitFor(() => expect(group.querySelector('summary')).toHaveFocus());
  });

  it('returns to the row after leaving the settings an action opened', async () => {
    const { user } = renderApp({ scenario: 'errors' });
    await screen.findByRole('button', { name: /^Triagem/ });
    await user.click(tile('Codex'));
    await user.click(await within(drawer()).findByRole('button', { name: 'Abrir configurações' }));
    // The first arrow leaves the source's section; the second leaves the settings screen.
    await user.click(within(drawer()).getByRole('button', { name: 'Voltar' }));
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(tile('Codex')).toHaveFocus();
  });
});

describe('surface shortcuts', () => {
  it('switches the vertical tab with Alt+digit', async () => {
    const { user } = renderApp();
    await ready();
    await user.keyboard('{Alt>}2{/Alt}');
    expect(screen.getByRole('button', { name: /^Horizonte/ })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    await user.keyboard('{Alt>}1{/Alt}');
    expect(screen.getByRole('button', { name: /^Triagem/ })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });
});
