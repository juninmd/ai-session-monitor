import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import { LIMITS } from '../../lib/contract';
import { problem } from '../../mock/fixtures';
import { renderPanel, settingsWith } from '../../test/helpers';
import { GeneralSettings } from './GeneralSettings';

const optionValues = (label: string) =>
  Array.from(screen.getByLabelText(label).querySelectorAll('option')).map((o) => Number(o.value));

async function stored(backend: { getSettings: () => Promise<{ general: object }> }) {
  return (await backend.getSettings()).general as Record<string, unknown>;
}

describe('selects', () => {
  it('save the moment they change', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    await user.selectOptions(screen.getByLabelText('Atualizar a cada'), '300');
    await vi.waitFor(async () => expect((await stored(backend)).intervalSecs).toBe(300));
    await user.selectOptions(screen.getByLabelText('Avisar quando restar até'), '40');
    await vi.waitFor(async () => expect((await stored(backend)).warnAtPct).toBe(40));
  });

  it.each([
    ['Atualizar a cada', LIMITS.intervalSecs],
    ['Avisar quando restar até', LIMITS.warnAtPct],
  ])('offer %s only inside the range the backend accepts', (label, range) => {
    renderPanel(<GeneralSettings />);
    const values = optionValues(label);
    expect(values[0]).toBe(range.min);
    expect(values.at(-1)).toBe(range.max);
    expect(values).toEqual([...values].sort((a, b) => a - b));
    expect(new Set(values).size).toBe(values.length);
  });

  it('keep a stored value that is not one of the presets', async () => {
    const odd = settingsWith((s) => {
      s.general.intervalSecs = 45;
      s.general.warnAtPct = 33;
    });
    renderPanel(<GeneralSettings />, { settings: odd });
    await vi.waitFor(() => expect(screen.getByLabelText('Atualizar a cada')).toHaveValue('45'));
    expect(screen.getByLabelText('Avisar quando restar até')).toHaveValue('33');
    expect(screen.getByRole('option', { name: '33%' })).toBeInTheDocument();
  });

  it('list "Menor disponível" first and then every source, in tab order', () => {
    renderPanel(<GeneralSettings />);
    const labels = Array.from(
      screen.getByLabelText('Ícone da bandeja mostra').querySelectorAll('option'),
    ).map((o) => o.textContent);
    expect(labels).toEqual([
      'Menor disponível',
      'Claude Code',
      'Codex',
      'Antigravity',
      'OpenCode',
      'Kimi Code',
      'z.ai',
      'OpenRouter',
      'DeepSeek',
      'LiteLLM',
    ]);
  });

  it('store "Menor disponível" as no choice, and a source as its id', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    const tray = screen.getByLabelText('Ícone da bandeja mostra');
    expect(tray).toHaveValue('lowest');
    await user.selectOptions(tray, 'codex');
    await vi.waitFor(async () => expect((await stored(backend)).traySource).toBe('codex'));
    await user.selectOptions(tray, 'lowest');
    await vi.waitFor(async () => expect((await stored(backend)).traySource).toBeNull());
  });
});

describe('switches', () => {
  it.each([
    ['Iniciar com o Windows', 'autostart'],
    ['Avisar ao esgotar', 'notifyOnOut'],
    ['Avisar quando a janela resetar', 'notifyOnReset'],
  ])('%s toggles %s and says which way it is', async (label, key) => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    const control = screen.getByRole('switch', { name: label });
    expect(control).toHaveAttribute('aria-checked', 'false');
    await user.click(control);
    await vi.waitFor(async () => expect((await stored(backend))[key]).toBe(true));
    expect(control).toHaveAttribute('aria-checked', 'true');
    await user.click(control);
    await vi.waitFor(async () => expect((await stored(backend))[key]).toBe(false));
  });

  it('reach the keyboard: Space flips a focused switch', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    screen.getByRole('switch', { name: 'Avisar ao esgotar' }).focus();
    await user.keyboard(' ');
    await vi.waitFor(async () => expect((await stored(backend)).notifyOnOut).toBe(true));
  });
});

describe('segmented choices', () => {
  it.each([
    ['Formato do tempo', '1:53', 'timeFormat', 'colon'],
    ['Formato do tempo', '1,9 h', 'timeFormat', 'decimal'],
    ['Semana em', 'dias e horas', 'weekFormat', 'days_hours'],
    ['Mostrar cota como', 'usado', 'showUsed', true],
  ])('%s: picking %s saves %s=%s', async (group, choice, key, expected) => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    const radios = within(screen.getByRole('radiogroup', { name: group }));
    await user.click(radios.getByRole('radio', { name: choice }));
    await vi.waitFor(async () => expect((await stored(backend))[key]).toBe(expected));
    expect(radios.getByRole('radio', { name: choice })).toBeChecked();
  });

  it('start on the stored choice', () => {
    renderPanel(<GeneralSettings />);
    const time = within(screen.getByRole('radiogroup', { name: 'Formato do tempo' }));
    expect(time.getByRole('radio', { name: '1h 53m' })).toBeChecked();
    const week = within(screen.getByRole('radiogroup', { name: 'Semana em' }));
    expect(week.getByRole('radio', { name: 'horas' })).toBeChecked();
  });
});

