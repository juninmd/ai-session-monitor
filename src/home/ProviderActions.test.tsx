import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { ViewContext } from '../app/windowModel';
import { createMockBackend } from '../mock';
import { fact, viewOf } from '../mock/fixtures';
import { ctxOf, freezeClock, NOW, win } from '../test/helpers';
import { PreviewActionsContext, resetCreditCount, simulateCodexReset } from './actionModel';
import { ProviderActions } from './ProviderActions';

const ctx: ViewContext = ctxOf();
const codex = (count: number | null = 2) =>
  viewOf('codex', {
    source: { id: 'app_server', official: true },
    windows: [win()],
    facts: [{ ...fact('reset_credits', 2, 'resets'), number: count }],
  });

describe('preview provider actions', () => {
  it('only enables reset with a known whole count and current consented app-server data', () => {
    expect(resetCreditCount(codex(), ctx)).toBe(2);
    expect(resetCreditCount(codex(0), ctx)).toBe(0);
    for (const count of [null, -1, 1.5, Number.NaN, Number.POSITIVE_INFINITY, 1_000_001]) {
      expect(resetCreditCount(codex(count), ctx)).toBeNull();
    }
    for (const patch of [
      { stale: true },
      { facts: [] },
      { source: { id: 'rollout', official: false } },
    ])
      expect(resetCreditCount({ ...codex(), ...patch }, ctx)).toBeNull();
    expect(
      resetCreditCount(codex(), {
        ...ctx,
        settings: { ...ctx.settings, codex: { ...ctx.settings.codex, consent: false } },
      }),
    ).toBeNull();
    expect(
      resetCreditCount(codex(), {
        ...ctx,
        settings: { ...ctx.settings, codex: { ...ctx.settings.codex, enabled: false } },
      }),
    ).toBeNull();
  });

  it('simulates one credit consumption while preserving unknown and expired quota readings', async () => {
    const backend = createMockBackend({ scenario: 'demo', now: () => NOW });
    const view = {
      ...codex(),
      windows: [
        win(),
        win({ id: 'unknown', usedPct: null }),
        { ...win({ id: 'expired', usedPct: 95 }), resetsAt: NOW },
      ],
    };
    backend.override(view);
    expect(await simulateCodexReset(backend, NOW)).toBe('reset');
    const result = (await backend.getSnapshot()).views.find(
      (candidate) => candidate.id === 'codex',
    );
    expect(result?.facts.find((entry) => entry.id === 'reset_credits')?.number).toBe(1);
    expect(result?.windows.map((window) => window.usedPct)).toEqual([0, null, 95]);
    expect(result?.windows[0]?.resetsAt).toBe(NOW + 300 * 60);
    expect(await simulateCodexReset(backend, NOW)).toBe('nothingToReset');
    expect(
      (await backend.getSnapshot()).views
        .find((candidate) => candidate.id === 'codex')
        ?.facts.find((entry) => entry.id === 'reset_credits')?.number,
    ).toBe(1);
  });

  it('rechecks the latest balance and does not mutate missing or stale capabilities', async () => {
    const backend = createMockBackend({ scenario: 'demo', now: () => NOW });
    backend.override(codex(0));
    expect(await simulateCodexReset(backend, NOW)).toBe('noCredit');
    backend.override({ ...codex(), stale: true });
    expect(await simulateCodexReset(backend, NOW)).toBe('unavailable');
    expect(
      (await backend.getSnapshot()).views.find((view) => view.id === 'codex')?.windows[0]?.usedPct,
    ).toBe(48);
  });

  it('requires confirmation, supports cancellation and shows the reset outcome', async () => {
    const user = userEvent.setup();
    const consumeReset = vi.fn(async () => 'reset' as const);
    render(
      <PreviewActionsContext value={{ lowPriorityOffered: false, consumeReset }}>
        <ProviderActions view={codex()} ctx={ctx} />
      </PreviewActionsContext>,
    );
    await user.click(screen.getByRole('button', { name: 'Usar reset' }));
    expect(consumeReset).not.toHaveBeenCalled();
    expect(screen.getByRole('group', { name: 'Confirmar reset de cota' })).toHaveTextContent(
      'consumo é definitivo',
    );
    await user.click(screen.getByRole('button', { name: 'Cancelar' }));
    expect(screen.queryByRole('group')).toBeNull();
    expect(screen.getByRole('button', { name: 'Usar reset' })).toHaveFocus();
    expect(consumeReset).not.toHaveBeenCalled();
    await user.click(screen.getByRole('button', { name: 'Usar reset' }));
    await user.click(screen.getByRole('button', { name: 'Confirmar reset' }));
    expect(consumeReset).toHaveBeenCalledTimes(1);
    expect(await screen.findByRole('status')).toHaveTextContent('Reset simulado');
    expect(screen.getByRole('status')).toHaveFocus();
  });

  it('only exposes copying when an offer exists, without claiming it activated the mode', async () => {
    const user = userEvent.setup();
    const write = vi.spyOn(navigator.clipboard, 'writeText');
    const view = viewOf('claude');
    const consumeReset = vi.fn(async () => 'unavailable' as const);
    const ui = (offer: boolean) => (
      <PreviewActionsContext value={{ lowPriorityOffered: offer, consumeReset }}>
        <ProviderActions view={view} ctx={ctx} />
      </PreviewActionsContext>
    );
    const { rerender } = render(ui(false));
    expect(screen.queryByRole('button', { name: 'Copiar /low-priority' })).toBeNull();
    rerender(ui(true));
    await user.click(screen.getByRole('button', { name: 'Copiar /low-priority' }));
    expect(write).toHaveBeenCalledWith('/low-priority');
    expect(screen.getByRole('status')).toHaveTextContent(
      'ainda precisa ser ativado no Claude Code',
    );
    expect(consumeReset).not.toHaveBeenCalled();
  });

  it('renders nothing without the actions the mocked preview provides', () => {
    freezeClock();
    const { container } = render(<ProviderActions view={codex()} ctx={ctx} />);
    expect(container).toBeEmptyDOMElement();
  });
});
