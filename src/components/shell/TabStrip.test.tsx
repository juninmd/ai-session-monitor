import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { placeholderView, viewsInTabOrder } from '../../app/placeholder';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { createMockBackend } from '../../mock';
import { ctxOf, NOW } from '../../test/helpers';
import { PANEL_ID, TabStrip } from './TabStrip';

const NAMES = ['Claude Code', 'Codex', 'Antigravity', 'OpenCode', 'Kimi Code', 'z.ai', 'LiteLLM'];
const IDS: HarnessId[] = ['claude', 'codex', 'antigravity', 'opencode', 'kimi', 'zai', 'litellm'];

async function viewsOf(scenario: Parameters<typeof createMockBackend>[0] = {}) {
  const backend = createMockBackend({ now: () => NOW, ...scenario });
  return viewsInTabOrder(await backend.getSnapshot());
}

function Strip({ views, onSelect }: { views: HarnessView[]; onSelect?: (id: HarnessId) => void }) {
  const [selected, setSelected] = useState<HarnessId>('claude');
  return (
    <TabStrip
      views={views}
      selected={selected}
      ctx={ctxOf()}
      onSelect={(id) => {
        setSelected(id);
        onSelect?.(id);
      }}
    />
  );
}

const tabs = () => screen.getAllByRole('tab');
const selectedId = () => tabs().find((tab) => tab.getAttribute('aria-selected') === 'true')?.id;

describe('TabStrip semantics', () => {
  it('is one labelled tablist with the seven tools in order', async () => {
    render(<Strip views={await viewsOf({ scenario: 'healthy' })} />);
    expect(screen.getByRole('tablist', { name: 'Ferramentas de IA' })).toBeInTheDocument();
    expect(tabs().map((tab) => tab.id)).toEqual(IDS.map((id) => `tab-${id}`));
    tabs().forEach((tab, index) => {
      expect(tab).toHaveAccessibleName(new RegExp(`^${NAMES[index]}\\b`));
    });
  });

  it("leads each tab with that tool's own logo, hidden from assistive technology", async () => {
    render(<Strip views={await viewsOf({ scenario: 'healthy' })} />);
    tabs().forEach((tab, index) => {
      const logo = tab.querySelector('svg');
      expect(logo).toHaveAttribute('data-logo', IDS[index]);
      expect(logo).toHaveAttribute('aria-hidden', 'true');
      expect(logo).toHaveClass('t-icon');
    });
  });

  it('exposes the available share and the status as words in each name', async () => {
    render(<Strip views={await viewsOf({ scenario: 'healthy' })} />);
    expect(
      screen.getByRole('tab', { name: 'Claude Code 52% disponível, dentro do ritmo' }),
    ).toBeVisible();
  });

  it('points every tab at the one panel, and only the selected tab is tabbable', async () => {
    render(<Strip views={await viewsOf()} />);
    for (const tab of tabs()) expect(tab).toHaveAttribute('aria-controls', PANEL_ID);
    expect(tabs().map((tab) => tab.tabIndex)).toEqual([0, -1, -1, -1, -1, -1, -1]);
    expect(tabs().map((tab) => tab.getAttribute('aria-selected'))).toEqual([
      'true',
      'false',
      'false',
      'false',
      'false',
      'false',
      'false',
    ]);
  });

  it('says "carregando" for every tab before the first snapshot', () => {
    render(<Strip views={viewsInTabOrder(null)} />);
    for (const tab of tabs()) expect(tab).toHaveAccessibleName(/carregando$/);
    expect(within(tabs()[0] as HTMLElement).queryByText(/%/)).toBeNull();
  });

  it('never leaves a tab with only a colour to tell its state', async () => {
    render(<Strip views={await viewsOf({ scenario: 'critical' })} />);
    for (const tab of tabs()) {
      expect(tab).toHaveAccessibleName(/(atenção|crítico|esgotado)$/);
      // A glyph next to the percentage repeats the status for people who cannot see colour.
      expect(tab.querySelector('.t-read svg')).not.toBeNull();
    }
  });

  it('draws no glyph for a tab that is simply fine', async () => {
    render(<Strip views={await viewsOf({ scenario: 'healthy' })} />);
    for (const tab of tabs()) expect(tab.querySelector('.t-read svg')).toBeNull();
  });

  it('shows words for tabs that have no percentage', async () => {
    render(<Strip views={await viewsOf({ scenario: 'fresh' })} />);
    expect(tabs()[0]).toHaveAccessibleName(/precisa ser conectado$/);
    expect(tabs()[0]).toHaveTextContent('Conectar');
  });

  it('keeps a stale tab apart from a fresh one', async () => {
    render(<Strip views={await viewsOf({ scenario: 'stale' })} />);
    for (const tab of tabs()) expect(tab).toHaveAccessibleName(/dados desatualizados$/);
  });

  it('renders a placeholder as a loading tab', () => {
    render(<Strip views={IDS.map(placeholderView)} />);
    expect(tabs()).toHaveLength(7);
  });
});

