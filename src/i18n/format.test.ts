import { describe, expect, it } from 'vitest';
import type { Fact, TimeFormat, WeekFormat } from '../lib/contract';
import {
  type DurationFormat,
  durationParts,
  formatAge,
  formatAmountValue,
  formatClock,
  formatDateTime,
  formatDuration,
  formatFactValue,
  formatInterval,
  formatNumber,
  formatPercent,
  formatUsd,
  spokenDuration,
} from './format';

// Intl separates a currency or unit from its number with a no-break space.
const plain = (text: string) => text.replace(/[  ]/g, ' ');

const fmt = (timeFormat: TimeFormat, weekFormat: WeekFormat = 'hours'): DurationFormat => ({
  timeFormat,
  weekFormat,
});

describe('formatDuration', () => {
  it.each([
    // [minutes, timeFormat, expected]
    [0, 'hm', '0 min'],
    [45, 'hm', '45 min'],
    [44.6, 'hm', '45 min'],
    [60, 'hm', '1h 00m'],
    [113, 'hm', '1h 53m'],
    [1439, 'hm', '23h 59m'],
    [0, 'colon', '0:00'],
    [45, 'colon', '0:45'],
    [113, 'colon', '1:53'],
    [30, 'decimal', '0,5 h'],
    [90, 'decimal', '1,5 h'],
    [113, 'decimal', '1,9 h'],
    // x.25 h is an exact tie that toFixed rounds up; the Rust tooltip must agree (tooltip/tests.rs)
    [15, 'decimal', '0,3 h'],
    [75, 'decimal', '1,3 h'],
    [135, 'decimal', '2,3 h'],
    [45, 'decimal', '0,8 h'],
    [9, 'decimal', '0,1 h'],
  ] as const)('%s min as %s reads "%s"', (minutes, timeFormat, expected) => {
    expect(formatDuration(minutes, fmt(timeFormat))).toBe(expected);
  });

  it.each([
    // [minutes, weekFormat, expected]: a week or less stays in hours unless days were asked for
    [6432, 'hours', '107h 12m'],
    [6432, 'days_hours', '4d 11h'],
    [10080, 'hours', '168h 00m'],
    [10080, 'days_hours', '7d 0h'],
    [11000, 'hours', '7d 15h'],
    [600, 'days_hours', '10h 00m'],
  ] as const)('long window of %s min with %s reads "%s"', (minutes, weekFormat, expected) => {
    expect(formatDuration(minutes, fmt('hm', weekFormat), true)).toBe(expected);
  });

  it('never switches a short window to days', () => {
    expect(formatDuration(2000, fmt('hm', 'days_hours'), false)).toBe('33h 20m');
  });

  it.each([[Number.NaN], [Number.POSITIVE_INFINITY], [-5]])(
    'clamps %s to zero instead of printing nonsense',
    (minutes) => {
      expect(formatDuration(minutes, fmt('hm'))).toBe('0 min');
    },
  );

  it('splits the number from its unit so the hero can size them apart', () => {
    expect(durationParts(113, fmt('hm'))).toEqual([
      { n: '1', u: 'h', spaced: false },
      { n: '53', u: 'm', spaced: false },
    ]);
    expect(durationParts(45, fmt('hm'))).toEqual([{ n: '45', u: 'min', spaced: true }]);
    expect(durationParts(113, fmt('colon'))).toEqual([{ n: '1:53', u: '', spaced: false }]);
  });
});

describe('spokenDuration', () => {
  it.each([
    [113, '1 hora e 53 minutos'],
    [60, '1 hora'],
    [1, '1 minuto'],
    [0, '0 minuto'],
    [1440, '1 dia'],
    [1500, '1 dia e 1 hora'],
    [1565, '1 dia, 2 horas e 5 minutos'],
  ])('%s min is read as "%s"', (minutes, expected) => {
    expect(spokenDuration(minutes)).toBe(expected);
  });
});

