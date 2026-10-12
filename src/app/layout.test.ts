import { describe, expect, it } from 'vitest';
import type { HarnessView } from '../lib/contract';
import { antigravityView, claudeView, codexView, litellmView } from '../mock/builders';
import { antigravityEmitted, claudeEmitted, codexEmitted, litellmEmitted } from '../mock/emitted';
import { viewOf } from '../mock/fixtures';
import { ctxOf, envOf, settingsWith, win } from '../test/helpers';
import { groupName, layoutWindows } from './layout';

const ctx = ctxOf();

const withOauth = envOf({
  settings: settingsWith((s) => {
    s.claude.oauthUsage = true;
  }),
});

const ids = (models: { window: { id: string } }[]) => models.map((m) => m.window.id);

describe('layoutWindows', () => {
  it('makes the first ungrouped window the hero and the rest flat sections', () => {
    const layout = layoutWindows(codexView(envOf(), 'ok'), ctx);
    expect(layout.hero?.window.id).toBe('five_hour');
    expect(ids(layout.flats)).toEqual(['weekly']);
    expect(layout.pools).toEqual([]);
    expect(layout.groupList).toBeNull();
  });

  it('keeps per-model windows of a single-window group in one list', () => {
    const layout = layoutWindows(claudeView(withOauth, 'ok'), ctx);
    expect(layout.hero?.window.id).toBe('five_hour');
    expect(ids(layout.flats)).toEqual(['weekly']);
    expect(layout.groupList?.title).toBe('Semana por modelo');
    expect(layout.groupList?.rows.map((row) => row.name)).toEqual(['Opus', 'Sonnet', 'Fable']);
    expect(layout.pools).toEqual([]);
  });

  it('shows no per-model list while the usage API is off', () => {
    const layout = layoutWindows(claudeView(envOf(), 'ok'), ctx);
    expect(layout.groupList).toBeNull();
    expect(ids(layout.flats)).toEqual(['weekly']);
  });

  it('turns multi-window groups into pools with their own note', () => {
    const layout = layoutWindows(antigravityView(envOf(), 'ok'), ctx);
    expect(layout.hero).toBeNull();
    expect(layout.flats).toEqual([]);
    expect(layout.groupList).toBeNull();
    expect(layout.pools.map((pool) => pool.title)).toEqual(['Gemini', 'Claude e GPT-OSS']);
    expect(layout.pools.map((pool) => pool.note)).toEqual([
      'Pro e Flash dividem a mesma cota.',
      'Cota separada da do Gemini.',
    ]);
    expect(layout.pools[0]?.rows.map((row) => row.name)).toEqual(['5 horas', 'Semana']);
  });

  it('keeps unknown pool windows in their pool instead of dropping them', () => {
    const layout = layoutWindows(antigravityView(envOf(), 'ok', true), ctx);
    const pool = layout.pools.find((candidate) => candidate.id === 'claude_gpt');
    expect(pool?.rows).toHaveLength(2);
    expect(pool?.rows.every((row) => row.model.available === null)).toBe(true);
  });

  it('uses the budget as the hero of a LiteLLM view', () => {
    const layout = layoutWindows(litellmView(envOf(), 'ok'), ctx);
    expect(layout.hero?.window.id).toBe('budget');
    expect(layout.hero?.window.amount?.unit).toBe('usd');
  });

  it('lays out nothing for a view without windows', () => {
    const layout = layoutWindows(viewOf('codex'), ctx);
    expect(layout).toEqual({ hero: null, flats: [], groupList: null, pools: [] });
  });

  it('titles a mixed per-model list generically', () => {
    const view: HarnessView = viewOf('opencode', {
      windows: [
        { ...win({ id: 'a', kind: 'weekly' }), group: 'alpha' },
        { ...win({ id: 'b', kind: 'five_hour' }), group: 'beta' },
      ],
    });
    expect(layoutWindows(view, ctx).groupList?.title).toBe('Por modelo');
  });

  it('prefers the label the source gave a window over the group name', () => {
    const view: HarnessView = viewOf('opencode', {
      windows: [{ ...win({ id: 'a', kind: 'weekly' }), group: 'alpha', label: 'Modelo Alfa' }],
    });
    expect(layoutWindows(view, ctx).groupList?.rows[0]?.name).toBe('Modelo Alfa');
  });

  it('does not mix a multi-window group into the per-model list', () => {
    const view: HarnessView = viewOf('opencode', {
      windows: [
        { ...win({ id: 'a1', kind: 'five_hour' }), group: 'alpha' },
        { ...win({ id: 'a2', kind: 'weekly' }), group: 'alpha' },
        { ...win({ id: 'b1', kind: 'weekly' }), group: 'beta' },
      ],
    });
    const layout = layoutWindows(view, ctx);
    expect(layout.groupList).toBeNull();
    expect(layout.pools.map((pool) => pool.id)).toEqual(['alpha', 'beta']);
  });
});

