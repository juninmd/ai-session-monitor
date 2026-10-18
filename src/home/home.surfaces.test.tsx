// The surfaces that are not provider lists: the command palette, the first-run wizard and the
// evidence list.
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HARNESS_IDS } from '../lib/contract';
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

  it('offers the fabricated scenarios in the browser dev loop', async () => {
    const { user } = mount();
    await rows('data-tier');
    fireEvent.keyDown(window, { key: 'k', ctrlKey: true });
    const dialog = await screen.findByRole('dialog', { name: 'Comando rápido' });
    await user.type(within(dialog).getByRole('combobox'), 'cenario');
    expect(within(dialog).getAllByRole('option').length).toBeGreaterThan(0);
  });
});

describe('guided first run', () => {
  const wizard = () => screen.findByRole('heading', { name: /O que você quer acompanhar\?/ });

  it('asks for the key of a source that cannot read yet, then walks to the first reading', async () => {
    const { user } = mount('fresh');
    await wizard();
    await user.click(screen.getByRole('button', { name: /^Continuar com/ }));
    await user.click(screen.getByRole('button', { name: 'Já conectei' }));
    await user.click(screen.getByRole('button', { name: /Ler agora/ }));
    await user.click(screen.getByRole('button', { name: 'Concluir' }));
    expect(screen.getByRole('button', { name: 'Ir para o painel' })).toBeVisible();
  });

  it('skips the credential step when every chosen source already has what it needs', async () => {
    // A switched-off source has nothing to ask for, so picking it cannot invent a credential step.
    const { user } = mount('fresh', {
      settings: (draft) => {
        for (const id of HARNESS_IDS) draft[id].enabled = false;
      },
    });
    await wizard();
    await user.click(screen.getByRole('button', { name: /Claude Code/ }));
    await user.click(screen.getByRole('button', { name: /^Continuar com 1/ }));
    expect(screen.getByRole('heading', { name: /O que cada fonte respondeu/ })).toBeVisible();
  });

  it('keeps the wizard step and the picked list when a source sends us to its settings', async () => {
    const { user } = mount('fresh');
    await wizard();
    await user.click(screen.getByRole('button', { name: /^Continuar com/ }));
    const card = screen.getByText('Claude Code').closest('.nx-connect') as HTMLElement;
    await user.click(within(card).getByRole('button', { name: 'Ver detalhes' }));
    expect(document.querySelector('#grp-claude')).not.toBeNull();
    await user.click(screen.getByRole('button', { name: 'Configurações' }));
    expect(screen.getByRole('button', { name: 'Já conectei' })).toBeVisible();
  });

  it('does not switch a source off just because it has no key yet', async () => {
    const { backend, user } = mount('fresh', { secrets: { kimi: false } });
    await wizard();
    await user.click(screen.getByRole('button', { name: /^Continuar com/ }));
    expect((await backend.getSettings()).kimi.enabled).toBe(true);
  });

  it('leaves the empty panel a way back to the wizard', async () => {
    const { user } = mount('fresh');
    await wizard();
    await user.click(screen.getByRole('button', { name: /^Continuar com/ }));
    await user.click(screen.getByRole('button', { name: 'Já conectei' }));
    await user.click(screen.getByRole('button', { name: 'Concluir' }));
    await user.click(screen.getByRole('button', { name: 'Ir para o painel' }));
    expect(await screen.findByText('Nenhuma fonte monitorada')).toBeVisible();
    await user.click(screen.getByRole('button', { name: 'Escolher fontes' }));
    expect(await wizard()).toBeVisible();
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
