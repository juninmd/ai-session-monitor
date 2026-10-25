import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { DrawerShell } from './DrawerShell';

function mountDrawer() {
  render(
    <>
      <button type="button">behind</button>
      <DrawerShell titleId="drawer-title" title="Detail" closeLabel="Back" onClose={vi.fn()}>
        <button type="button">inside</button>
      </DrawerShell>
    </>,
  );
  const dialog = screen.getByRole('dialog');
  return {
    user: userEvent.setup(),
    dialog,
    close: within(dialog).getByRole('button', { name: 'Back' }),
  };
}

describe('the drawer dialog', () => {
  // The scrim covers the flyout, so assistive tech must treat what is behind it as unavailable.
  it('is modal', () => {
    const { dialog } = mountDrawer();
    expect(dialog).toHaveAttribute('aria-modal', 'true');
  });

  it('keeps Tab inside: from its last control it wraps to the first', async () => {
    const { user, close } = mountDrawer();
    expect(close).toHaveFocus();
    await user.tab();
    expect(
      within(screen.getByRole('dialog')).getByRole('button', { name: 'inside' }),
    ).toHaveFocus();
    await user.tab();
    expect(close).toHaveFocus();
  });

  it('keeps Shift+Tab inside: from its first control it wraps to the last', async () => {
    const { user, close } = mountDrawer();
    await user.tab({ shift: true });
    expect(
      within(screen.getByRole('dialog')).getByRole('button', { name: 'inside' }),
    ).toHaveFocus();
    expect(close).not.toHaveFocus();
  });
});
