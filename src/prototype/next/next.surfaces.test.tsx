// The stage surfaces that are not provider lists: the command palette, the first-run wizard, the
// evidence list, and the side-by-side comparison.
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { mount, panel, rows } from './testHarness';

describe('command palette', () => {
  it('opens with Ctrl+K, filters and runs the highlighted command', async () => {
    const { user } = mount();
    await rows('data-tier');
    fireEvent.keyDown(window, { key: 'k', ctrlKey: true });
    const dialog = await screen.findByRole('dialog', { name: 'Comando rápido' });
    await user.type(within(dialog).getByRole('combobox'), 'codex');
    await user.keyboard('{Enter}');
    expect(screen.queryByRole('dialog', { name: 'Comando rápido' })).toBeNull();
    expect(panel('Codex')).toBeVisible();
  });

  it('closes on Escape and returns focus to whatever opened it', async () => {
    const { user } = mount();
    await rows('data-tier');
    const opener = screen.getByRole('button', { name: /^Triagem/ });
    opener.focus();
    fireEvent.keyDown(window, { key: 'k', ctrlKey: true });
    const dialog = await screen.findByRole('dialog', { name: 'Comando rápido' });
    await user.type(within(dialog).getByRole('combobox'), 'nada-a-ver');
    expect(within(dialog).getByText('Nada encontrado')).toBeVisible();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog', { name: 'Comando rápido' })).toBeNull();
    expect(document.activeElement).toBe(opener);
  });

  it('runs a whole-fleet refresh from the palette', async () => {
    const { backend, user } = mount();
    await rows('data-tier');
    fireEvent.keyDown(window, { key: 'k', ctrlKey: true });
    const dialog = await screen.findByRole('dialog', { name: 'Comando rápido' });
    await user.type(within(dialog).getByRole('combobox'), 'atualizar todas');
    await user.keyboard('{Enter}');
    expect(backend.calls).toContain('refresh');
    expect(screen.queryByRole('dialog', { name: 'Comando rápido' })).toBeNull();
  });

  it('lists the command surface without opening the dialog', async () => {
    const { user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Comando/ }));
    expect(screen.getByRole('heading', { name: /Comando rápido/ })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Abrir comando rápido' })).toBeVisible();
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});

describe('guided first run', () => {
  it('asks for the key of a source that cannot read yet, then walks to the first reading', async () => {
    const { user } = mount('demo', { secrets: { kimi: false } });
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Primeiro uso/ }));
    // Kimi is on but has no key, so it is already picked and the wizard owes us a credential step.
    await user.click(screen.getByRole('button', { name: /^Continuar com/ }));
    await user.click(screen.getByRole('button', { name: 'Já conectei' }));
    await user.click(screen.getByRole('button', { name: /Ler agora/ }));
    await user.click(screen.getByRole('button', { name: 'Concluir' }));
    expect(screen.getByRole('button', { name: 'Ir para o painel' })).toBeVisible();
  });

  it('skips the credential step when every chosen source already has what it needs', async () => {
    const { user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Primeiro uso/ }));
    await user.click(screen.getByRole('button', { name: /^Continuar com/ }));
    expect(screen.getByRole('heading', { name: /O que cada fonte respondeu/ })).toBeVisible();
  });

  it('keeps the wizard step and the picked list when a source sends us to its settings', async () => {
    const { user } = mount('demo', {
      settings: (draft) => {
        draft.claude.shimConsent = false;
      },
    });
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Primeiro uso/ }));
    await user.click(screen.getByRole('button', { name: /^Continuar com/ }));
    await user.click(screen.getByRole('button', { name: 'Ver detalhes' }));
    expect(document.querySelector('#grp-claude')).not.toBeNull();
    await user.click(screen.getByRole('button', { name: 'Configurações' }));
    expect(screen.getByRole('button', { name: 'Já conectei' })).toBeVisible();
  });

  it('does not switch a source off just because it has no key yet', async () => {
    const { backend, user } = mount('demo', { secrets: { kimi: false } });
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Primeiro uso/ }));
    await user.click(screen.getByRole('button', { name: /^Continuar com/ }));
    expect((await backend.getSettings()).kimi.enabled).toBe(true);
  });
});

describe('the evidence list', () => {
  it('separates what is reachable from what the project must not read', async () => {
    const { user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Canais de cota/ }));
    expect(screen.getByRole('heading', { name: 'O que ainda dá para ler' })).toBeVisible();
    expect(screen.getByText('Créditos de uso (orçamento mensal)')).toBeVisible();
    expect(screen.getByText(/Orçamentos por escopo, com reset/)).toBeVisible();
    expect(screen.getByRole('heading', { name: /Existe, mas fora das regras/ })).toBeVisible();
    expect(screen.getAllByText(/precisa de entrada própria em docs\/data-sources.md/).length).toBe(
      2,
    );
  });
});

describe('side-by-side comparison', () => {
  it('puts the current interface next to the proposal on the same fabricated data', async () => {
    const { user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Comparar/ }));
    expect(screen.getByRole('region', { name: 'Interface atual' })).toBeVisible();
    expect(screen.getByRole('region', { name: 'Proposta' })).toBeVisible();
  });

  it('leaves the comparison when settings open, so no element id appears twice', async () => {
    const { user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Comparar/ }));
    // The second gear is the proposal pane's; the first belongs to the current interface.
    const gears = screen.getAllByRole('button', { name: 'Configurações' });
    const mine = gears[1];
    if (mine === undefined) throw new Error('the proposal pane has no settings toggle');
    await user.click(mine);
    expect(screen.queryByRole('region', { name: 'Proposta' })).toBeNull();
    const ids = [...document.querySelectorAll('[id]')].map((node) => node.id);
    expect(ids.length).toBe(new Set(ids).size);
  });

  it('keeps the proposal within its own ids so the two panes never share one', async () => {
    const { user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Comparar/ }));
    const ids = [...document.querySelectorAll('[id]')].map((node) => node.id);
    const duplicates = ids.filter((id, index) => ids.indexOf(id) !== index);
    expect(duplicates).toEqual([]);
    expect(ids.filter((id) => id.startsWith('nx-')).length).toBeGreaterThan(3);
  });
});
