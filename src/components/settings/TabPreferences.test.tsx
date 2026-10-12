import { screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import { problem } from '../../mock/fixtures';
import { renderPanel, settingsWith } from '../../test/helpers';
import { TabPreferences } from './TabPreferences';

describe('tab preferences', () => {
  it('saves individual choices and retains source settings and secret flags', async () => {
    const { user, backend } = renderPanel(<TabPreferences />);
    const before = await backend.getSettings();
    const secrets = await backend.secretStatus();
    await user.click(screen.getByRole('checkbox', { name: 'Codex' }));
    await waitFor(() => expect(screen.getByRole('checkbox', { name: 'Codex' })).not.toBeChecked());
    const after = await backend.getSettings();
    expect(after.general.visibleTabs).not.toContain('codex');
    expect(after.codex).toEqual(before.codex);
    expect(after.litellm).toEqual(before.litellm);
    expect(await backend.secretStatus()).toEqual(secrets);
    expect(screen.getByText('13 de 14')).toBeVisible();
  });

  it('can hide all and restore all', async () => {
    const { user } = renderPanel(<TabPreferences />);
    await user.click(screen.getByRole('button', { name: 'Ocultar todos' }));
    await waitFor(() => expect(screen.getByText('0 de 14')).toBeVisible());
    expect(
      screen.getAllByRole('checkbox').every((input) => !(input as HTMLInputElement).checked),
    ).toBe(true);
    await user.click(screen.getByRole('button', { name: 'Exibir todos' }));
    await waitFor(() => expect(screen.getByText('14 de 14')).toBeVisible());
    expect(
      screen.getAllByRole('checkbox').every((input) => (input as HTMLInputElement).checked),
    ).toBe(true);
  });

  it('reports a failed save, retains the saved choice and allows a retry', async () => {
    const { user, backend } = renderPanel(<TabPreferences />, {
      prepare: (b) =>
        vi.spyOn(b, 'saveSettings').mockRejectedValueOnce(new BackendError(problem('internal'))),
    });
    await user.click(screen.getByRole('checkbox', { name: 'Codex' }));
    expect(await screen.findByRole('alert')).toBeVisible();
    expect(screen.getByRole('checkbox', { name: 'Codex' })).toBeChecked();
    expect((await backend.getSettings()).general.visibleTabs).toContain('codex');
    await user.click(screen.getByRole('checkbox', { name: 'Codex' }));
    await waitFor(() => expect(screen.getByRole('checkbox', { name: 'Codex' })).not.toBeChecked());
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('shows pending feedback and restores keyboard focus after the save', async () => {
    const { user } = renderPanel(<TabPreferences />, {
      latencyMs: 30,
      settings: settingsWith((s) => {
        s.general.visibleTabs = ['codex'];
      }),
    });
    await waitFor(() => expect(screen.getByText('1 de 14')).toBeVisible());
    const input = screen.getByRole('checkbox', { name: 'Codex' });
    input.focus();
    await user.keyboard(' ');
    expect(screen.getByRole('status')).toHaveTextContent('Salvando seleção');
    expect(input).toBeDisabled();
    await waitFor(() => expect(input).toBeEnabled());
    expect(input).toHaveFocus();
  });
});
