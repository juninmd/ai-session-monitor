// The settings menu: one row per section, the state each row reports, and the way in and out.
import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { SettingsSection } from '../../app/harness';
import { type PanelOptions, renderPanel, settingsWith } from '../../test/helpers';
import { SettingsView } from './SettingsView';

function open(section: SettingsSection | null = null, options: PanelOptions = {}) {
  return renderPanel(<SettingsView section={section} onBack={vi.fn()} />, options);
}

const group = (id: string) => document.getElementById(`grp-${id}`) as HTMLDetailsElement;
const menu = () => screen.findByRole('heading', { level: 2, name: 'Configurações' });
/** One menu row, named by the section it leads with; the state follows in the same button. */
const row = (name: string) => screen.getByRole('button', { name: new RegExp(`^${name}`) });
const heading = (name: string) => screen.getByRole('heading', { level: 2, name });

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
    expect(screen.getByText('14 fontes · 3 não oficiais')).toBeVisible();
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

  it('opens one section in place of the menu, and comes back with the arrow', async () => {
    const { user } = open();
    await menu();
    await user.click(row('Histórico de uso'));
    expect(heading('Histórico de uso')).toHaveFocus();
    expect(screen.getByRole('switch', { name: 'Guardar o histórico de uso' })).toBeVisible();
    expect(screen.queryByRole('button', { name: /^Fontes de dados/ })).toBeNull();

    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    expect(await screen.findByRole('button', { name: /^Fontes de dados/ })).toBeVisible();
    expect(heading('Configurações')).toHaveFocus();
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

    await user.click(screen.getByRole('button', { name: 'Voltar' }));
    await user.click(await screen.findByRole('button', { name: /^Fontes de dados/ }));

    expect(heading('Fontes de dados')).toHaveFocus();
    expect(group('codex')).toHaveAttribute('open');
  });
});