import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { renderPanel } from '../../test/helpers';
import { SettingsMenu } from '../settings/SettingsMenu';
import { VendorConsoleLink } from './KeyConnect';
import { OpencodeLinks } from './OpencodeParts';

// The same guard as LinkButton: while a page is still opening, another click opens nothing.
describe('external link buttons', () => {
  it('the vendor console opens once even when clicked again while pending', async () => {
    const { backend, user } = renderPanel(<VendorConsoleLink harness="zai" />);
    const open = vi.spyOn(backend, 'openLink').mockReturnValue(new Promise(() => {}));
    const button = screen.getByRole('button', { name: 'Abrir as chaves do z.ai' });
    await user.click(button);
    await user.click(button);
    expect(open).toHaveBeenCalledTimes(1);
    expect(button).toBeDisabled();
  });

  it('the OpenCode console opens once even when clicked again while pending', async () => {
    const { backend, user } = renderPanel(<OpencodeLinks />);
    const open = vi.spyOn(backend, 'openLink').mockReturnValue(new Promise(() => {}));
    const button = screen.getByRole('button');
    await user.click(button);
    await user.click(button);
    expect(open).toHaveBeenCalledTimes(1);
    expect(button).toBeDisabled();
  });

  it('the settings help opens the readme once even when clicked again while pending', async () => {
    const { backend, user } = renderPanel(<SettingsMenu onOpen={() => {}} />);
    const open = vi.spyOn(backend, 'openLink').mockReturnValue(new Promise(() => {}));
    const button = screen.getByRole('button', { name: 'Como cada fonte funciona' });
    await user.click(button);
    await user.click(button);
    expect(open).toHaveBeenCalledTimes(1);
    expect(button).toBeDisabled();
  });
});
