import { describe, expect, it } from 'vitest';
import { ctxOf, settingsWith, win } from '../test/helpers';
import {
  gaugeLabel,
  gaugeTip,
  isLongWindow,
  leftText,
  modelWindow,
  paceLine,
  resetMoment,
  rowTitle,
  windowTitle,
} from './windowModel';

const ctx = ctxOf();
const model = (spec = {}, c = ctx) => modelWindow('claude', win(spec), c);

describe('modelWindow', () => {
  it('derives availability, status and the share of the window left from a healthy window', () => {
    const m = model();
    expect(m.available).toBe(52);
    expect(m.expired).toBe(false);
    expect(m.status).toBe('ok');
    expect(m.leftMin).toBeCloseTo(113);
    expect(m.tick).toBeCloseTo((113 / 300) * 100);
    expect(m.over).toBeNull();
    expect(m.long).toBe(false);
  });

  it('treats a window whose reset has passed as reset, not as an old reading', () => {
    const m = model({ resetsInMin: -10 });
    expect(m.expired).toBe(true);
    expect(m.status).toBeNull();
    expect(m.tick).toBeNull();
    expect(m.leftMin).toBe(0);
    expect(m.pace.kind).toBe('unknown');
    expect(paceLine(m, ctx)).toBeNull();
  });

  it('keeps an unknown percentage unknown instead of turning it into zero', () => {
    const m = model({ usedPct: null });
    expect(m.available).toBeNull();
    expect(m.status).toBeNull();
    expect(paceLine(m, ctx)).toBeNull();
    expect(m.over).toBeNull();
  });

  it('reports usage above the limit and clamps availability to zero', () => {
    const m = model({ usedPct: 130 });
    expect(m.available).toBe(0);
    expect(m.over).toBe(130);
    expect(m.status).toBe('out');
  });

  it('has no tick when the window length or the reset is unknown', () => {
    expect(model({ windowMinutes: null }).tick).toBeNull();
    expect(model({ resetsInMin: null }).tick).toBeNull();
    expect(model({ resetsInMin: null }).leftMin).toBeNull();
  });

  it('reads the warning threshold from the settings, not from a constant', () => {
    const spec = { usedPct: 70, resetsInMin: 20, windowMinutes: 300 };
    const strict = settingsWith((s) => {
      s.general.warnAtPct = 35;
    });
    const relaxed = settingsWith((s) => {
      s.general.warnAtPct = 10;
    });
    expect(model(spec, ctxOf(strict)).status).toBe('warn');
    expect(model(spec, ctxOf(relaxed)).status).toBe('ok');
  });

  it('is over the limit only above 100 percent: exactly 100 is exhausted, not over', () => {
    expect(model({ usedPct: 100 }).over).toBeNull();
    expect(model({ usedPct: 100 }).status).toBe('out');
    expect(model({ usedPct: 100.5 }).over).toBe(100.5);
  });

  it('never puts the tick beyond the whole window when the reset is farther than its length', () => {
    expect(model({ resetsInMin: 400, windowMinutes: 300 }).tick).toBe(100);
  });
});

describe('isLongWindow', () => {
  it.each([
    [300, 'five_hour', false],
    [1440, 'daily', false],
    [1441, 'other', true],
    [10080, 'weekly', true],
    [null, 'weekly', true],
    [null, 'monthly', true],
    [null, 'budget', true],
    [null, 'five_hour', false],
    [null, 'daily', false],
  ] as const)('%s min of kind %s is long: %s', (windowMinutes, kind, expected) => {
    expect(isLongWindow(win({ windowMinutes, kind }))).toBe(expected);
  });
});

