// Locale-aware formatters shared by every component. Pure functions: callers pass the clock in.
// Every formatter follows getLocale() at call time (the language can change while the app runs),
// always with Latin digits, and reads its unit words from the catalog (`duration.*`, `amount.*`).

import type { AmountUnit, Fact, TimeFormat, WeekFormat } from '../lib/contract';
import { t, tn } from '.';
import { formatDateTime } from './formatTime';
import { cached, decimalSeparator } from './intl';
import { getLocale } from './locale';

export { type ClockMoment, formatClock, formatDateTime } from './formatTime';

const MIN_PER_DAY = 1440;
const MIN_PER_WEEK = 10080;
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
      { n: String(days), u: t('duration.d'), spaced: false },
      { n: String(hours), u: t('duration.h'), spaced: false },
    ];
  }
  const h = Math.floor(total / 60);
  const m = total % 60;
  if (fmt.timeFormat === 'colon') return [{ n: `${h}:${pad2(m)}`, u: '', spaced: false }];
  if (fmt.timeFormat === 'decimal') {
    // toFixed (not Intl) so ties round exactly as the Rust tooltip does; only the separator is local.
    const n = (total / 60).toFixed(1).replace('.', decimalSeparator());
    return [{ n, u: t('duration.h'), spaced: true }];
  }
  if (h === 0) return [{ n: String(m), u: t('duration.min'), spaced: true }];
  return [
    { n: String(h), u: t('duration.h'), spaced: false },
    { n: pad2(m), u: t('duration.m'), spaced: false },
  ];
}

export function formatDuration(minutes: number, fmt: DurationFormat, long = false): string {
  return durationParts(minutes, fmt, long)
    .map((p) => `${p.n}${p.spaced ? ' ' : ''}${p.u}`)
    .join(' ');
}

type Unit = 'day' | 'hour' | 'minute';

const unitFormat = (unit: Unit) =>
  cached(
    `unit-${unit}`,
    (tag) => new Intl.NumberFormat(tag, { style: 'unit', unit, unitDisplay: 'long' }),
  );

/** Items joined the way the language does it: "80% e 95%", "80% and 95%". */
export function formatList(items: readonly string[]): string {
  return cached(
    'list',
    (tag) => new Intl.ListFormat(tag, { style: 'long', type: 'conjunction' }),
  ).format(items);
}

/** Long-form duration for screen readers: "1 hora e 53 minutos". */
export function spokenDuration(minutes: number): string {
  const total = wholeMinutes(minutes);
  const d = Math.floor(total / MIN_PER_DAY);
  const h = Math.floor((total % MIN_PER_DAY) / 60);
  const m = total % 60;
  const parts: string[] = [];
  if (d > 0) parts.push(unitFormat('day').format(d));
  if (h > 0) parts.push(unitFormat('hour').format(h));
  if (m > 0 || parts.length === 0) parts.push(unitFormat('minute').format(m));
  return formatList(parts);
}

/** Available percentage for display. Unknown is a dash, never "0%". */
export function formatPercent(pct: number | null): string {
  if (pct === null || !Number.isFinite(pct)) return DASH;
  if (pct > 0 && pct < 1) return '<1%';
  return `${Math.round(pct)}%`;
}

/** "4 s", "4 min", "3 h", "2 d": how old a reading is. */
export function formatAge(secs: number): string {
  const s = Number.isFinite(secs) ? Math.max(0, Math.floor(secs)) : 0;
  if (s < 60) return withUnit(s, 'duration.s');
  if (s < 3600) return withUnit(Math.floor(s / 60), 'duration.min');
  if (s < 86_400) return withUnit(Math.floor(s / 3600), 'duration.h');
  return withUnit(Math.floor(s / 86_400), 'duration.d');
}

const withUnit = (n: number, unit: 'duration.s' | 'duration.min' | 'duration.h' | 'duration.d') =>
  `${n} ${t(unit)}`;