describe('formatPercent', () => {
  it.each([
    [null, '—'],
    [Number.NaN, '—'],
    [0, '0%'],
    [0.4, '<1%'],
    [1, '1%'],
    [52.4, '52%'],
    [52.5, '53%'],
    [100, '100%'],
  ])('%s reads "%s"', (pct, expected) => {
    expect(formatPercent(pct)).toBe(expected);
  });

  it('keeps unknown apart from zero', () => {
    expect(formatPercent(null)).not.toBe(formatPercent(0));
  });
});

describe('formatAge', () => {
  it.each([
    [0, '0 s'],
    [59, '59 s'],
    [60, '1 min'],
    [3599, '59 min'],
    [3600, '1 h'],
    [86_399, '23 h'],
    [86_400, '1 d'],
    [-30, '0 s'],
    [Number.NaN, '0 s'],
  ])('%s s reads "%s"', (secs, expected) => {
    expect(formatAge(secs)).toBe(expected);
  });
});

describe('formatInterval', () => {
  it.each([
    [30, '30 s'],
    [60, '1 min'],
    [90, '90 s'],
    [300, '5 min'],
    [3600, '1 h'],
    [5400, '90 min'],
  ])('%s s reads "%s"', (secs, expected) => {
    expect(formatInterval(secs)).toBe(expected);
  });
});

describe('numbers and amounts', () => {
  it.each([
    [0, 'US$ 0,00'],
    [0.42, 'US$ 0,42'],
    [3.18, 'US$ 3,18'],
    [0.0037, 'US$ 0,0037'],
    [1234.5, 'US$ 1.234,50'],
  ])('formatUsd(%s) is "%s"', (value, expected) => {
    expect(plain(formatUsd(value))).toBe(expected);
  });

  it('formats plain numbers with pt-BR separators', () => {
    expect(formatNumber(1234.567)).toBe('1.234,57');
  });

  it.each([
    [37.6, 'usd', 'US$ 37,60'],
    [1, 'credits', '1 crédito'],
    [120, 'credits', '120 créditos'],
    [1, 'requests', '1 requisição'],
    [60, 'requests', '60 requisições'],
    [950, 'tokens', '950 tokens'],
    [184_000, 'tokens', '184 mil tokens'],
  ] as const)('amount %s %s reads "%s"', (value, unit, expected) => {
    expect(plain(formatAmountValue(value, unit))).toBe(expected);
  });

  it.each([
    [{ number: 0.42, text: null, unit: 'usd' }, 'US$ 0,42'],
    [{ number: 200_000, text: null, unit: 'tokens' }, '200 mil tokens'],
    [{ number: 60, text: null, unit: 'requests' }, '60 req.'],
    [{ number: 120, text: null, unit: 'credits' }, '120 créditos'],
    [{ number: 12, text: null, unit: null }, '12'],
    [{ number: 3, text: null, unit: 'sessões' }, '3 sessões'],
    [{ number: null, text: 'Team', unit: null }, 'Team'],
    [{ number: null, text: null, unit: 'usd' }, '—'],
  ] as const)('fact %j reads "%s"', (partial, expected) => {
    const fact: Fact = { id: 'fabricated', ...partial };
    expect(plain(formatFactValue(fact))).toBe(expected);
  });
});

