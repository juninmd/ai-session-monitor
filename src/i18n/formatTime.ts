// Wall-clock moments in the viewer's zone and language.
import { t } from '.';
import { cached } from './intl';

const DAY_MS = 86_400_000;
const WEEK_DAYS = 7;
const DASH = '—';

export interface ClockMoment {
  /** Compact form for tables ("14:35", "sex 14:35"). */
  text: string;
  /** Reads after a verb: "reseta às 14:35", "reseta sex 14:35". */
  label: string;
}

// "ymd" is only ever an ISO-like key (2027-01-15) for counting days: en-CA gives that in every
// language. Everything the user reads follows the active locale.
function dateFormatter(kind: string, tz: string | undefined, o: Intl.DateTimeFormatOptions) {
  return cached(
    `date-${kind}|${tz ?? ''}`,
    (tag) => new Intl.DateTimeFormat(kind === 'ymd' ? 'en-CA' : tag, { ...o, timeZone: tz }),
  );
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
  if (offset === 0) return { text: time, label: t('clock.at', { time }) };
  if (offset > 0 && offset < WEEK_DAYS) {
    const weekday = dateFormatter('weekday', timeZone, { weekday: 'short' })
      .format(at)
      .replace('.', '');
    return { text: `${weekday} ${time}`, label: `${weekday} ${time}` };
  }
  const date = dateFormatter('date', timeZone, { day: '2-digit', month: '2-digit' }).format(at);
  return { text: `${date} ${time}`, label: t('clock.on_at', { date, time }) };
}

/** "15/01/2027 08:00" in the viewer's zone. Date and time are formatted apart and joined by a space:
 * the locale's own date-time separator varies ("15/01/2027, 08:00"). */
export function formatDateTime(epochSecs: number, timeZone?: string): string {
  const at = new Date(epochSecs * 1000);
  if (Number.isNaN(at.getTime())) return DASH;
  const date = dateFormatter('fulldate', timeZone, {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  }).format(at);
  const time = dateFormatter('time', timeZone, {
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).format(at);
  return `${date} ${time}`;
}
