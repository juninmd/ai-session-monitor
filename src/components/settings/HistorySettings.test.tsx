import { screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import { renderPanel } from '../../test/helpers';
import { HistorySettings } from './HistorySettings';

const clearButton = () => screen.getByRole('button', { name: 'Limpar histórico' });
const show = () => renderPanel(<HistorySettings enabled />);

describe('history settings', () => {
  it('saves the switch like any other setting', async () => {
    const { user, backend } = show();
    const toggle = screen.getByRole('switch', { name: 'Guardar o histórico de uso' });
    expect(toggle).toBeChecked();
    await user.click(toggle);
    await waitFor(async () =>
      expect((await backend.getSettings()).general.historyEnabled).toBe(false),
    );
  });

  it('says what is stored and where', () => {
    show();
    expect(screen.getByText(/só porcentagens e horários/)).toBeVisible();
  });

  it('asks before erasing, and cancelling erases nothing', async () => {
    const { user, backend } = show();
    await user.click(clearButton());
    expect(screen.getByText(/Apagar todo o histórico salvo/)).toBeVisible();
    expect(backend.calls).not.toContain('clear_history');
    await user.click(screen.getByRole('button', { name: 'Cancelar' }));
    expect(screen.queryByText(/Apagar todo o histórico salvo/)).toBeNull();
    expect(backend.calls).not.toContain('clear_history');
    expect(clearButton()).toHaveFocus();
  });

  it('ties the erase button to the confirmation it opens', async () => {
    const { user } = show();
    expect(clearButton()).toHaveAttribute('aria-expanded', 'false');
    expect(clearButton()).not.toHaveAttribute('aria-controls');
    await user.click(clearButton());
    expect(clearButton()).toHaveAttribute('aria-expanded', 'true');
    const region = document.getElementById(clearButton().getAttribute('aria-controls') ?? '');
    expect(region).toHaveTextContent(/Apagar todo o histórico salvo/);
  });

  it('erases after the confirmation and announces it', async () => {
    const { user, backend } = show();
    await user.click(clearButton());
    await user.click(screen.getByRole('button', { name: 'Apagar' }));
    await waitFor(() => expect(backend.calls).toContain('clear_history'));
    expect(await screen.findByRole('status')).toHaveTextContent('Histórico apagado.');
    expect(clearButton()).toHaveFocus();
  });

  it('shows a mapped message, not the raw error, when erasing fails', async () => {
    const { user, backend } = show();
    vi.spyOn(backend, 'clearHistory').mockRejectedValue(
      new BackendError({ code: 'internal', retryable: true, params: { where: 'C:\\raw\\path' } }),
    );
    await user.click(clearButton());
    await user.click(screen.getByRole('button', { name: 'Apagar' }));
    expect(await screen.findByRole('alert')).toBeVisible();
    expect(document.body.textContent).not.toContain('C:\\raw\\path');
    expect(screen.queryByText('Histórico apagado.')).toBeNull();
  });
});
