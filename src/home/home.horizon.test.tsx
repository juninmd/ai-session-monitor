// The horizon, density and pinned-provider surfaces, rendered on fabricated data.
import { screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../lib/backend';
import { mount, openDetail, rows } from './testHarness';

describe('horizon surface', () => {
  it('explains the bar and refuses to project unknown readings', async () => {
    const { user } = mount();
    await rows('data-tier');
    await user.click(screen.getByRole('button', { name: /^Horizonte/ }));
    expect(screen.getByRole('button', { name: 'Sobre o horizonte' })).toBeVisible();
    expect(screen.getByRole('tooltip')).toHaveTextContent('Chego até a renovação?');
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

describe('pinned providers', () => {
  it('lead the triage list in pin order and carry a pin marker', async () => {
    mount('demo', {
      settings: (draft) => {
        draft.general.pinned = ['minimax', 'codex'];
      },
    });
    const list = await rows('data-tier');
    expect(list[0]?.textContent).toContain('MiniMax');
    expect(list[1]?.textContent).toContain('Codex');
    expect(list[0]?.querySelector('[aria-label="Fixado no topo"]')).not.toBeNull();
    expect(list[2]?.querySelector('[aria-label="Fixado no topo"]')).toBeNull();
  });

  it('toggles from the detail drawer and moves the provider to the top', async () => {
    const { user } = mount();
    await openDetail(user, 'MiniMax');
    await user.click(screen.getByRole('button', { name: 'Fixar MiniMax no topo' }));
    await waitFor(async () => {
      const list = await rows('data-tier');
      expect(list[0]?.textContent).toContain('MiniMax');
    });
    expect(screen.getByRole('button', { name: 'Fixar MiniMax no topo' })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });

  it('says when the pin could not be saved and leaves the pin where it was', async () => {
    const { backend, user } = mount();
    vi.spyOn(backend, 'saveSettings').mockRejectedValueOnce(
      new BackendError({ code: 'internal', retryable: false, params: {} }),
    );
    await openDetail(user, 'MiniMax');
    const pin = screen.getByRole('button', { name: 'Fixar MiniMax no topo' });
    await user.click(pin);
    const drawer = screen.getByRole('dialog');
    expect(await within(drawer).findByRole('alert')).toBeInTheDocument();
    expect(pin).toHaveAttribute('aria-pressed', 'false');
  });
});
