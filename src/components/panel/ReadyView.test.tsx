import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { HarnessView, Settings } from '../../lib/contract';
import { antigravityView, claudeView, codexView, litellmView } from '../../mock/builders';
import { viewOf } from '../../mock/fixtures';
import { ctxOf, envOf, renderPanel, settingsWith, win } from '../../test/helpers';
import { HarnessPanel } from '../HarnessPanel';

function ready(view: HarnessView, settings?: Settings) {
  const options = settings ? { settings } : {};
  return renderPanel(<HarnessPanel view={view} ctx={ctxOf(settings)} />, options);
}

const section = (name: string | RegExp) => {
  const found = screen.getByRole('heading', { name }).closest('section');
  if (!found) throw new Error(`no section for ${String(name)}`);
  return found;
};
const meterIn = (el: HTMLElement) => within(el).getByRole('meter') as HTMLMeterElement;

describe('the hero window', () => {
  it('leads with the time to reset, then the gauge, availability and pace', () => {
    ready(claudeView(envOf(), 'ok'));
    const hero = section('Sessão de 5 horas');
    expect(hero).toHaveClass('hero');
    expect(hero).toHaveAttribute('data-status', 'ok');
    expect(within(hero).getByText('Faltam 1 hora e 53 minutos')).toBeInTheDocument();
    expect(hero.querySelector('.hero-time')).toHaveTextContent(/1h53m\s*para resetar/);
    expect(within(hero).getByText(/^reseta às \d{2}:\d{2}$/)).toBeVisible();
    expect(within(hero).getByText('52%')).toBeVisible();
    expect(within(hero).getByText('No ritmo atual, dura além do reset.')).toBeVisible();
  });

  it('is a meter whose fill is what is still available', () => {
    ready(claudeView(envOf(), 'ok'));
    const meter = screen.getByRole('meter', { name: 'Sessão de 5 horas: 52% disponível' });
    expect(meter).toHaveAttribute('value', '52');
    expect(meter).toHaveAttribute('min', '0');
    expect(meter).toHaveAttribute('max', '100');
  });

  it('marks the share of the window left with a tick and explains it in a tooltip', () => {
    ready(claudeView(envOf(), 'ok'));
    const hero = section('Sessão de 5 horas');
    expect(hero.querySelector<HTMLElement>('.tick')?.style.getPropertyValue('--x')).toBe('37.7%');
    expect(
      within(hero).getByText(/Faltam 1h 53m \(38% da janela de 5 horas\)/),
    ).toBeInTheDocument();
  });

  it('warns about a pace that will not last', () => {
    ready(claudeView(envOf(), 'warn'));
    expect(section('Sessão de 5 horas')).toHaveAttribute('data-status', 'warn');
    expect(screen.getByText(/No ritmo atual, esgota às \d{2}:\d{2}/)).toBeVisible();
  });

  it('says exhausted, and when the window comes back', () => {
    ready(claudeView(envOf(), 'crit'));
    const hero = section('Sessão de 5 horas');
    expect(hero).toHaveAttribute('data-status', 'out');
    expect(within(hero).getAllByText('Esgotado').length).toBeGreaterThan(0);
    expect(within(hero).getByText(/^volta às \d{2}:\d{2}$/)).toBeVisible();
    expect(within(hero).getByText(/Esgotado\. Volta às \d{2}:\d{2}, em 47 min\./)).toBeVisible();
    expect(meterIn(hero)).toHaveAttribute('value', '0');
  });

  it('reports usage above the limit instead of hiding it', () => {
    ready(viewOf('codex', { windows: [win({ usedPct: 130 })] }));
    expect(screen.getByText('Uso acima do limite: 130% usado.')).toBeVisible();
    expect(meterIn(section('Janela de 5 horas'))).toHaveAttribute('value', '0');
  });

  it('formats time as the user chose', () => {
    const colon = settingsWith((s) => {
      s.general.timeFormat = 'colon';
    });
    ready(claudeView(envOf(), 'ok'), colon);
    expect(section('Sessão de 5 horas').querySelector('.hero-time')).toHaveTextContent('1:53');
  });

  it('reads multi-day windows in days and hours when asked', () => {
    const days = settingsWith((s) => {
      s.general.weekFormat = 'days_hours';
    });
    ready(claudeView(envOf(), 'ok'), days);
    expect(section('Semana, todos os modelos')).toHaveTextContent('4d 11h');
  });
});

