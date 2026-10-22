// The settings menu: one row per section, the state each row reports, and the way in and out.
import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { SettingsSection } from '../../app/harness';
import { SettingsScreen } from '../../home/settingsHarness';
import { type PanelOptions, renderPanel, settingsWith } from '../../test/helpers';

function open(section: SettingsSection | null = null, options: PanelOptions = {}) {
  return renderPanel(<SettingsScreen initial={section} />, options);
}

const group = (id: string) => document.getElementById(`grp-${id}`) as HTMLDetailsElement;
const menu = () => screen.findByRole('heading', { level: 2, name: 'Configurações' });
/** One menu row, named by the section it leads with; the state follows in the same button. */
const row = (name: string) => screen.getByRole('button', { name: new RegExp(`^${name}`) });
const heading = (name: string) => screen.getByRole('heading', { level: 2, name });
/** The section drawer's own arrow, which leaves the section and not the screen. */
const back = () => within(screen.getByRole('dialog')).getByRole('button', { name: 'Voltar' });

describe('the settings menu', () => {
  it('lists one row per section, each with the state it is in now', async () => {
    open();
    await menu();
    for (const name of [
      'Monitoramento e alertas',
      'Histórico de uso',
      'Atualizações',
      'Fontes de dados',
    ]) {
      expect(row(name)).toBeVisible();
    }
    expect(screen.getByText('1 min · adaptativo · avisa em 25%')).toBeVisible();
    expect(screen.getByText('Ligado · 35 dias')).toBeVisible();
    expect(screen.getByText(/^Automáticas/)).toBeVisible();
    expect(screen.getByText('16 fontes · 5 não oficiais')).toBeVisible();
  });

  it('says the history is off instead of how long it keeps it', async () => {
    const off = settingsWith((s) => {
      s.general.historyEnabled = false;
    });
    open(null, { settings: off });
    await menu();
    expect(screen.getByText('Desligado')).toBeVisible();
    expect(screen.queryByText(/dias/)).toBeNull();
  });

  it('opens one section as a drawer over the menu, and comes back with the arrow', async () => {
    const { user } = open();
    await menu();
    await user.click(row('Histórico de uso'));
    // The section takes the keyboard on the drawer's own controls, and the menu stays behind the
    // scrim instead of being replaced.
    expect(back()).toHaveFocus();
    expect(heading('Histórico de uso')).toBeVisible();
    expect(screen.getByRole('switch', { name: 'Guardar o histórico de uso' })).toBeVisible();
    expect(screen.queryByRole('button', { name: /^Fontes de dados/ })).toBeNull();

    await user.click(back());
    expect(await screen.findByRole('button', { name: /^Fontes de dados/ })).toBeVisible();
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(await menu()).toBeVisible();
  });

  it('heads a section below the screen title, with no level skipped', async () => {
    const { user } = open();
    await menu();
    await user.click(row('Monitoramento e alertas'));
    expect(heading('Monitoramento e alertas')).toBeVisible();
    expect(screen.getByRole('heading', { level: 3, name: 'Geral' })).toBeVisible();
    expect(screen.getByRole('heading', { level: 3, name: 'Alertas' })).toBeVisible();
  });

  it('leaves the deep link behind once the menu opened the section', async () => {
    const { user } = open('codex');
    await heading('Fontes de dados');
    await vi.waitFor(() => expect(group('codex').querySelector('summary')).toHaveFocus());

    await user.click(back());
    await user.click(await screen.findByRole('button', { name: /^Fontes de dados/ }));

    // The row opened the section, so the drawer's own controls hold the keyboard; the group the
    // deep link expanded stays where the user left it.
    expect(back()).toHaveFocus();
    expect(group('codex')).toHaveAttribute('open');
  });
});
