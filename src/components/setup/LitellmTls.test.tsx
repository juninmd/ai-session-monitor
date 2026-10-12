import { screen } from '@testing-library/react';
import type { UserEvent } from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { renderPanel, settingsWith } from '../../test/helpers';
import { LitellmForm } from './LitellmForm';

const URL_OK = 'https://gateway.exemplo.dev';
const SECRET = 'fab-litellm-key';
const url = () => screen.getByLabelText('URL do servidor');
const key = () => screen.getByLabelText('Chave de API');
const submit = () => screen.getByRole('button', { name: 'Salvar e conectar' });
const mode = (name: string) => screen.getByRole('radio', { name });
const confirm = () => screen.getByRole('checkbox', { name: /Entendo o risco/ });
const host = () => screen.getByLabelText('Servidor sem verificação');
const advanced = () => screen.getByText('Avançado').closest('details');

/** A connected setup, so the key is already stored and only the transport is under test. */
const connected = () => renderPanel(<LitellmForm />, { scenario: 'healthy' });

async function untilLoaded() {
  await screen.findByText('Chave salva');
  expect(url()).toHaveValue(URL_OK);
}

/** The transport options live behind a disclosure, so a user has to open it first. */
async function openAdvanced(user: UserEvent) {
  await user.click(screen.getByText('Avançado'));
  expect(advanced()).toHaveAttribute('open');
}

async function chooseInsecure(user: UserEvent) {
  await untilLoaded();
  await openAdvanced(user);
  await user.click(mode('Não verificar'));
}

/** Another way of trusting the server is another destination for the key, so it is typed again. */
async function submitWithKey(user: UserEvent) {
  await user.type(key(), SECRET);
  await user.click(submit());
}

describe('the certificate choice', () => {
  it('starts on the Windows store and hides the other options', async () => {
    connected();
    await untilLoaded();
    expect(mode('Do Windows')).toBeChecked();
    expect(screen.queryByLabelText('Arquivo da CA (.pem)')).toBeNull();
    expect(screen.queryByLabelText('Servidor sem verificação')).toBeNull();
    expect(advanced()).not.toHaveAttribute('open');
  });

  it('asks for the CA file when a custom CA is chosen', async () => {
    const { user, backend } = connected();
    await untilLoaded();
    await openAdvanced(user);
    await user.click(mode('CA própria'));
    await submitWithKey(user);
    expect(screen.getByText('Informe o caminho do arquivo .pem.')).toBeVisible();
    expect(screen.getByLabelText('Arquivo da CA (.pem)')).toHaveFocus();
    expect(backend.calls).not.toContain('save_settings');
  });

  it('saves the CA path trimmed', async () => {
    const { user, backend } = connected();
    await untilLoaded();
    await openAdvanced(user);
    await user.click(mode('CA própria'));
    await user.type(screen.getByLabelText('Arquivo da CA (.pem)'), '  C:\\certs\\ca.pem  ');
    await submitWithKey(user);
    await screen.findByText('Configuração salva. Lendo o servidor…');
    expect((await backend.getSettings()).litellm.tls).toEqual({
      mode: 'custom_ca',
      pemPath: 'C:\\certs\\ca.pem',
    });
  });

  it('opens the advanced section by itself when the saved setup is not the default', async () => {
    const saved = settingsWith((s) => {
      s.litellm.tls = { mode: 'custom_ca', pemPath: 'C:\\certs\\ca.pem' };
    });
    renderPanel(<LitellmForm />, { scenario: 'healthy', settings: saved });
    await untilLoaded();
    expect(advanced()).toHaveAttribute('open');
    expect(screen.getByLabelText('Arquivo da CA (.pem)')).toHaveValue('C:\\certs\\ca.pem');
  });
});