describe('secondary windows', () => {
  it('list their title, time, gauge and reset', () => {
    ready(codexView(envOf(), 'ok'));
    const week = section('Semana');
    expect(week).toHaveClass('flat');
    expect(within(week).getByText('Faltam 4 dias, 3 horas e 40 minutos')).toBeInTheDocument();
    expect(week.querySelector('.row-time')).toHaveTextContent('99h 40m');
    expect(meterIn(week)).toHaveAttribute('value', '72');
    expect(within(week).getByText(/^reseta /)).toBeVisible();
  });
});

describe('windows that cannot be trusted', () => {
  it('say an expired window has reset once per card, not once per slot', () => {
    const view = viewOf('codex', {
      windows: [
        win({ resetsInMin: -10 }),
        win({ id: 'weekly', kind: 'weekly', windowMinutes: 10080, resetsInMin: -5 }),
      ],
    });
    ready(view);
    const hero = section('Janela de 5 horas');
    expect(within(hero).getAllByText('Reiniciou')).toHaveLength(1);
    expect(within(hero).getAllByText(/^A janela reiniciou /)).toHaveLength(1);
    expect(within(hero).queryAllByText('reiniciou')).toHaveLength(0);
    expect(within(hero).queryAllByText(/^reiniciou (às|em) /)).toHaveLength(0);
    // An empty footer row would still cost the grid gap under the hatched gauge.
    expect(hero.querySelector('.meter-foot')).toBeNull();
    const week = section('Semana');
    expect(within(week).queryAllByText('reiniciou')).toHaveLength(0);
    expect(within(week).getAllByText(/^reiniciou (às|em) /)).toHaveLength(1);
  });

  it('say "reiniciou" for a window whose reset has passed, with no status or pace', () => {
    const view = viewOf('codex', {
      windows: [win({ id: 'weekly', kind: 'weekly', windowMinutes: 10080, resetsInMin: -10 })],
    });
    ready(view);
    const hero = section('Semana');
    expect(hero).toHaveAttribute('data-status', 'none');
    expect(within(hero).getAllByText('Reiniciou').length).toBeGreaterThan(0);
    expect(
      within(hero).getByText(/^A janela reiniciou .* Aguardando uma nova leitura\.$/),
    ).toBeVisible();
    expect(within(hero).queryByRole('meter')).toBeNull();
    expect(
      within(hero).getByRole('img', { name: 'Semana: janela reiniciada' }),
    ).toBeInTheDocument();
    expect(within(hero).queryByText(/No ritmo atual/)).toBeNull();
    expect(hero).not.toHaveTextContent('%');
  });

  it('keep an expired secondary window out of the status colours', () => {
    const view = viewOf('codex', {
      windows: [
        win(),
        win({ id: 'weekly', kind: 'weekly', windowMinutes: 10080, resetsInMin: -5 }),
      ],
    });
    ready(view);
    const week = section('Semana');
    expect(week).toHaveAttribute('data-status', 'none');
    expect(within(week).getByText(/^reiniciou (às|em) /)).toBeVisible();
    expect(within(week).queryByRole('meter')).toBeNull();
  });

  it('show an unknown percentage as unknown, never as zero', () => {
    ready(viewOf('codex', { windows: [win({ usedPct: null })] }));
    const hero = section('Janela de 5 horas');
    expect(within(hero).getByText('Percentual desconhecido')).toBeVisible();
    expect(within(hero).getByRole('img', { name: /percentual desconhecido$/ })).toBeInTheDocument();
    expect(within(hero).queryByRole('meter')).toBeNull();
    expect(hero).not.toHaveTextContent('0%');
    expect(hero).toHaveAttribute('data-status', 'none');
  });

  it('keep an unknown quota pool unknown next to a known one', () => {
    ready(antigravityView(envOf(), 'ok', true));
    const known = section('Gemini');
    const unknown = section('Claude e GPT-OSS');
    expect(within(known).getAllByRole('meter')).toHaveLength(2);
    expect(within(unknown).queryByRole('meter')).toBeNull();
    expect(
      within(unknown).getByRole('img', {
        name: 'Claude e GPT-OSS, 5 horas: percentual desconhecido',
      }),
    ).toBeInTheDocument();
    expect(unknown).not.toHaveTextContent('0%');
    expect(within(unknown).getAllByText('—')).toHaveLength(2);
  });

  it('tell the user about the unknown pool in words', () => {
    ready(antigravityView(envOf(), 'ok', true));
    expect(
      screen.getByText(
        'A cota Claude e GPT-OSS não retornou dados. Ela aparece como desconhecida, não como zero.',
      ),
    ).toBeVisible();
  });

  it('say there is no quota when the source reported no windows', () => {
    ready(viewOf('codex'));
    expect(screen.getByText('Esta fonte não informou nenhuma janela de cota.')).toBeVisible();
  });
});

