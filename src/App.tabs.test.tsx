import { act, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { renderApp, settingsWith } from './test/helpers';

describe('visible provider tabs', () => {
  it('loads the stored selection and labels the fallback panel with its visible tab', async () => {
    renderApp({
      settings: settingsWith((s) => {
        s.general.visibleTabs = ['codex', 'litellm'];
      }),
    });
    expect(await screen.findByRole('tabpanel', { name: 'Codex' })).toBeVisible();
    expect(screen.getAllByRole('tab').map((tab) => tab.id)).toEqual(['tab-codex', 'tab-litellm']);
    expect(screen.queryByRole('tab', { name: /^Claude/ })).toBeNull();
  });

  it('skips hidden providers with arrows, wrapping, Home and End', async () => {
    const { user } = renderApp({
      settings: settingsWith((s) => {
        s.general.visibleTabs = ['claude', 'deepseek', 'litellm'];
      }),
    });
    const claude = await screen.findByRole('tab', { name: /^Claude Code 52%/ });
    claude.focus();
    const keys = [
      ['{ArrowRight}', 'deepseek'],
      ['{ArrowRight}', 'litellm'],
      ['{ArrowRight}', 'claude'],
      ['{ArrowLeft}', 'litellm'],
      ['{Home}', 'claude'],
      ['{End}', 'litellm'],
    ] as const;
    for (const [key, id] of keys) {
      await user.keyboard(key);
      expect(document.getElementById(`tab-${id}`)).toHaveFocus();
      expect(document.getElementById(`tab-${id}`)).toHaveAttribute('aria-selected', 'true');
    }
  });

  it('hides the active provider, keeps its setup accessible and returns focus to a visible tab', async () => {
    const { user, backend } = renderApp();
    await screen.findByRole('heading', { name: 'Sessão de 5 horas' });
    await user.click(screen.getByRole('button', { name: 'Personalizar abas' }));
    expect(screen.getByRole('heading', { name: 'Abas visíveis' })).toHaveFocus();
    await user.click(screen.getByRole('checkbox', { name: 'Claude Code' }));
    expect(screen.getByRole('checkbox', { name: 'Claude Code' })).not.toBeChecked();
    expect(document.getElementById('grp-claude')).toBeInTheDocument();
    expect((await backend.getSettings()).claude.enabled).toBe(true);
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(screen.getByRole('tabpanel')).toHaveAccessibleName('Codex');
    expect(screen.getByRole('tab', { name: /^Codex/ })).toHaveFocus();
  });

  it('recovers from hiding every provider, including Escape and flyout shown focus', async () => {
    const { user, backend } = renderApp({
      settings: settingsWith((s) => {
        s.general.visibleTabs = [];
      }),
    });
    await screen.findByRole('heading', { name: 'Seu monitor, do seu jeito' });
    expect(screen.queryByRole('tablist')).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Escolher providers' }));
    await user.keyboard('{Escape}');
    expect(screen.getByRole('button', { name: 'Personalizar abas' })).toHaveFocus();
    act(() => backend.emitFlyoutShown());
    expect(screen.getByRole('button', { name: 'Personalizar abas' })).toHaveFocus();
    await user.click(screen.getByRole('button', { name: 'Escolher providers' }));
    await user.click(screen.getByRole('checkbox', { name: 'OpenRouter' }));
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(screen.getAllByRole('tab')).toHaveLength(1);
    expect(screen.getByRole('tabpanel')).toHaveAccessibleName('OpenRouter');
  });

  it('takes settings events immediately and preserves the tray reading of a hidden provider', async () => {
    const { backend } = renderApp();
    await screen.findByRole('heading', { name: 'Sessão de 5 horas' });
    await act(async () => {
      const settings = await backend.getSettings();
      settings.general.visibleTabs = ['litellm'];
      settings.general.traySource = 'codex';
      await backend.saveSettings(settings);
    });
    expect(screen.getByRole('tabpanel')).toHaveAccessibleName('LiteLLM');
    expect(screen.getByRole('heading', { level: 1 })).toHaveAttribute(
      'title',
      'Menor disponível: 18%',
    );
    expect(within(screen.getByRole('tablist')).getAllByRole('tab')).toHaveLength(1);
  });
});
