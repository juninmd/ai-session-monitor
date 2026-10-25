import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { renderPanel } from '../../test/helpers';
import { LinkButton } from './LinkButton';

describe('a link button', () => {
  // A second click while the first is still opening the page must not open a second page.
  it('opens its link once even when clicked again while pending', async () => {
    const { backend, user } = renderPanel(
      <LinkButton link="cursor_dashboard" label="Abrir painel" harness="cursor" />,
    );
    const open = vi.spyOn(backend, 'openLink').mockReturnValue(new Promise(() => {}));
    const button = screen.getByRole('button', { name: 'Abrir painel' });
    await user.click(button);
    await user.click(button);
    expect(open).toHaveBeenCalledTimes(1);
    expect(button).toBeDisabled();
  });
});