describe('TabStrip keyboard model', () => {
  async function setup() {
    const onSelect = vi.fn();
    const user = userEvent.setup();
    render(<Strip views={await viewsOf({ scenario: 'healthy' })} onSelect={onSelect} />);
    await user.tab();
    return { user, onSelect };
  }

  it('lands on the selected tab when tabbing in', async () => {
    await setup();
    expect(document.activeElement?.id).toBe('tab-claude');
  });

  it('moves with the arrows, selecting as it goes', async () => {
    const { user, onSelect } = await setup();
    await user.keyboard('{ArrowRight}');
    expect(document.activeElement?.id).toBe('tab-codex');
    expect(selectedId()).toBe('tab-codex');
    expect(onSelect).toHaveBeenLastCalledWith('codex');
    await user.keyboard('{ArrowRight}{ArrowRight}');
    expect(selectedId()).toBe('tab-opencode');
    await user.keyboard('{ArrowLeft}');
    expect(selectedId()).toBe('tab-antigravity');
  });

  it('wraps around at both ends', async () => {
    const { user } = await setup();
    await user.keyboard('{ArrowLeft}');
    expect(selectedId()).toBe('tab-litellm');
    expect(document.activeElement?.id).toBe('tab-litellm');
    await user.keyboard('{ArrowRight}');
    expect(selectedId()).toBe('tab-claude');
  });

  it('jumps with Home and End', async () => {
    const { user } = await setup();
    await user.keyboard('{End}');
    expect(selectedId()).toBe('tab-litellm');
    await user.keyboard('{Home}');
    expect(selectedId()).toBe('tab-claude');
  });

  it('keeps the roving tabindex on the new tab', async () => {
    const { user } = await setup();
    await user.keyboard('{ArrowRight}');
    expect(tabs().map((tab) => tab.tabIndex)).toEqual([-1, 0, -1, -1, -1, -1, -1]);
  });

  it('leaves other keys to the browser', async () => {
    const { user, onSelect } = await setup();
    await user.keyboard('{ArrowDown}{ArrowUp}a');
    expect(onSelect).not.toHaveBeenCalled();
    expect(selectedId()).toBe('tab-claude');
  });

  it('selects on click', async () => {
    const { user, onSelect } = await setup();
    await user.click(screen.getByRole('tab', { name: /^OpenCode/ }));
    expect(onSelect).toHaveBeenLastCalledWith('opencode');
    expect(selectedId()).toBe('tab-opencode');
  });
});

describe('TabStrip scrolling', () => {
  it('scrolls the selected tab into view, also when the selection changes from outside', async () => {
    const scroll = vi.spyOn(Element.prototype, 'scrollIntoView');
    const views = await viewsOf({ scenario: 'healthy' });
    const { rerender } = render(
      <TabStrip views={views} selected="claude" ctx={ctxOf()} onSelect={() => {}} />,
    );
    rerender(<TabStrip views={views} selected="zai" ctx={ctxOf()} onSelect={() => {}} />);
    expect(scroll).toHaveBeenLastCalledWith({ block: 'nearest', inline: 'nearest' });
    expect(scroll.mock.contexts.at(-1)).toBe(document.getElementById('tab-zai'));
    scroll.mockRestore();
  });
});
