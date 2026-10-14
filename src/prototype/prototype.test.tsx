import { act, render, renderHook, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { App } from '../App';
import type { ViewContext } from '../app/windowModel';
import { t } from '../i18n';
import { defaultSettings, type HarnessView } from '../lib/contract';
import { createMockBackend } from '../mock';
import { viewOf, windowOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';
import { freezeClock, NOW } from '../test/helpers';
import { activeViews, criticalViews, usageChip } from './model';
import { ProviderOverview } from './ProviderOverview';
import { useCriticalToast } from './useCriticalToast';

const ctx: ViewContext = { nowSecs: NOW, settings: structuredClone(defaultSettings) };
const env = { now: NOW, settings: ctx.settings, secrets: startFor('demo').secrets };
const reading = (usedPct: number | null, patch: Partial<HarnessView> = {}) =>
  viewOf('codex', {
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

describe('prototype source visibility and critical usage', () => {
  it('hides disabled and unconfigured sources, keeping temporary failures and stale data', () => {
    const views = [
      'ready',
      'loading',
      'error',
      'no_data',
      'disabled',
      'needs_setup',
      'needs_consent',
    ].map((state) => reading(40, { state: state as HarnessView['state'] }));
    expect(activeViews(views, ctx).map((view) => view.state)).toEqual([
      'ready',
      'loading',
      'error',
      'no_data',
    ]);
    const disabled = {
      ...ctx,
      settings: { ...ctx.settings, codex: { ...ctx.settings.codex, enabled: false } },
    };
    expect(activeViews([reading(95)], disabled)).toHaveLength(0);
    expect(usageChip(reading(95), disabled).kind).toBe('text');
  });
  it('marks exactly 90% and skips unknown, expired and stale readings', () => {
    expect(criticalViews([reading(89.99)], ctx)).toHaveLength(0);
    expect(criticalViews([reading(90)], ctx)).toHaveLength(1);
    expect(
      criticalViews(
        [
          reading(null),
          reading(95, { stale: true }),
          reading(95, {
            windows: reading(95).windows.map((window) => ({ ...window, resetsAt: NOW })),
          }),
        ],
        ctx,
      ),
    ).toHaveLength(0);
  });
});

describe('preview crossing notification', () => {
  const alerts = {
    ...ctx,
    settings: { ...ctx.settings, general: { ...ctx.settings.general, alertThresholds: [90] } },
  };
  it('notifies once at 90%, even after a repeated read or a stale interruption', () => {
    const { result, rerender } = renderHook(({ view }) => useCriticalToast([view], alerts), {
      initialProps: { view: reading(89.99) },
    });
    expect(result.current.toasts).toHaveLength(0);
    rerender({ view: reading(90) });
    expect(result.current.toasts).toHaveLength(1);
    rerender({ view: reading(95, { stale: true }) });
    rerender({ view: reading(95) });
    expect(result.current.toasts).toHaveLength(1);
    const notice = result.current.toasts[0];
    if (!notice) throw new Error('expected a crossing notification');
    act(() => result.current.dismiss(notice.key));
    rerender({ view: reading(96) });
    expect(result.current.toasts).toHaveLength(0);
  });
  it('does not toast an initial critical reading or when threshold alerts are disabled', () => {
    const { result } = renderHook(() => useCriticalToast([reading(95)], alerts));
    expect(result.current.toasts).toHaveLength(0);
    const off = {
      ...alerts,
      settings: {
        ...alerts.settings,
        general: { ...alerts.settings.general, alertThresholds: [] },
      },
    };
    const disabled = renderHook(({ view }) => useCriticalToast([view], off), {
      initialProps: { view: reading(89) },
    });
    disabled.rerender({ view: reading(90) });
    expect(disabled.result.current.toasts).toHaveLength(0);
  });
  it('arms a new reset epoch after observing a reading below 90%', () => {
    const { result, rerender } = renderHook(({ view }) => useCriticalToast([view], alerts), {
      initialProps: { view: reading(89) },
    });
    rerender({ view: reading(90) });
    const nextEpoch = (used: number) =>
      reading(used, {
        windows: reading(used).windows.map((window) => ({ ...window, resetsAt: NOW + 7200 })),
      });
    rerender({ view: nextEpoch(10) });
    rerender({ view: nextEpoch(90) });
    expect(result.current.toasts).toHaveLength(2);
  });
});

describe('preview interactions', () => {
  it('moves the searchable overview into settings and opens an inactive provider configuration', async () => {
    freezeClock();
    const start = startFor('demo');
    start.settings.litellm.enabled = false;
    const backend = createMockBackend({ ...start, now: () => NOW });
    const user = userEvent.setup();
    render(<App backend={backend} preview />);
    await screen.findByRole('tab', { name: /^Claude Code/ });
    expect(screen.getAllByRole('tab')).toHaveLength(13);
    expect(screen.queryByText(t('providers.title'))).toBeNull();
    await user.click(screen.getByRole('button', { name: /^Configurações$/ }));
    await user.click(screen.getByText(t('providers.title')));
    await user.type(screen.getByRole('searchbox'), 'LiteLLM');
    await user.click(screen.getByRole('button', { name: /^LiteLLM/ }));
    expect(document.getElementById('grp-litellm')).toHaveAttribute('open');
    expect(document.querySelector('#grp-litellm summary')).toHaveFocus();
  });
  it('shows all statuses on hover, stays readable over the tooltip and closes on Escape', async () => {
    const user = userEvent.setup();
    render(
      <ProviderOverview
        views={[reading(90), viewOf('litellm', { state: 'disabled' })]}
        ctx={ctx}
      />,
    );
    const trigger = screen.getByRole('button', { name: 'Resumo de todos os providers' });
    await user.hover(trigger);
    const tooltip = screen.getByRole('tooltip');
    expect(within(tooltip).getByText(/90% usado/)).toBeVisible();
    expect(within(tooltip).getByText('LiteLLM')).toBeVisible();
    await user.hover(tooltip);
    expect(screen.getByRole('tooltip')).toBeVisible();
    trigger.focus();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('tooltip')).toBeNull();
  });
  it('falls back to an active tab when the selected provider is disabled', async () => {
    freezeClock();
    const start = startFor('demo');
    const backend = createMockBackend({ settings: start.settings, now: () => NOW });
    render(<App backend={backend} preview />);
    await screen.findByRole('tab', { name: /^Claude Code/ });
    await act(async () =>
      backend.saveSettings({
        ...start.settings,
        claude: { ...start.settings.claude, enabled: false },
      }),
    );
    expect(screen.queryByRole('tab', { name: /^Claude Code/ })).toBeNull();
    expect(screen.getByRole('tabpanel')).toHaveAccessibleName('Codex');
  });
});
