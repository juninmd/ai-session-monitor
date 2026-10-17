// The proposed surfaces, rendered on fabricated data. These assertions are about behaviour the
// current interface does not have: urgency order, a headline, and a keyboard-first command list.
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import type { Settings } from '../../lib/contract';
import { createMockBackend } from '../../mock';
import { type ScenarioName, startFor } from '../../mock/scenarios';
import { freezeClock, NOW } from '../../test/helpers';
import { NextStage } from './NextStage';

const RANK: Record<string, number> = {
  blocked: 0,
  risk: 1,
  broken: 2,
  watch: 3,
  loading: 4,
  fine: 5,
  unknown: 6,
};

const OPTIONS = { now: () => NOW, latencyMs: 0 };

interface MountOptions {
  secrets?: Record<string, boolean>;
  settings?: (draft: Settings) => void;
}

function mount(scenario: ScenarioName = 'demo', options: MountOptions = {}) {
  freezeClock();
  const start = startFor(scenario);
  const settings = structuredClone(start.settings);
  options.settings?.(settings);
  const backend = createMockBackend({
    scenario,
    settings,
    secrets: { ...start.secrets, ...options.secrets },
    ...OPTIONS,
  });
  const user = userEvent.setup();
  render(<NextStage backend={backend} />);
  return { backend, user };
}

/** The rows of a list surface: buttons, marked by the tier or kind they carry. */
async function rows(attribute: 'data-tier' | 'data-kind'): Promise<HTMLElement[]> {
  await waitFor(() => expect(document.querySelector(`button[${attribute}]`)).not.toBeNull());
  return [...document.querySelectorAll<HTMLElement>(`button[${attribute}]`)];
}

function panel(name: string) {
  return screen.getByRole('region', { name });
}

describe('triage surface', () => {
  it('lists every reachable provider in tier order and never puts a calm one first', async () => {
    mount();
    const list = await rows('data-tier');
    expect(list.length).toBeGreaterThan(3);
    const ranks = list.map((row) => RANK[row.getAttribute('data-tier') ?? 'unknown'] ?? 6);
    expect(ranks).toEqual([...ranks].sort((a, b) => a - b));
    expect(ranks[0]).toBeLessThanOrEqual(1);
  });

  it('shows the projection, the renewal and the headline together', async () => {
    mount();
    await rows('data-tier');
    expect(screen.getAllByText(/Esgota às/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/renova em /).length).toBeGreaterThan(0);
    const headline = document.querySelector('.nx-headline');
    expect(headline?.getAttribute('data-tone')).toBe('attention');
    expect(headline?.textContent).toMatch(/esgota|atenção/i);
  });

  it('offers a jump to every provider the headline names', async () => {
    mount();
    await rows('data-tier');
    const jump = document.querySelectorAll('.nx-headline-jump button');
    expect(jump.length).toBeGreaterThan(0);
    expect(jump[0]?.textContent).toMatch(/^Ver /);
  });

  it('opens the selected provider in the shared detail panel', async () => {
    const { user } = mount();
    const list = await rows('data-tier');
    const target = list.find((row) => row.textContent?.includes('Codex'));
    if (target === undefined) throw new Error('no Codex row');
    await user.click(target);
    expect(panel('Codex')).toBeVisible();
  });

  it('moves the selection with the arrow keys and keeps one tab stop', async () => {
    const { user } = mount();
    const list = await rows('data-tier');
    const pressed = () => list.findIndex((row) => row.getAttribute('aria-pressed') === 'true');
    const start = pressed();
    expect(start).toBeGreaterThanOrEqual(0);
    list[start]?.focus();
    await user.keyboard('{ArrowDown}');
    expect(pressed()).toBe((start + 1) % list.length);
    expect(document.activeElement).toBe(list[(start + 1) % list.length]);
    expect(list.filter((row) => row.getAttribute('tabindex') === '0')).toHaveLength(1);
    await user.keyboard('{End}');
    expect(document.activeElement).toBe(list.at(-1));
    await user.keyboard('{Home}');
    expect(document.activeElement).toBe(list[0]);
  });

  it('says it is reading instead of reporting a gap on the first snapshot', async () => {
    mount('loading');
    const list = await rows('data-tier');
    expect(list.every((row) => row.getAttribute('data-tier') === 'loading')).toBe(true);
    expect(document.querySelector('.nx-headline-text')?.textContent).toMatch(/^Lendo /);
    expect(screen.getAllByText('Lendo…').length).toBeGreaterThan(0);
  });

  it('invites the first run when nothing is being monitored', async () => {
    const { user } = mount('fresh');
    await screen.findByText('Nenhuma fonte monitorada');
    await user.click(screen.getByRole('button', { name: 'Escolher fontes' }));
    expect(screen.getByRole('button', { name: /^Continuar com/ })).toBeVisible();
  });

  it('renders the setup surface for a provider that cannot be read yet', async () => {
    mount('errors');
    await rows('data-tier');
    expect(panel('Codex')).toBeVisible();
    expect(screen.getAllByRole('heading').length).toBeGreaterThan(1);
  });
});

describe('horizon surface', () => {
  it('explains the bar and refuses to project unknown readings', async () => {
    const { user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Horizonte/ }));
    expect(screen.getByRole('heading', { name: 'Chego até a renovação?' })).toBeVisible();
    const list = await rows('data-kind');
    expect(list.length).toBeGreaterThan(3);
    expect(screen.getAllByText(/sem cota/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/esgota às|já esgotado/i).length).toBeGreaterThan(0);
  });

  it('only asks for history once the trend is switched on', async () => {
    const { backend, user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Horizonte/ }));
    await rows('data-kind');
    expect(backend.calls).not.toContain('get_history');
    await user.click(screen.getByRole('checkbox', { name: /Mostrar tendência/ }));
    await waitFor(() => expect(backend.calls).toContain('get_history'));
  });
});

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

describe('guided first run and the evidence list', () => {
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

  it('lists the quota channels that are reachable and the ones that are not', async () => {
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
    const namespaced = [...document.querySelectorAll('[id]')].filter((node) =>
      node.id.startsWith('nx-'),
    );
    expect(namespaced.length).toBeGreaterThan(3);
    const ids = [...document.querySelectorAll('[id]')].map((node) => node.id);
    expect(ids.length).toBe(new Set(ids).size);
  });
});

/** The rows of a list surface: buttons, marked by the tier or kind they carry. */