describe('quota pools', () => {
  it('give each pool its own card, rows and note', () => {
    ready(antigravityView(envOf(), 'ok'));
    const gemini = section('Gemini');
    expect(within(gemini).getByText('Pro e Flash dividem a mesma cota.')).toBeVisible();
    const rows = within(gemini).getAllByRole('listitem');
    expect(rows).toHaveLength(2);
    expect(within(rows[0] as HTMLElement).getByText('5 horas')).toBeVisible();
    expect(
      within(rows[0] as HTMLElement).getByRole('meter', {
        name: 'Gemini, 5 horas: 46% disponível',
      }),
    ).toBeInTheDocument();
    expect(within(rows[1] as HTMLElement).getByText('Semana')).toBeVisible();
  });
});

describe('Claude per-model weeks', () => {
  it('explain why they are missing while the usage API is off, and open its settings', async () => {
    const { user, openSettings } = ready(claudeView(envOf(), 'ok'));
    const locked = section('Semana por modelo');
    expect(
      within(locked).getByText(/Opus e Sonnet só aparecem pela API de uso da conta/),
    ).toBeVisible();
    await user.click(within(locked).getByRole('button', { name: 'Abrir configurações' }));
    expect(openSettings).toHaveBeenCalledExactlyOnceWith('claude');
  });

  it('list Opus and Sonnet once the usage API is on', () => {
    const on = settingsWith((s) => {
      s.claude.oauthUsage = true;
    });
    ready(claudeView(envOf({ settings: on }), 'ok'), on);
    const list = section('Semana por modelo');
    expect(within(list).queryByRole('button')).toBeNull();
    expect(
      within(list).getByRole('meter', { name: 'Semana por modelo, Opus: 82% disponível' }),
    ).toBeInTheDocument();
    expect(
      within(list).getByRole('meter', { name: 'Semana por modelo, Sonnet: 71% disponível' }),
    ).toBeInTheDocument();
  });
});

describe('money windows', () => {
  it('show what is left of a budget, not what was spent', () => {
    ready(litellmView(envOf(), 'ok'));
    const hero = section('Orçamento');
    expect(within(hero).getByText('disponível de US$ 50,00')).toBeInTheDocument();
    expect(within(hero).getByText('US$ 37,60 disponíveis de US$ 50,00')).toBeInTheDocument();
    expect(within(hero).getByText('reseta em 148h 00m')).toBeVisible();
    expect(meterIn(hero)).toHaveAttribute('value', '75');
  });

  it('say so when a budget has no cap', () => {
    const window = win({
      id: 'budget',
      kind: 'budget',
      windowMinutes: 43200,
      resetsInMin: 8880,
      usedPct: null,
      amount: { used: 12.4, limit: null, unit: 'usd' },
    });
    ready(viewOf('litellm', { windows: [window] }));
    expect(screen.getByText('gastos no período, sem teto')).toBeVisible();
  });
});