describe('the theme', () => {
  it('follows Windows and offers no control that would pretend otherwise', () => {
    renderPanel(<GeneralSettings />);
    expect(screen.getByText('Segue o Windows.')).toBeVisible();
    expect(screen.queryByRole('radio', { name: /claro|escuro/i })).toBeNull();
    expect(screen.queryByLabelText('Tema')).toBeNull();
  });
});

describe('a refused save', () => {
  it('leaves the control where it was and says why beside it', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    vi.spyOn(backend, 'saveSettings').mockRejectedValueOnce(
      new BackendError(problem('invalid_input', { field: 'intervalSecs' })),
    );
    const select = screen.getByLabelText('Atualizar a cada');
    await user.selectOptions(select, '300');
    expect(await screen.findByText(/Valor inválido\. Corrija o campo indicado/)).toBeVisible();
    expect(select).toHaveValue('60');
    expect((await stored(backend)).intervalSecs).toBe(60);
  });

  it('is spoken, because focus stays on the control that was used', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    vi.spyOn(backend, 'saveSettings').mockRejectedValueOnce(new BackendError(problem('internal')));
    const autostart = screen.getByRole('switch', { name: 'Iniciar com o Windows' });
    await user.click(autostart);
    expect(await screen.findByRole('alert')).toHaveTextContent(/Algo deu errado\./);
    expect(autostart).toHaveFocus();
    expect(autostart).not.toBeChecked();
  });

  it('forgets the complaint once a later change goes through', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    vi.spyOn(backend, 'saveSettings').mockRejectedValueOnce(new BackendError(problem('internal')));
    const select = screen.getByLabelText('Atualizar a cada');
    await user.selectOptions(select, '300');
    expect(await screen.findByText(/Algo deu errado\./)).toBeVisible();
    await user.selectOptions(select, '600');
    await vi.waitFor(() => expect(screen.queryByText(/Algo deu errado\./)).toBeNull());
    expect(select).toHaveValue('600');
  });

  it('keeps raw error text away from the screen', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    vi.spyOn(backend, 'saveSettings').mockRejectedValueOnce(
      new Error('EPERM: C:\\Users\\someone\\settings.json'),
    );
    await user.click(screen.getByRole('switch', { name: 'Iniciar com o Windows' }));
    expect(await screen.findByText(/Algo deu errado\./)).toBeVisible();
    expect(document.body).not.toHaveTextContent('EPERM');
    expect(document.body).not.toHaveTextContent('someone');
  });
});

describe('usage alerts and adaptive pacing', () => {
  it('start on the defaults: alerts at 80 and 95, adaptive on', () => {
    renderPanel(<GeneralSettings />);
    expect(screen.getByLabelText('Avisar ao atingir este uso')).toHaveValue('80,95');
    expect(screen.getByRole('switch', { name: /Ritmo adaptativo/ })).toBeChecked();
  });

  it('save a preset, off, and keep a custom list that was stored', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    const select = screen.getByLabelText('Avisar ao atingir este uso');
    await user.selectOptions(select, '90');
    await vi.waitFor(async () => expect((await stored(backend)).alertThresholds).toEqual([90]));
    await user.selectOptions(select, 'off');
    await vi.waitFor(async () => expect((await stored(backend)).alertThresholds).toEqual([]));
  });

  it('show a stored custom list as "Personalizado"', async () => {
    const custom = settingsWith((s) => {
      s.general.alertThresholds = [60, 85];
    });
    renderPanel(<GeneralSettings />, { settings: custom });
    await vi.waitFor(() =>
      expect(screen.getByLabelText('Avisar ao atingir este uso')).toHaveValue('60,85'),
    );
    expect(screen.getByRole('option', { name: 'Personalizado (60% e 85%)' })).toBeInTheDocument();
  });

  it('turn adaptive pacing off and on', async () => {
    const { user, backend } = renderPanel(<GeneralSettings />);
    await user.click(screen.getByRole('switch', { name: /Ritmo adaptativo/ }));
    await vi.waitFor(async () => expect((await stored(backend)).adaptive).toBe(false));
  });
});
