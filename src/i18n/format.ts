// pt-BR formatters shared by every component. Pure functions: callers pass the clock in.
import type { AmountUnit, Fact, TimeFormat, WeekFormat } from '../lib/contract';

const LOCALE = 'pt-BR';
const MIN_PER_DAY = 1440;
const MIN_PER_WEEK = 10080;
const DAY_MS = 86_400_000;
const WEEK_DAYS = 7;
const DASH = '—';

export interface DurationFormat {
  timeFormat: TimeFormat;
  weekFormat: WeekFormat;
}

export interface DurationPart {
  n: string;
  u: string;
  /** A space separates the number from its unit ("45 min", "1,9 h"). */
  spaced: boolean;
}

const pad2 = (n: number) => String(n).padStart(2, '0');

function wholeMinutes(minutes: number): number {
  return Number.isFinite(minutes) ? Math.max(0, Math.round(minutes)) : 0;
}

/**
 * Splits a duration for display. `long` marks multi-day windows (weekly, monthly, budgets): those
 * switch to days + hours when the user asked for it, and always beyond one week.
 */
export function durationParts(minutes: number, fmt: DurationFormat, long = false): DurationPart[] {
  const total = wholeMinutes(minutes);
  const days = Math.floor(total / MIN_PER_DAY);
  const useDays = long && days > 0 && (fmt.weekFormat === 'days_hours' || total > MIN_PER_WEEK);
  if (useDays) {
    const hours = Math.floor((total % MIN_PER_DAY) / 60);
    return [
      { n: String(days), u: 'd', spaced: false },
      { n: String(hours), u: 'h', spaced: false },
    ];
  }
  const h = Math.floor(total / 60);
  const m = total % 60;
  if (fmt.timeFormat === 'colon') return [{ n: `${h}:${pad2(m)}`, u: '', spaced: false }];
  if (fmt.timeFormat === 'decimal') {
    return [{ n: (total / 60).toFixed(1).replace('.', ','), u: 'h', spaced: true }];
  }
  if (h === 0) return [{ n: String(m), u: 'min', spaced: true }];
  return [
    { n: String(h), u: 'h', spaced: false },
    { n: pad2(m), u: 'm', spaced: false },
  ];
}

export function formatDuration(minutes: number, fmt: DurationFormat, long = false): string {
  return durationParts(minutes, fmt, long)
    .map((p) => `${p.n}${p.spaced ? ' ' : ''}${p.u}`)
    .join(' ');
}

const unitFormats = {
  day: new Intl.NumberFormat(LOCALE, { style: 'unit', unit: 'day', unitDisplay: 'long' }),
  hour: new Intl.NumberFormat(LOCALE, { style: 'unit', unit: 'hour', unitDisplay: 'long' }),
  minute: new Intl.NumberFormat(LOCALE, { style: 'unit', unit: 'minute', unitDisplay: 'long' }),
};
const listFormat = new Intl.ListFormat(LOCALE, { style: 'long', type: 'conjunction' });

/** Long-form duration for screen readers: "1 hora e 53 minutos". */
export function spokenDuration(minutes: number): string {
  const total = wholeMinutes(minutes);
  const d = Math.floor(total / MIN_PER_DAY);
  const h = Math.floor((total % MIN_PER_DAY) / 60);
  const m = total % 60;
  const parts: string[] = [];
  if (d > 0) parts.push(unitFormats.day.format(d));
  if (h > 0) parts.push(unitFormats.hour.format(h));
  if (m > 0 || parts.length === 0) parts.push(unitFormats.minute.format(m));
  return listFormat.format(parts);
}

/** Available percentage for display. Unknown is a dash, never "0%". */
export function formatPercent(pct: number | null): string {
  if (pct === null || !Number.isFinite(pct)) return DASH;
  if (pct > 0 && pct < 1) return '<1%';
  return `${Math.round(pct)}%`;
}

export interface ClockMoment {
  /** Compact form for tables ("14:35", "sex 14:35"). */
  text: string;
  /** Reads after a verb: "reseta às 14:35", "reseta sex 14:35". */
  label: string;
}

const formatterCache = new Map<string, Intl.DateTimeFormat>();

function dateFormatter(kind: string, tz: string | undefined, o: Intl.DateTimeFormatOptions) {
  const key = `${kind}|${tz ?? ''}`;
  let cached = formatterCache.get(key);
  if (!cached) {
    cached = new Intl.DateTimeFormat(kind === 'ymd' ? 'en-CA' : LOCALE, { ...o, timeZone: tz });
    formatterCache.set(key, cached);
  }
  return cached;
}