/** A polling interval in seconds: "30 s", "5 min", "1 h", "90 min". */
export function formatInterval(secs: number): string {
  if (secs < 60 || secs % 60 !== 0) return withUnit(secs, 'duration.s');
  if (secs % 3600 === 0) return withUnit(secs / 3600, 'duration.h');
  return withUnit(secs / 60, 'duration.min');
}

/** A second count with its unit ("30 s"): the polling and timeout labels. */
export const formatSeconds = (secs: number): string => withUnit(secs, 'duration.s');

const numberFormat = () =>
  cached('number', (tag) => new Intl.NumberFormat(tag, { maximumFractionDigits: 2 }));
const compactFormat = () =>
  cached(
    'compact',
    (tag) =>
      new Intl.NumberFormat(tag, {
        notation: 'compact',
        compactDisplay: 'short',
        maximumFractionDigits: 1,
      }),
  );

export function formatNumber(n: number): string {
  return numberFormat().format(n);
}

/** Download progress: "3,5 MB". */
export function formatMegabytes(bytes: number): string {
  const mb = cached('megabytes', (tag) => new Intl.NumberFormat(tag, { maximumFractionDigits: 1 }));
  return `${mb.format(bytes / 1_048_576)} MB`;
}

/** Small amounts keep four decimals so a 0.0037 cost does not read as zero. */
export function formatUsd(n: number): string {
  const abs = Math.abs(n);
  const digits = abs > 0 && abs < 0.01 ? 4 : 2;
  return cached(
    `usd-${digits}`,
    (tag) =>
      new Intl.NumberFormat(tag, {
        style: 'currency',
        currency: 'USD',
        minimumFractionDigits: 2,
        maximumFractionDigits: digits,
      }),
  ).format(n);
}

/**
 * Plural by the language's own rules. Portuguese reads 0 and 1.5 as "one" in CLDR, but this UI has
 * always said "0 créditos": only exactly 1 is singular there.
 */
function counted(base: string, n: number, value: string): string {
  const count = getLocale() === 'pt-BR' && n !== 1 ? 2 : n;
  return tn(base, count, { value });
}

export function formatAmountValue(value: number, unit: AmountUnit): string {
  switch (unit) {
    case 'usd':
      return formatUsd(value);
    case 'credits':
      return counted('amount.credits', value, formatNumber(value));
    case 'requests':
      return counted('amount.requests', value, formatNumber(value));
    case 'tokens':
      return t('amount.tokens', { value: compactFormat().format(value) });
  }
}

/** Yuan, as DeepSeek reports it; kept apart from dollars, never converted. */
function formatCny(n: number): string {
  return cached(
    'cny',
    (tag) => new Intl.NumberFormat(tag, { style: 'currency', currency: 'CNY' }),
  ).format(n);
}

/** Cursor's `unlimited` and Copilot's `*_unlimited` carry the words `true` / `unlimited`: localized here. */
function isUnlimitedFact(fact: Fact): boolean {
  const named = fact.id === 'unlimited' || fact.id.endsWith('_unlimited');
  return named && (fact.text === 'true' || fact.text === 'unlimited');
}

/** A fact's value: a number with its unit, free text, or a dash when neither is present. */
export function formatFactValue(fact: Fact): string {
  if (fact.number !== null) {
    switch (fact.unit) {
      case 'usd':
        return formatUsd(fact.number);
      case 'cny':
        return formatCny(fact.number);
      case 'tokens':
        return t('amount.tokens', { value: compactFormat().format(fact.number) });
      case 'requests':
        return t('amount.req_short', { value: formatNumber(fact.number) });
      case 'credits':
        return formatAmountValue(fact.number, 'credits');
      case 'epoch_s':
        return formatDateTime(fact.number);
      case 'resets':
        return counted('amount.resets', fact.number, formatNumber(fact.number));
      case null:
        return formatNumber(fact.number);
      default:
        return `${formatNumber(fact.number)} ${fact.unit}`;
    }
  }
  if (isUnlimitedFact(fact)) return t('fact.value.unlimited');
  return fact.text ?? DASH;
}