describe('titles', () => {
  it('prefers a label the source provided', () => {
    const labelled = { ...win(), label: 'Pro e Flash' };
    expect(windowTitle('antigravity', labelled)).toBe('Pro e Flash');
    expect(rowTitle(labelled)).toBe('Pro e Flash');
  });

  it('uses the Claude wording only for Claude', () => {
    expect(windowTitle('claude', win())).toBe('Sessão de 5 horas');
    expect(windowTitle('codex', win())).toBe('Janela de 5 horas');
    expect(windowTitle('claude', win({ kind: 'weekly' }))).toBe('Semana, todos os modelos');
    expect(windowTitle('opencode', win({ kind: 'weekly' }))).toBe('Semana');
  });

  it('shortens titles inside a pool list', () => {
    expect(rowTitle(win())).toBe('5 horas');
    expect(rowTitle(win({ kind: 'monthly' }))).toBe('Mês');
  });
});

describe('titles of the windows whose id says more than their kind', () => {
  const named = (spec: Parameters<typeof win>[0], label: string | null = null) => ({
    ...win(spec),
    label,
  });

  it.each([
    // Codex: a length that matches no known cadence arrives as primary/secondary, kind other.
    ['codex', named({ id: 'primary', kind: 'other', windowMinutes: 2880 }), 'Janela de 2 dias'],
    ['codex', named({ id: 'secondary', kind: 'other', windowMinutes: 20160 }), 'Janela de 14 dias'],
    ['codex', named({ id: 'primary', kind: 'other', windowMinutes: null }), 'Janela principal'],
    ['codex', named({ id: 'secondary', kind: 'other', windowMinutes: null }), 'Janela secundária'],
    ['codex', named({ id: 'tertiary', kind: 'other', windowMinutes: null }), 'Janela'],
    ['codex', named({ id: 'five_hour', kind: 'five_hour' }), 'Janela de 5 horas'],
    // LiteLLM: the headline carries its record's alias, the concurrent caps a raw duration.
    ['litellm', named({ id: 'budget', kind: 'budget' }), 'Orçamento'],
    ['litellm', named({ id: 'budget', kind: 'budget' }, 'squad-alfa'), 'Orçamento: squad-alfa'],
    [
      'litellm',
      named({ id: 'limit:1h', kind: 'budget', windowMinutes: 60 }, '1h'),
      'Orçamento de 1 hora',
    ],
    [
      'litellm',
      named({ id: 'limit:30d', kind: 'budget', windowMinutes: 43200 }, '30d'),
      'Orçamento de 30 dias',
    ],
    [
      'litellm',
      named({ id: 'limit:weekly', kind: 'budget', windowMinutes: 10080 }, 'weekly'),
      'Orçamento de 7 dias',
    ],
    [
      'litellm',
      named({ id: 'limit:90m', kind: 'budget', windowMinutes: 90 }, '90m'),
      'Orçamento de 1 hora e 30 minutos',
    ],
    [
      'litellm',
      named({ id: 'limit:fortnight', kind: 'budget', windowMinutes: null }, 'fortnight'),
      'Orçamento: fortnight',
    ],
    ['litellm', named({ id: 'model:gpt-x', kind: 'budget' }, 'gpt-x'), 'gpt-x'],
    // Claude: the gateway spend cap is money, not a session.
    ['claude', named({ id: 'spend_limit', kind: 'budget' }), 'Limite de gasto'],
  ] as const)('%s window %j reads "%s"', (harness, window, expected) => {
    expect(windowTitle(harness, window)).toBe(expected);
  });

  it('never prints the raw id of a window nobody has a rule for', () => {
    const title = windowTitle(
      'litellm',
      named({ id: 'limit:???', kind: 'budget', windowMinutes: null }),
    );
    expect(title).toBe('Orçamento');
    expect(title).not.toContain('limit:');
  });

  it('leaves another source alone even when its ids look alike', () => {
    expect(windowTitle('codex', named({ id: 'budget', kind: 'budget' }, 'alias'))).toBe('alias');
    expect(
      windowTitle('opencode', named({ id: 'primary', kind: 'other', windowMinutes: 2880 })),
    ).toBe('Janela');
  });
});