function dayNumber(at: Date, tz: string | undefined): number {
  const iso = dateFormatter('ymd', tz, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(at);
  const [y, m, d] = iso.split('-').map(Number);
  return Date.UTC(y ?? 1970, (m ?? 1) - 1, d ?? 1) / DAY_MS;
}

/** Wall-clock moment of an epoch-seconds instant relative to `nowSecs`, in the local zone. */
export function formatClock(epochSecs: number, nowSecs: number, timeZone?: string): ClockMoment {
  const at = new Date(epochSecs * 1000);
  const time = dateFormatter('time', timeZone, {
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).format(at);
  const offset = dayNumber(at, timeZone) - dayNumber(new Date(nowSecs * 1000), timeZone);
  if (offset === 0) return { text: time, label: `às ${time}` };
  if (offset > 0 && offset < WEEK_DAYS) {
    const weekday = dateFormatter('weekday', timeZone, { weekday: 'short' })
      .format(at)
      .replace('.', '');
    return { text: `${weekday} ${time}`, label: `${weekday} ${time}` };
  }
  const date = dateFormatter('date', timeZone, { day: '2-digit', month: '2-digit' }).format(at);
  return { text: `${date} ${time}`, label: `em ${date} às ${time}` };
}

/** "15/01/2027 08:00" in the viewer's zone. Built from parts: the locale's own separator varies. */
export function formatDateTime(epochSecs: number, timeZone?: string): string {
  const at = new Date(epochSecs * 1000);
  if (Number.isNaN(at.getTime())) return DASH;
  const parts = dateFormatter('datetime', timeZone, {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).formatToParts(at);
  const of = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((p) => p.type === type)?.value ?? '';
  return `${of('day')}/${of('month')}/${of('year')} ${of('hour')}:${of('minute')}`;
}

/** "4 s", "4 min", "3 h", "2 d": how old a reading is. */
export function formatAge(secs: number): string {
  const s = Number.isFinite(secs) ? Math.max(0, Math.floor(secs)) : 0;
  if (s < 60) return `${s} s`;
  if (s < 3600) return `${Math.floor(s / 60)} min`;
  if (s < 86_400) return `${Math.floor(s / 3600)} h`;
  return `${Math.floor(s / 86_400)} d`;
}

/** A polling interval in seconds: "30 s", "5 min", "1 h", "90 min". */
export function formatInterval(secs: number): string {
  if (secs < 60 || secs % 60 !== 0) return `${secs} s`;
  if (secs % 3600 === 0) return `${secs / 3600} h`;
  return `${secs / 60} min`;
}

const numberFormat = new Intl.NumberFormat(LOCALE, { maximumFractionDigits: 2 });
const compactFormat = new Intl.NumberFormat(LOCALE, {
  notation: 'compact',
  compactDisplay: 'short',
  maximumFractionDigits: 1,
});

export function formatNumber(n: number): string {
  return numberFormat.format(n);
}

/** Small amounts keep four decimals so a 0.0037 cost does not read as zero. */
export function formatUsd(n: number): string {
  const abs = Math.abs(n);
  return new Intl.NumberFormat(LOCALE, {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: abs > 0 && abs < 0.01 ? 4 : 2,
  }).format(n);
}

export function formatAmountValue(value: number, unit: AmountUnit): string {
  switch (unit) {
    case 'usd':
      return formatUsd(value);
    case 'credits':
      return `${formatNumber(value)} ${value === 1 ? 'crédito' : 'créditos'}`;
    case 'requests':
      return `${formatNumber(value)} ${value === 1 ? 'requisição' : 'requisições'}`;
    case 'tokens':
      return `${compactFormat.format(value)} tokens`;
  }
}

/** A fact's value: a number with its unit, free text, or a dash when neither is present. */
export function formatFactValue(fact: Fact): string {
  if (fact.number !== null) {
    switch (fact.unit) {
      case 'usd':
        return formatUsd(fact.number);
      case 'tokens':
        return `${compactFormat.format(fact.number)} tokens`;
      case 'requests':
        return `${formatNumber(fact.number)} req.`;
      case 'credits':
        return formatAmountValue(fact.number, 'credits');
      case 'epoch_s':
        return formatDateTime(fact.number);
      case null:
        return formatNumber(fact.number);
      default:
        return `${formatNumber(fact.number)} ${fact.unit}`;
    }
  }
  return fact.text ?? DASH;
}