describe('groupName', () => {
  it.each([
    ['opus', 'Opus'],
    ['claude_gpt', 'Claude e GPT-OSS'],
    ['brand_new_pool', 'Brand new pool'],
    ['x', 'X'],
    // Antigravity numbers a group the server did not name from 0; people count from 1.
    ['group_0', 'Grupo 1'],
    ['group_2', 'Grupo 3'],
    ['group_15', 'Grupo 16'],
  ])('%s reads "%s"', (group, expected) => {
    expect(groupName(group)).toBe(expected);
  });

  it.each(['group_', 'group_x', 'group_1000', 'group_-1', 'group_1_2'])(
    'does not number %j, which is not a plain group index',
    (group) => {
      expect(groupName(group)).not.toMatch(/^Grupo \d/);
    },
  );
});

describe('windows as the collectors emit them', () => {
  it('lists a LiteLLM view: the headline first, concurrent caps flat, models in one list', () => {
    const layout = layoutWindows(litellmEmitted(envOf()), ctx);
    expect(layout.hero?.title).toBe('Orçamento: squad-alfa');
    expect(layout.flats.map((m) => m.title)).toEqual([
      'Orçamento de 1 hora',
      'Orçamento de 30 dias',
    ]);
    expect(layout.groupList?.title).toBe('Orçamento por modelo');
    expect(layout.groupList?.rows.map((row) => row.name)).toEqual(['modelo-a', 'modelo-b']);
    expect(layout.pools).toEqual([]);
  });

  it('titles Codex windows of an unusual length apart', () => {
    const layout = layoutWindows(codexEmitted(envOf()), ctx);
    expect([layout.hero, ...layout.flats].map((m) => m?.title)).toEqual([
      'Janela de 2 dias',
      'Janela de 14 dias',
    ]);
  });

  it('puts Claude per-model weeks, a provider and the spend cap where they belong', () => {
    const layout = layoutWindows(claudeEmitted(envOf()), ctx);
    expect(layout.hero?.title).toBe('Sessão de 5 horas');
    expect(layout.flats.map((m) => m.title)).toEqual([
      'Semana, todos os modelos',
      'Limite de gasto',
    ]);
    expect(layout.groupList?.title).toBe('Semana por modelo');
    expect(layout.groupList?.rows.map((row) => row.name)).toEqual([
      'Opus',
      'Sonnet',
      'Fable',
      'Claude Design',
    ]);
  });

  it('names an unnamed Antigravity group and keeps a cadence-less window under its label', () => {
    const layout = layoutWindows(antigravityEmitted(envOf()), ctx);
    expect(layout.pools.map((pool) => pool.title)).toEqual(['Gemini', 'Grupo 3']);
    const unnamed = layout.pools.find((pool) => pool.id === 'group_2');
    expect(unnamed?.rows.map((row) => row.name)).toEqual(['5 horas', 'Cota mensal extra']);
  });
});