describe('time left', () => {
  it('formats the countdown with the user time format', () => {
    const colon = ctxOf(
      settingsWith((s) => {
        s.general.timeFormat = 'colon';
      }),
    );
    expect(leftText(model(), ctx)).toBe('1h 53m');
    expect(leftText(modelWindow('claude', win(), colon), colon)).toBe('1:53');
  });

  it('switches multi-day windows to days and hours when asked', () => {
    const days = ctxOf(
      settingsWith((s) => {
        s.general.weekFormat = 'days_hours';
      }),
    );
    const weekly = { kind: 'weekly', windowMinutes: 10080, resetsInMin: 6432 } as const;
    expect(leftText(modelWindow('claude', win(weekly), ctx), ctx)).toBe('107h 12m');
    expect(leftText(modelWindow('claude', win(weekly), days), days)).toBe('4d 11h');
  });

  it('has no text without a reset time', () => {
    expect(leftText(model({ resetsInMin: null }), ctx)).toBeNull();
    expect(resetMoment(model({ resetsInMin: null }), ctx)).toBeNull();
  });

  it('places the reset on the clock', () => {
    expect(resetMoment(model(), ctx)?.label).toMatch(/^às \d{2}:\d{2}$/);
  });
});

describe('paceLine', () => {
  it('says the window lasts when the pace is sustainable', () => {
    expect(paceLine(model(), ctx)).toEqual({
      icon: 'check',
      text: 'No ritmo atual, dura além do reset.',
      urgent: false,
    });
  });

  it('warns when the window runs dry before it resets', () => {
    const line = paceLine(model({ usedPct: 84 }), ctx);
    expect(line?.icon).toBe('warn');
    expect(line?.urgent).toBe(true);
    expect(line?.text).toMatch(
      /^No ritmo atual, esgota às \d{2}:\d{2} \(1h 17m antes do reset\)\.$/,
    );
  });

  it('announces when an exhausted window comes back', () => {
    const line = paceLine(model({ usedPct: 100, resetsInMin: 47 }), ctx);
    expect(line?.icon).toBe('ban');
    expect(line?.urgent).toBe(true);
    expect(line?.text).toMatch(/^Esgotado\. Volta às \d{2}:\d{2}, em 47 min\.$/);
  });

  it('still says exhausted when the source gave no reset time', () => {
    expect(paceLine(model({ usedPct: 100, resetsInMin: null }), ctx)?.text).toBe('Esgotado');
  });

  it('does not extrapolate a brand-new window', () => {
    const line = paceLine(model({ usedPct: 10, resetsInMin: 295 }), ctx);
    expect(line?.text).toBe('Janela recém-iniciada, ritmo ainda sem base.');
  });

  it('has nothing to say without a window length', () => {
    expect(paceLine(model({ windowMinutes: null }), ctx)).toBeNull();
  });
});

describe('gauge text', () => {
  it('names the window and its available share', () => {
    const m = model();
    expect(gaugeLabel(m)).toBe('Sessão de 5 horas: 52% disponível');
    expect(gaugeLabel(m, 'Gemini, 5 horas')).toBe('Gemini, 5 horas: 52% disponível');
  });

  it('explains the tick in the tooltip', () => {
    expect(gaugeTip(model(), ctx)).toBe(
      '52% disponível. Faltam 1h 53m (38% da janela de 5 horas).',
    );
    expect(gaugeTip(model({ windowMinutes: null }), ctx)).toBe('52% disponível. Faltam 1h 53m.');
    expect(gaugeTip(model({ resetsInMin: null }), ctx)).toBe('52% disponível.');
  });

  it('says reset or unknown, never 0%', () => {
    const expired = model({ resetsInMin: -1 });
    const unknown = model({ usedPct: null });
    expect(gaugeLabel(expired)).toBe('Sessão de 5 horas: janela reiniciada');
    expect(gaugeTip(expired, ctx)).toBe('Janela reiniciada; aguardando nova leitura.');
    expect(gaugeLabel(unknown)).toBe('Sessão de 5 horas: percentual desconhecido');
    expect(gaugeTip(unknown, ctx)).toBe('Percentual desconhecido.');
    expect(`${gaugeLabel(unknown)}${gaugeTip(unknown, ctx)}`).not.toContain('0%');
  });
});
