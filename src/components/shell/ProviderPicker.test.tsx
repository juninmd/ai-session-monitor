import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import { viewsInTabOrder } from '../../app/placeholder';
import { createMockBackend } from '../../mock';
import { ctxOf, NOW } from '../../test/helpers';
import { ProviderPicker } from './ProviderPicker';
import { TabStrip } from './TabStrip';

async function setup() {
  const views = viewsInTabOrder(
    await createMockBackend({ scenario: 'healthy', now: () => NOW }).getSnapshot(),
  );
  const onSelect = vi.fn();
  render(
    <>
      <TabStrip views={views} selected="claude" ctx={ctxOf()} onSelect={onSelect} />
      <ProviderPicker views={views} selected="claude" ctx={ctxOf()} onSelect={onSelect} />
    </>,
  );
  const user = userEvent.setup();
  await user.click(screen.getByText('Todas as ferramentas'));
  await screen.findByRole('searchbox');
  return { user, onSelect };
}

it('finds a distant provider, selects it and restores focus to its tab', async () => {
  const { user, onSelect } = await setup();
  await user.type(screen.getByRole('searchbox'), '  mInImAx');
  expect(screen.getAllByRole('button', { pressed: false })).toHaveLength(1);
  await user.click(screen.getByRole('button', { name: /^MiniMax/ }));
  expect(onSelect).toHaveBeenCalledWith('minimax');
  expect(document.activeElement?.id).toBe('tab-minimax');
  await waitFor(() => expect(screen.queryByRole('searchbox')).not.toBeInTheDocument());
});

it('provides a recoverable empty search and clears the query on closing', async () => {
  const { user } = await setup();
  await user.type(screen.getByRole('searchbox'), 'nonexistent');
  expect(screen.getByRole('status')).toHaveTextContent('Nenhuma ferramenta encontrada');
  await user.click(screen.getByRole('button', { name: 'Limpar busca' }));
  expect(screen.getByRole('searchbox')).toHaveValue('');
  expect(screen.getByRole('searchbox')).toHaveFocus();
  await user.type(screen.getByRole('searchbox'), 'codex');
  await user.click(screen.getByText('Todas as ferramentas'));
  await waitFor(() => expect(screen.queryByRole('searchbox')).not.toBeInTheDocument());
  await user.click(screen.getByText('Todas as ferramentas'));
  expect(await screen.findByRole('searchbox')).toHaveValue('');
  expect(screen.getAllByRole('button', { pressed: false })).toHaveLength(13);
});

it('Escape closes the overview without hiding the app and returns focus to its summary', async () => {
  const { user } = await setup();
  const listener = vi.fn((event: KeyboardEvent) => expect(event.defaultPrevented).toBe(true));
  document.addEventListener('keydown', listener);
  try {
    await user.click(screen.getByRole('searchbox'));
    await user.keyboard('{Escape}');
    await waitFor(() => expect(screen.queryByRole('searchbox')).not.toBeInTheDocument());
    expect(screen.getByText('Todas as ferramentas').closest('summary')).toHaveFocus();
    expect(listener).toHaveBeenCalledOnce();
  } finally {
    document.removeEventListener('keydown', listener);
  }
});
