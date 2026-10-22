// The command list is built from data and filtered by text, so both can be checked without
// rendering the palette.
import { describe, expect, it, vi } from 'vitest';
import type { ViewContext } from '../app/windowModel';
import { defaultSettings, type HarnessView } from '../lib/contract';
import { viewOf, windowOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';
import { NOW } from '../test/helpers';
import { commandsFor, filterCommands, type PaletteInput, step, summaryText } from './paletteModel';

const settings = structuredClone(defaultSettings);
const ctx: ViewContext = { nowSecs: NOW, settings };
const env = { now: NOW, settings, secrets: startFor('demo').secrets };

function reading(id: HarnessView['id'], usedPct: number, patch: Partial<HarnessView> = {}) {
  return viewOf(id, {
    windows: [
      windowOf(env, {
        id: 'five_hour',
        kind: 'five_hour',
        usedPct,
        windowMinutes: 300,
        resetsInMin: 60,
      }),
    ],
    ...patch,
  });
}

function input(views: readonly HarnessView[], overrides: Partial<PaletteInput> = {}): PaletteInput {
  return {
    views,
    ctx,
    select: vi.fn(),
    refresh: vi.fn(),
    openSettings: vi.fn(),
    copySummary: vi.fn(),
    scenario: vi.fn(),
    ...overrides,
  };
}

describe('command list', () => {
  it('offers one jump per monitored provider and skips the ones awaiting setup', () => {
    const commands = commandsFor(
      input([
        reading('claude', 48),
        reading('codex', 90),
        viewOf('kimi', { state: 'needs_setup' }),
        viewOf('zai', { state: 'disabled' }),
      ]),
    );
    const jumps = commands.filter((command) => command.group === 'providers');
    expect(jumps.map((command) => command.label)).toEqual(['Claude Code', 'Codex']);
    expect(jumps[1]?.hint).toContain('Esgota');
  });

  it('carries the actions, the settings entries and, only when asked, every scenario', () => {
    const commands = commandsFor(input([reading('codex', 90)]));
    const ids = commands.map((command) => command.id);
    expect(ids).toContain('refresh:all');
    expect(ids).toContain('copy:status');
    expect(ids).toContain('settings:general');
    expect(ids).toContain('settings:codex');
    expect(ids.filter((id) => id.startsWith('scenario:'))).toHaveLength(7);
    // The packaged app has no fabricated scenario to switch to.
    const shipped = commandsFor(input([reading('codex', 90)], { scenario: undefined }));
    expect(shipped.some((command) => command.id.startsWith('scenario:'))).toBe(false);
  });

  it('writes one line per provider into the copied summary', () => {
    const text = summaryText([reading('claude', 48), reading('codex', 90)], ctx);
    const lines = text.split('\n');
    expect(lines).toHaveLength(3);
    expect(lines[0]).toBe('AI Session Monitor');
    expect(lines[1]).toContain('Claude Code');
    expect(lines[2]).toContain('Codex');
    expect(text).not.toContain('undefined');
  });

  it('never carries a secret, a plan detail or a raw error into the copied text', () => {
    const planted = 'fab-token-123';
    const view = reading('kimi', 90, {
      plan: planted,
      meta: { alias: planted, host: `http://${planted}` },
      facts: [{ id: 'balance', number: 12, text: planted, unit: 'usd' }],
      error: { code: 'network', retryable: true, params: { reason: planted } },
      stale: true,
    });
    const text = summaryText([view], ctx);
    expect(text).not.toContain(planted);
    expect(text).toContain('Kimi Code');
  });
});

describe('command filtering', () => {
  // Built per test: the labels are written in the locale pinned before each test, not at import.
  const build = () => commandsFor(input([reading('codex', 90)]));

  it('keeps the built-in order for an empty query', () => {
    const commands = build();
    expect(filterCommands(commands, '   ')).toHaveLength(commands.length);
  });

  it('puts a label match before a keyword match and ignores accents and case', () => {
    const commands = build();
    const first = filterCommands(commands, 'cod')[0];
    expect(first?.label).toBe('Codex');
    const accented = filterCommands(commands, 'configuracoes');
    expect(accented[0]?.id).toBe('settings:general');
    const cased = filterCommands(commands, 'ATUALIZAR');
    expect(cased.map((command) => command.id)).toContain('refresh:all');
  });

  it('returns nothing when nothing matches', () => {
    const commands = build();
    expect(filterCommands(commands, 'zzzz')).toEqual([]);
  });

  it('wraps the highlight at both ends', () => {
    expect(step(0, -1, 3)).toBe(2);
    expect(step(2, 1, 3)).toBe(0);
    expect(step(0, 1, 0)).toBe(0);
  });
});
