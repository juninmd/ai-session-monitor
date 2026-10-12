import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { type SettingsSection, TAB_ORDER } from '../../app/harness';
import { BackendError } from '../../lib/backend';
import { problem } from '../../mock/fixtures';
import { type PanelOptions, renderPanel, settingsWith } from '../../test/helpers';
import { SettingsView } from './SettingsView';

function open(section: SettingsSection | null = null, options: PanelOptions = {}) {
  const onBack = vi.fn();
  const harness = renderPanel(<SettingsView section={section} onBack={onBack} />, options);
  return { ...harness, onBack };
}

const group = (id: string) => document.getElementById(`grp-${id}`) as HTMLDetailsElement;
const loaded = () => screen.findByRole('heading', { name: 'Fontes de dados' });

describe('the settings screen', () => {
  it('says it is loading until the stored settings arrive, and shows no control before', async () => {
    open(null, { latencyMs: 20 });
    expect(screen.getByText('Carregando as configurações')).toBeVisible();
    expect(screen.queryByRole('heading', { name: 'Geral' })).toBeNull();
    expect(screen.queryByRole('switch')).toBeNull();
    expect(await screen.findByRole('heading', { name: 'Geral' })).toBeVisible();
    expect(screen.queryByText('Carregando as configurações')).toBeNull();
  });

  it('puts focus on its heading, so a keyboard user starts at the top', () => {
    open();
    expect(screen.getByRole('heading', { level: 2, name: 'Configurações' })).toHaveFocus();
  });

  it('goes back with the back button', async () => {
    const { user, onBack } = open();
    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it('tells where the secrets live', async () => {
    open();
    await loaded();
    expect(screen.getByText(/Gerenciador de Credenciais do Windows e nunca chegam/)).toBeVisible();
  });
});

describe('the source groups', () => {
  it('list every source in tab order, all collapsed', async () => {
    open();
    await loaded();
    const ids = Array.from(document.querySelectorAll('details.grp')).map((el) => el.id);
    expect(ids).toEqual(TAB_ORDER.map((id) => `grp-${id}`));
    for (const id of TAB_ORDER) expect(group(id)).not.toHaveAttribute('open');
  });

  it('lead with the source logo, which assistive technology skips', async () => {
    open();
    await loaded();
    for (const id of TAB_ORDER) {
      const logo = group(id).querySelector('summary svg');
      expect(logo).toHaveAttribute('data-logo', id);
      expect(logo).toHaveAttribute('aria-hidden', 'true');
    }
  });

  it('say in words whether each source is read', async () => {
    const codexOff = settingsWith((s) => {
      s.codex.enabled = false;
    });
    open(null, { settings: codexOff });
    await vi.waitFor(() => expect(within(group('codex')).getByText('Desligado')).toBeVisible());
    expect(within(group('claude')).getByText('Ligado')).toBeVisible();
  });

  it('mark Antigravity as unofficial before it is even opened', async () => {
    open();
    await loaded();
    expect(within(group('antigravity')).getByText('não oficial')).toBeVisible();
    expect(within(group('codex')).queryByText('não oficial')).toBeNull();
  });

  it('mount a body only while open', async () => {
    const { user } = open();
    await loaded();
    expect(screen.queryByLabelText('Ler esta fonte')).toBeNull();

    await user.click(within(group('codex')).getByText('Codex'));
    expect(await within(group('codex')).findByLabelText('Ler esta fonte')).toBeVisible();
    expect(screen.getAllByLabelText('Ler esta fonte')).toHaveLength(1);

    await user.click(within(group('codex')).getByText('Codex'));
    await vi.waitFor(() => expect(screen.queryByLabelText('Ler esta fonte')).toBeNull());
  });

  it('open the one an error action pointed at, and move focus to it', async () => {
    const scroll = vi.spyOn(Element.prototype, 'scrollIntoView');
    open('codex');
    await loaded();
    await vi.waitFor(() => expect(group('codex').querySelector('summary')).toHaveFocus());
    expect(group('codex')).toHaveAttribute('open');
    for (const id of TAB_ORDER.filter((other) => other !== 'codex')) {
      expect(group(id)).not.toHaveAttribute('open');
    }
    expect(scroll).toHaveBeenCalled();
  });

  it('stay shut for the general section, which has no group', async () => {
    open('general');
    await loaded();
    for (const id of TAB_ORDER) expect(group(id)).not.toHaveAttribute('open');
    expect(screen.getByRole('heading', { level: 2, name: 'Configurações' })).toHaveFocus();
  });
});

describe('the help button', () => {
  it('opens the project page through the backend', async () => {
    const { user, backend } = open();
    await user.click(await screen.findByRole('button', { name: 'Como cada fonte funciona' }));
    expect(backend.openedLinks).toEqual(['project_readme']);
  });

  it('says so when the page could not be opened', async () => {
    const { user, backend } = open();
    vi.spyOn(backend, 'openLink').mockRejectedValue(new BackendError(problem('internal')));
    await user.click(await screen.findByRole('button', { name: 'Como cada fonte funciona' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Algo deu errado');
  });
});

describe('when the settings cannot be read', () => {
  const unreadable = (backend: Parameters<NonNullable<PanelOptions['prepare']>>[0]) => {
    vi.spyOn(backend, 'getSettings').mockRejectedValueOnce(
      new BackendError(problem('settings_unreadable')),
    );
  };

  it('explains why, shows no control, and offers exactly one retry', async () => {
    open(null, { prepare: unreadable });
    expect(
      await screen.findByRole('heading', { name: 'Arquivo de configurações ilegível' }),
    ).toBeVisible();
    expect(screen.queryByRole('heading', { name: 'Geral' })).toBeNull();
    expect(screen.queryByRole('switch')).toBeNull();
    expect(screen.getAllByRole('button', { name: 'Tentar de novo' })).toHaveLength(1);
  });

  it('shows the settings after a successful retry', async () => {
    const { user } = open(null, { prepare: unreadable });
    await user.click(await screen.findByRole('button', { name: 'Tentar de novo' }));
    expect(await screen.findByRole('heading', { name: 'Geral' })).toBeVisible();
    expect(screen.queryByRole('heading', { name: 'Arquivo de configurações ilegível' })).toBeNull();
  });
});

describe('when only the Credential Manager cannot be read', () => {
  // A read that never succeeds would otherwise hide every unrelated setting behind a retry card.
  const storeDown = (backend: Parameters<NonNullable<PanelOptions['prepare']>>[0]) => {
    vi.spyOn(backend, 'secretStatus').mockRejectedValueOnce(
      new BackendError(problem('internal', { where: 'secret_store' })),
    );
  };

  it('keeps every setting reachable and says what is wrong where the keys live', async () => {
    open(null, { prepare: storeDown });
    expect(await screen.findByRole('heading', { name: 'Geral' })).toBeVisible();
    expect(screen.queryByRole('heading', { name: 'Algo deu errado' })).toBeNull();
    expect(screen.getAllByRole('switch').length).toBeGreaterThan(0);
    for (const id of TAB_ORDER) expect(group(id)).toBeInTheDocument();
    expect(await screen.findByRole('alert')).toHaveTextContent(/Gerenciador de Credenciais/);
  });

  it('still saves a general setting', async () => {
    const { user, backend } = open(null, { prepare: storeDown });
    await user.click(await screen.findByRole('switch', { name: 'Iniciar com o Windows' }));
    await vi.waitFor(async () =>
      expect((await backend.getSettings()).general.autostart).toBe(true),
    );
  });

  it('reads the keys again on retry, and the notice goes away', async () => {
    const { user } = open(null, { prepare: storeDown });
    await screen.findByRole('alert');
    await user.click(screen.getByRole('button', { name: 'Tentar de novo' }));
    await vi.waitFor(() => expect(screen.queryByRole('alert')).toBeNull());
    // The button went with the notice; the keyboard waits on the section it belonged to.
    expect(await loaded()).toHaveFocus();
    await user.click(within(group('opencode')).getByText('OpenCode'));
    expect(await within(group('opencode')).findByText('Chave salva')).toBeVisible();
  });
});
