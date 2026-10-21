// The two list surfaces, rendered on fabricated data: the urgency-ordered grid and the horizon.
import { screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import triageCss from './home-triage.css?raw';
import { detail, mount, openDetail, panel, RANK, rows } from './testHarness';

describe('triage surface', () => {
  it('lists every reachable provider in tier order and never puts a calm one first', async () => {
    mount();
    const list = await rows('data-tier');
    expect(list.length).toBeGreaterThan(3);
    const ranks = list.map((row) => RANK[row.getAttribute('data-tier') ?? 'unknown'] ?? 6);
    expect(ranks).toEqual([...ranks].sort((a, b) => a - b));
    expect(ranks[0]).toBeLessThanOrEqual(1);
  });

  it('fits several providers per row and several quotas per provider', async () => {
    mount();
    await rows('data-tier');
    // z.ai reports three windows; the card must show all three rather than one.
    const zai = [...document.querySelectorAll<HTMLElement>('button[data-tier]')].find((tile) =>
      tile.textContent?.includes('z.ai'),
    );
    if (zai === undefined) throw new Error('no z.ai tile');
    expect(zai.querySelectorAll('.nx-q')).toHaveLength(3);
  });

  it('lays the grid out for two providers per row, wide enough for the quotas to fit', () => {
    // jsdom has no layout, so the density claim is pinned against the stylesheet itself.
    expect(triageCss).toMatch(
      /\.nx-tiles\s*\{[^}]*grid-template-columns:\s*repeat\(auto-fill,\s*minmax\(10rem,\s*1fr\)\)/,
    );
  });

  it('keeps the renewal on the card and the projection in its own words', async () => {
    const { user } = mount();
    const list = await rows('data-tier');
    expect(screen.getAllByText(/sem cota/).length).toBeGreaterThan(0);
    // Each window row prints its own reading and renewal, so a card explains its own number.
    expect(list[0]?.textContent).toMatch(/%/);
    expect(list[0]?.textContent).toMatch(/\d+(h|min|d)/);
    // The detail opens beside the list, not stacked under it.
    await openDetail(user, 'Codex');
    expect(panel('Codex').textContent).toContain('renova em');
    const headline = document.querySelector('.nx-headline');
    expect(headline?.getAttribute('data-tone')).toBe('attention');
    expect(headline?.textContent).toMatch(/esgota|atenção/i);
  });

  it('draws the trend as one image with a description, projection included', async () => {
    const { user } = mount();
    await openDetail(user, 'Codex');
    await waitFor(() => expect(panel('Codex').querySelector('svg.nx-chart')).not.toBeNull());
    const chart = panel('Codex').querySelector('svg.nx-chart');
    expect(chart?.getAttribute('role')).toBe('img');
    expect(chart?.getAttribute('aria-labelledby')).toBeTruthy();
    expect(chart?.querySelector('.nx-chart-now')).not.toBeNull();
    expect(chart?.querySelector('.nx-chart-proj')).not.toBeNull();
  });

  it('opens the detail beside the list only when a provider is activated', async () => {
    const { user } = mount();
    await rows('data-tier');
    expect(document.querySelector('.nx-drawer')).toBeNull();
    await openDetail(user, 'Codex');
    expect(panel('Codex')).toBeVisible();
    // Escape leaves the drawer and puts focus back on the card that opened it.
    await user.keyboard('{Escape}');
    expect(document.querySelector('.nx-drawer')).toBeNull();
    expect(document.activeElement?.getAttribute('data-tier')).toBe('risk');
  });

  it('never opens the detail while the keyboard is only browsing the list', async () => {
    const { user } = mount();
    const list = await rows('data-tier');
    const pressed = () => list.findIndex((row) => row.getAttribute('aria-pressed') === 'true');
    const start = pressed();
    list[start]?.focus();
    await user.keyboard('{ArrowDown}{ArrowDown}');
    expect(document.querySelector('.nx-drawer')).toBeNull();
    expect(document.activeElement).toBe(list[(start + 2) % list.length]);
  });

  it('never prints an old reading as a current one, in the card or the detail', async () => {
    const { user } = mount('stale');
    await openDetail(user, 'Codex');
    const detail = panel('Codex');
    expect(detail.textContent).toContain('Percentual desconhecido');
    expect(detail.textContent).not.toContain('No ritmo atual');
    expect(detail.querySelectorAll('.gauge-void').length).toBeGreaterThan(0);
    expect(detail.querySelector('.nx-chart-now')).toBeNull();
    // A stale reading has no live number, so it must not print an amount either — on the card or
    // in the detail.
    expect(detail.textContent).not.toMatch(/de US\$|gastos, sem teto/);
    const litellm = [...document.querySelectorAll<HTMLElement>('button[data-tier]')].find((tile) =>
      tile.textContent?.includes('LiteLLM'),
    );
    if (litellm !== undefined) expect(litellm.textContent).not.toMatch(/US\$/);
  });

  it('gives a reading at the threshold its own words, not only a colour', async () => {
    mount();
    await rows('data-tier');
    const notes = [...document.querySelectorAll<HTMLElement>('.nx-tile-note')];
    expect(notes.length).toBeGreaterThan(0);
    expect(notes.every((note) => (note.textContent ?? '').trim() !== '')).toBe(true);
  });

  it('shows the per-model weekly caps a source reports, Fable included', async () => {
    const { user } = mount('demo', {
      settings: (draft) => {
        draft.claude.oauthUsage = true;
      },
    });
    await rows('data-tier');
    const claude = [...document.querySelectorAll<HTMLElement>('button[data-tier]')].find((tile) =>
      tile.textContent?.includes('Claude Code'),
    );
    if (claude === undefined) throw new Error('no Claude tile');
    await user.click(claude);
    const detail = panel('Claude Code');
    expect(detail.textContent).toContain('Fable');
    expect(detail.textContent).toContain('Opus');
    expect(detail.textContent).toContain('Sonnet');
    // Each model cap is its own section of the sheet, not folded into the all-models windows.
    expect(detail.querySelectorAll('.nx-sheet-group').length).toBeGreaterThanOrEqual(4);
  });

  it('gives a pool that keeps its own allowance its own section and explanation', async () => {
    const { user } = mount();
    await rows('data-tier');
    const antigravity = [...document.querySelectorAll<HTMLElement>('button[data-tier]')].find(
      (tile) => tile.textContent?.includes('Antigravity'),
    );
    if (antigravity === undefined) throw new Error('no Antigravity tile');
    await user.click(antigravity);
    const detail = panel('Antigravity');
    const pools = [...detail.querySelectorAll('.nx-sheet-pool')].map((node) => node.textContent);
    expect(pools.some((text) => text?.includes('Argon'))).toBe(true);
    // The Argon allowance is weekly-only and separate, and the sheet says so.
    expect(detail.textContent).toContain('Cota semanal própria');
    expect(pools.some((text) => text?.includes('Gemini'))).toBe(true);
  });

  it('keeps the detail heading order, never skipping a level', async () => {
    const { user } = mount();
    await openDetail(user, 'Codex');
    const levels = [...detail().querySelectorAll('h1,h2,h3,h4,h5,h6')].map((heading) =>
      Number(heading.tagName.slice(1)),
    );
    expect(levels[0]).toBe(2);
    for (let index = 1; index < levels.length; index += 1) {
      expect(levels[index]).toBeLessThanOrEqual((levels[index - 1] ?? 0) + 1);
    }
  });

  it('does not ask for history when the user turned it off', async () => {
    const { backend, user } = mount('demo', {
      settings: (draft) => {
        draft.general.historyEnabled = false;
      },
    });
    await openDetail(user, 'Codex');
    expect(backend.calls).not.toContain('get_history');
    expect(panel('Codex').textContent).not.toContain('Histórico');
  });

  it('copies the stored readings as CSV from an icon in the history header', async () => {
    const { user } = mount();
    await openDetail(user, 'Codex');
    await user.click(await screen.findByRole('button', { name: 'Copiar leituras em CSV' }));
    expect(await screen.findByText('Leituras copiadas em CSV.')).toBeInTheDocument();
    const copied = await navigator.clipboard.readText();
    expect(copied).toMatch(/^harness,window,resets_at,read_at,used_pct/);
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
    expect(panel('Codex').textContent).toContain('Histórico');
    expect(panel('Codex').textContent).toContain('Informações adicionais');
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
    expect(screen.getAllByText('lendo').length).toBeGreaterThan(0);
  });

  it('invites the first run when nothing is being monitored', async () => {
    mount('fresh');
    // Nothing readable yet, so the wizard replaces the surfaces: it is not a navigation item.
    expect(
      await screen.findByRole('heading', { name: /O que você quer acompanhar\?/ }),
    ).toBeVisible();
    expect(screen.getByRole('button', { name: /^Continuar com/ })).toBeVisible();
    expect(screen.queryByRole('button', { name: /^Triagem/ })).toBeNull();
  });

  it('renders the setup surface for a provider that cannot be read yet', async () => {
    const { user } = mount('errors');
    await openDetail(user, 'Codex');
    expect(panel('Codex')).toBeVisible();
    expect(detail().querySelectorAll('h1,h2,h3,h4,h5,h6').length).toBeGreaterThan(0);
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

  it('only asks every provider for its history once the trend is switched on', async () => {
    const { backend, user } = mount();
    await rows('data-tier');
    // The open detail panel always charts its one provider; the grid must not add to that.
    const afterPanel = backend.calls.filter((call) => call === 'get_history').length;
    expect(afterPanel).toBeLessThanOrEqual(1);
    await user.click(screen.getByRole('button', { name: /^Horizonte/ }));
    await rows('data-kind');
    expect(backend.calls.filter((call) => call === 'get_history')).toHaveLength(afterPanel);
    await user.click(screen.getByRole('checkbox', { name: /Mostrar tendência/ }));
    await waitFor(() =>
      expect(backend.calls.filter((call) => call === 'get_history').length).toBeGreaterThan(
        afterPanel,
      ),
    );
  });
});

describe('density setting', () => {
  it('marks the flyout compact only when the setting is on', async () => {
    mount('demo', {
      settings: (draft) => {
        draft.general.compact = true;
      },
    });
    await rows('data-tier');
    expect(document.querySelector('.nx-fly')).toHaveAttribute('data-density', 'compact');
  });

  it('keeps the comfortable spacing by default', async () => {
    mount();
    await rows('data-tier');
    expect(document.querySelector('.nx-fly')).not.toHaveAttribute('data-density');
  });
});