describe('formatDateTime', () => {
  const moment = Date.UTC(2027, 0, 15, 8, 0, 0) / 1000;

  it.each([
    [moment, 'UTC', '15/01/2027 08:00'],
    [moment, 'America/Sao_Paulo', '15/01/2027 05:00'],
    [Date.UTC(2027, 11, 31, 23, 59, 59) / 1000, 'UTC', '31/12/2027 23:59'],
    // Midnight is 00:00, never "24:00".
    [Date.UTC(2027, 0, 1, 0, 0, 0) / 1000, 'UTC', '01/01/2027 00:00'],
    // The viewer's zone decides the calendar day too.
    [Date.UTC(2027, 0, 1, 1, 30, 0) / 1000, 'America/Sao_Paulo', '31/12/2026 22:30'],
  ])('reads %s in %s as "%s"', (epoch, timeZone, expected) => {
    expect(formatDateTime(epoch, timeZone)).toBe(expected);
  });

  it.each([Number.NaN, Number.POSITIVE_INFINITY, 1e20, -1e20])(
    'is a dash, not "Invalid Date" or a crash, for %s',
    (epoch) => {
      expect(formatDateTime(epoch, 'UTC')).toBe('—');
    },
  );

  it('renders an epoch_s fact as a date in the local zone, never as a raw number', () => {
    const fact: Fact = { id: 'expires', number: moment, text: null, unit: 'epoch_s' };
    expect(formatFactValue(fact)).toMatch(/^\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}$/);
    expect(formatFactValue(fact)).not.toContain(String(moment));
    expect(formatFactValue({ ...fact, number: 1e20 })).toBe('—');
  });

  it('keeps an epoch_s fact without a number unknown', () => {
    expect(formatFactValue({ id: 'expires', number: null, text: null, unit: 'epoch_s' })).toBe('—');
    expect(
      formatFactValue({ id: 'expires', number: null, text: 'sem data', unit: 'epoch_s' }),
    ).toBe('sem data');
  });
});

describe('formatClock', () => {
  const now = Date.UTC(2027, 0, 15, 8, 0, 0) / 1000;
  const at = (day: number, hour: number, minute: number) =>
    Date.UTC(2027, 0, day, hour, minute, 0) / 1000;

  it.each([
    // [reset moment, text, label]
    [at(15, 14, 35), '14:35', 'às 14:35'],
    [at(15, 5, 10), '05:10', 'às 05:10'],
    [at(16, 3, 5), 'sáb 03:05', 'sáb 03:05'],
    [at(16, 0, 5), 'sáb 00:05', 'sáb 00:05'],
    [at(21, 9, 0), 'qui 09:00', 'qui 09:00'],
    [at(22, 9, 0), '22/01 09:00', 'em 22/01 às 09:00'],
    [at(14, 20, 0), '14/01 20:00', 'em 14/01 às 20:00'],
  ])('reads %s as "%s"', (epoch, text, label) => {
    expect(formatClock(epoch, now, 'UTC')).toEqual({ text, label });
  });

  it('follows the viewer time zone when deciding which day a moment is on', () => {
    const lateEvening = at(16, 1, 0);
    expect(formatClock(lateEvening, now, 'UTC').label).toBe('sáb 01:00');
    // In Sao Paulo (UTC-3) that moment is still the evening of the 15th.
    expect(formatClock(lateEvening, now, 'America/Sao_Paulo')).toEqual({
      text: '22:00',
      label: 'às 22:00',
    });
  });

  describe('across a month end', () => {
    const utc = (year: number, month: number, day: number, hour: number) =>
      Date.UTC(year, month, day, hour, 0, 0) / 1000;

    it('reads the first morning of the next month as a weekday, not as a date', () => {
      // Sunday 31 Jan 22:00 -> Monday 1 Feb 02:00: the next calendar day, even though 1 < 31.
      expect(formatClock(utc(2027, 1, 1, 2), utc(2027, 0, 31, 22), 'UTC')).toEqual({
        text: 'seg 02:00',
        label: 'seg 02:00',
      });
    });

    it('counts the day after a 31-day month and after New Year as tomorrow too', () => {
      expect(formatClock(utc(2027, 3, 1, 2), utc(2027, 2, 31, 22), 'UTC').label).toBe('qui 02:00');
      expect(formatClock(utc(2028, 0, 1, 2), utc(2027, 11, 31, 22), 'UTC').label).toBe('sáb 02:00');
    });
  });
});