describe('the saved key and the certificate choice', () => {
  const KEEP = 'Deixe em branco para manter a chave salva';

  it('is kept while the certificate setup is the saved one', async () => {
    const { user } = connected();
    await untilLoaded();
    await openAdvanced(user);
    await user.click(mode('CA própria'));
    await user.click(mode('Do Windows'));
    expect(key()).toHaveAttribute('placeholder', KEEP);
  });

  it('is asked for again when another way of trusting the server is chosen', async () => {
    const { user, backend } = connected();
    await untilLoaded();
    await openAdvanced(user);
    await user.click(mode('CA própria'));
    await user.type(screen.getByLabelText('Arquivo da CA (.pem)'), 'C:\\certs\\ca.pem');
    expect(key()).not.toHaveAttribute('placeholder', KEEP);
    await user.click(submit());
    expect(screen.getByText('Cole a chave de API do LiteLLM.')).toBeVisible();
    expect(backend.calls).not.toContain('save_settings');
  });

  it('is asked for again when the CA file changes', async () => {
    const saved = settingsWith((s) => {
      s.litellm.tls = { mode: 'custom_ca', pemPath: 'C:\\certs\\ca.pem' };
    });
    const { user, backend } = renderPanel(<LitellmForm />, {
      scenario: 'healthy',
      settings: saved,
    });
    await untilLoaded();
    expect(key()).toHaveAttribute('placeholder', KEEP);
    await user.type(screen.getByLabelText('Arquivo da CA (.pem)'), 'x');
    expect(key()).not.toHaveAttribute('placeholder', KEEP);
    await user.click(submit());
    expect(screen.getByText('Cole a chave de API do LiteLLM.')).toBeVisible();
    expect(backend.calls).not.toContain('save_settings');
  });
});

describe('not verifying the certificate', () => {
  it('warns in words, names the server and starts unconfirmed', async () => {
    const { user } = connected();
    await chooseInsecure(user);
    expect(screen.getByText(/a chave pode ser interceptada/)).toBeVisible();
    expect(host()).toHaveValue('gateway.exemplo.dev');
    expect(confirm()).not.toBeChecked();
  });

  it('will not save until the risk is confirmed, and says where', async () => {
    const { user, backend } = connected();
    await chooseInsecure(user);
    await submitWithKey(user);
    expect(screen.getByText('Confirme para continuar sem verificar o certificado.')).toBeVisible();
    expect(confirm()).toHaveAttribute('aria-invalid', 'true');
    expect(confirm()).toHaveFocus();
    expect(backend.calls).not.toContain('save_settings');
    expect((await backend.getSettings()).litellm.tls).toEqual({ mode: 'system' });
  });

  it('needs a server name too', async () => {
    const { user, backend } = connected();
    await chooseInsecure(user);
    await user.clear(host());
    await user.click(confirm());
    await submitWithKey(user);
    expect(screen.getByText('Informe o servidor.')).toBeVisible();
    expect(host()).toHaveFocus();
    expect(backend.calls).not.toContain('save_settings');
  });

  it('saves the confirmed server, and only that server', async () => {
    const { user, backend } = connected();
    await chooseInsecure(user);
    await user.click(confirm());
    await submitWithKey(user);
    await screen.findByText('Configuração salva. Lendo o servidor…');
    expect((await backend.getSettings()).litellm.tls).toEqual({
      mode: 'insecure',
      host: 'gateway.exemplo.dev',
    });
  });

  it('forgets the confirmation when the server changes', async () => {
    const { user } = connected();
    await chooseInsecure(user);
    await user.click(confirm());
    expect(confirm()).toBeChecked();
    await user.type(host(), 'x');
    expect(confirm()).not.toBeChecked();
  });

  it('forgets the confirmation when the user leaves and comes back', async () => {
    const { user } = connected();
    await chooseInsecure(user);
    await user.click(confirm());
    await user.click(mode('Do Windows'));
    await user.click(mode('Não verificar'));
    expect(confirm()).not.toBeChecked();
  });

  it('shows a saved insecure setup as already confirmed, with the warning still on screen', async () => {
    const saved = settingsWith((s) => {
      s.litellm.tls = { mode: 'insecure', host: 'gateway.exemplo.dev' };
    });
    renderPanel(<LitellmForm />, { scenario: 'healthy', settings: saved });
    await untilLoaded();
    expect(advanced()).toHaveAttribute('open');
    expect(mode('Não verificar')).toBeChecked();
    expect(confirm()).toBeChecked();
    expect(screen.getByText(/a chave pode ser interceptada/)).toBeVisible();
  });

  it('goes back to the Windows store without carrying the old server along', async () => {
    const saved = settingsWith((s) => {
      s.litellm.tls = { mode: 'insecure', host: 'gateway.exemplo.dev' };
    });
    const { user, backend } = renderPanel(<LitellmForm />, {
      scenario: 'healthy',
      settings: saved,
    });
    await untilLoaded();
    await user.click(mode('Do Windows'));
    await submitWithKey(user);
    await screen.findByText('Configuração salva. Lendo o servidor…');
    expect((await backend.getSettings()).litellm.tls).toEqual({ mode: 'system' });
  });
});
