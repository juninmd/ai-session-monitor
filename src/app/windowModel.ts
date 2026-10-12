// Turns a contract window into everything a component prints. Status, pace and expiry come from
// src/lib/usage.ts only (pinned to the Rust rules); nothing here recomputes them.
import type { IconName } from '../components/icons';
import { hasMessage, t } from '../i18n';
import {
  type ClockMoment,
  type DurationFormat,
  formatClock,
  formatDuration,
  formatPercent,
  spokenDuration,
} from '../i18n/format';
import type { HarnessId, Settings, UsageWindow } from '../lib/contract';
import {
  availablePct,
  isExpired,
  minutesLeft,
  type Pace,
  paceOf,
  type Status,
  statusOf,
} from '../lib/usage';
import { hasGuessedCadence } from './estimate';
import { sharePct, showsUsed, tq } from './quota';
import { specialTitle } from './windowTitles';

const MIN_PER_DAY = 1440;

/** The clock and the settings every view derives its numbers from. */
export interface ViewContext {
  nowSecs: number;
  settings: Settings;
}

export function durationFormat(settings: Settings): DurationFormat {
  return { timeFormat: settings.general.timeFormat, weekFormat: settings.general.weekFormat };
}

/** Multi-day windows read as days + hours when the user asks for it. */
export function isLongWindow(window: UsageWindow): boolean {
  if (window.windowMinutes !== null) return window.windowMinutes > MIN_PER_DAY;
  return window.kind === 'weekly' || window.kind === 'monthly' || window.kind === 'budget';
}

export interface WindowModel {
  window: UsageWindow;
  title: string;
  /** 0..100, null while unknown. */
  available: number | null;
  /** What is printed: `available`, or its complement under `showUsed`. */
  shown: number | null;
  showUsed: boolean;
  /** The 5 h / weekly name was guessed by the collector, so the title is an estimate. */
  cadenceGuessed: boolean;
  expired: boolean;
  /** Null for unknown and expired windows: they must not be coloured. */
  status: Status | null;
  pace: Pace;
  /** Minutes to the reset (0 once expired); null without a reset time. */
  leftMin: number | null;
  /** Share of the window still left, 0..100; null when the length or reset is unknown. */
  tick: number | null;
  /** The raw used percentage when it is above 100 (over budget). */
  over: number | null;
  long: boolean;
}

export function windowTitle(harness: HarnessId, window: UsageWindow): string {
  const special = specialTitle(harness, window);
  if (special !== null) return special;
  if (window.label) return window.label;
  const specific = `window.${harness}.${window.kind}`;
  return hasMessage(specific) ? t(specific) : t(`window.${window.kind}`);
}

/** Short row name inside a pool list: "5 horas", "Semana". */
export function rowTitle(window: UsageWindow): string {
  return window.label ?? t(`window.row.${window.kind}`);
}

export function modelWindow(
  harness: HarnessId,
  window: UsageWindow,
  ctx: ViewContext,
): WindowModel {
  const { nowSecs, settings } = ctx;
  const showUsed = showsUsed(settings);
  const available = availablePct(window);
  const expired = isExpired(window, nowSecs);
  const leftMin = expired ? 0 : minutesLeft(window, nowSecs);
  const hasLength = window.windowMinutes !== null && window.windowMinutes > 0;
  const tick =
    !expired && leftMin !== null && hasLength
      ? Math.min(100, Math.max(0, (leftMin / (window.windowMinutes ?? 1)) * 100))
      : null;
  return {
    window,
    title: windowTitle(harness, window),
    available,
    shown: available === null ? null : sharePct(available, showUsed),
    showUsed,
    cadenceGuessed: hasGuessedCadence(harness, window),
    expired,
    status: statusOf(window, nowSecs, settings.general.warnAtPct),
    pace: paceOf(window, nowSecs),
    leftMin,
    tick,
    over: window.usedPct !== null && window.usedPct > 100 ? window.usedPct : null,
    long: isLongWindow(window),
  };
}

export function resetMoment(m: WindowModel, ctx: ViewContext): ClockMoment | null {
  return m.window.resetsAt === null ? null : formatClock(m.window.resetsAt, ctx.nowSecs);
}

export function leftText(m: WindowModel, ctx: ViewContext): string | null {
  return m.leftMin === null
    ? null
    : formatDuration(m.leftMin, durationFormat(ctx.settings), m.long);
}

export interface PaceLine {
  icon: IconName;
  text: string;
  /** Out or at risk: shown even in compact rows. */
  urgent: boolean;
  /** A projection from the burn rate, not a reading. */
  estimate: boolean;
}

export function paceLine(m: WindowModel, ctx: ViewContext): PaceLine | null {
  if (m.expired || m.available === null) return null;
  const fmt = durationFormat(ctx.settings);
  if (m.status === 'out') {
    const when = resetMoment(m, ctx);
    const left = leftText(m, ctx);
    if (when === null || left === null)
      return { icon: 'ban', text: t('window.out'), urgent: true, estimate: false };
    return {
      icon: 'ban',
      text: t('pace.out', { when: when.label, dur: left }),
      urgent: true,
      estimate: false,
    };
  }
  const pace = m.pace;
  switch (pace.kind) {
    case 'risk': {
      const exhausted = formatClock(Math.round(ctx.nowSecs + pace.runwayMin * 60), ctx.nowSecs);
      const early = formatDuration(pace.earlyMin, fmt, m.long);
      return {
        icon: 'warn',
        text: t('pace.risk', { when: exhausted.label, early }),
        urgent: true,
        estimate: true,
      };
    }
    case 'fine':
      return { icon: 'check', text: t('pace.fine'), urgent: false, estimate: true };
    case 'early':
      return { icon: 'clock', text: t('pace.early'), urgent: false, estimate: false };
    default:
      return null;
  }
}

export function gaugeTip(m: WindowModel, ctx: ViewContext): string {
  if (m.shown === null) return t('gauge.tip_unknown');
  if (m.expired) return t('gauge.tip_expired');
  const parts = [tq('gauge.tip', m.showUsed, { pct: formatPercent(m.shown) })];
  const dur = leftText(m, ctx);
  if (dur !== null) {
    parts.push(
      m.tick !== null && m.window.windowMinutes
        ? t('gauge.tip_left', {
            dur,
            share: formatPercent(m.tick),
            window: spokenDuration(m.window.windowMinutes),
          })
        : t('gauge.tip_left_only', { dur }),
    );
  }
  return parts.join(' ');
}

export function gaugeLabel(m: WindowModel, title: string = m.title): string {
  if (m.shown === null) return t('gauge.label_unknown', { title });
  if (m.expired) return t('gauge.label_expired', { title });
  const named = m.cadenceGuessed ? `${title} (${t('estimate.word')})` : title;
  return tq('gauge.label', m.showUsed, { title: named, pct: formatPercent(m.shown) });
}
