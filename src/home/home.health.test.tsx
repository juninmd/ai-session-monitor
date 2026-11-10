// The triage's health strip on fabricated data: the chips count what the grid shows, filter it,
// and leave the keyboard a way into whatever is left.
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { mount, openDetail, panel, rows } from './testHarness';

const chips = () => within(screen.getByRole('group', { name: 'Filtrar por estado' }));
const chip = (name: RegExp) => chips().getByRole('button', { name });

describe('triage health strip', () => {
  it('counts every provider once across its chips', async () => {
    mount();
    const all = await rows('data-tier');
    expect(chip(/^Todos/)).toHaveAttribute('aria-pressed', 'true');
    expect(chip(/^Todos/).textContent).toContain(String(all.length));
    const counted = chips()
      .getAllByRole('button')
      .filter((button) => button.getAttribute('data-bucket') !== 'all')
      .reduce((sum, button) => sum + Number(button.querySelector('b')?.textContent), 0);
    expect(counted).toBe(all.length);
  });

  it('filters the grid to one state and back', async () => {
    const { user } = mount();
    const all = await rows('data-tier');
    await user.click(chip(/^Agir agora/));
    const acting = await rows('data-tier');
    expect(acting.length).toBeLessThan(all.length);
    for (const tile of acting) {
      expect(['blocked', 'risk', 'broken']).toContain(tile.getAttribute('data-tier'));
    }
    // Pressing the active chip again shows everything.
    await user.click(chip(/^Agir agora/));
    await waitFor(async () => expect(await rows('data-tier')).toHaveLength(all.length));
    expect(chip(/^Todos/)).toHaveAttribute('aria-pressed', 'true');
  });

  it('keeps one tile in the tab order when the highlighted provider is filtered out', async () => {
    const { user } = mount();
    await rows('data-tier');
    // The demo highlights a calm provider; none of those is among the ones to act on.
    await user.click(chip(/^Agir agora/));
    const acting = await rows('data-tier');
    expect(acting.every((tile) => tile.getAttribute('aria-pressed') === 'false')).toBe(true);
    expect(acting.filter((tile) => tile.tabIndex === 0)).toEqual([acting[0]]);
  });

  it('is not drawn when every provider is in the same state', async () => {
    mount('loading');
    await rows('data-tier');
    expect(screen.queryByRole('group', { name: 'Filtrar por estado' })).toBeNull();
  });
});

describe('trend chart inspection', () => {
  it('reads the running pass where the scrubber rests, and the latest reading otherwise', async () => {
    const { user } = mount();
    await openDetail(user, 'Codex');
    const drawer = panel('Codex');
    const scrub = await within(drawer).findByRole('slider', {
      name: 'Inspecionar as leituras da janela atual',
    });
    const readout = () => drawer.querySelector('.nx-chart-readout');
    const latest = readout()?.textContent ?? '';
    expect(latest).toMatch(/% usado após/);
    expect(scrub).toHaveAttribute('aria-valuetext', latest);
    expect(drawer.querySelector('.nx-chart-cursor')).toBeNull();

    // jsdom does not move a range with the keyboard; the change is what a key or a drag produces.
    fireEvent.change(scrub, { target: { value: '0' } });
    await waitFor(() => expect(drawer.querySelector('.nx-chart-cursor')).not.toBeNull());
    expect(readout()?.textContent).not.toBe(latest);
    expect(readout()).toHaveAttribute('data-live');

    // Leaving the scrubber returns the line to the latest reading.
    fireEvent.blur(scrub);
    await waitFor(() => expect(drawer.querySelector('.nx-chart-cursor')).toBeNull());
    expect(readout()?.textContent).toBe(latest);
  });
});
