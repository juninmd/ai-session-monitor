// Triage: orders the providers by what the user must act on and writes one plain-language
// verdict per provider. Status, pace and expiry come from src/lib/usage.ts through
// src/app/windowModel.ts; nothing here recomputes a rule.
import { describeError } from '../app/errorText';
import { summaryOf } from '../app/tabModel';
import {
  durationFormat,
  modelWindow,
  paceLine,
  type ViewContext,
  type WindowModel,
} from '../app/windowModel';
import { formatAge, formatClock, formatDuration } from '../i18n/format';
import type { HarnessId, HarnessView, UsageWindow } from '../lib/contract';
import type { Pace, Status } from '../lib/usage';

export type Tier = 'blocked' | 'risk' | 'broken' | 'loading' | 'watch' | 'fine' | 'unknown';

/** The order a person should act in, not the order the tabs are stored in. */
const TIER_ORDER: Record<Tier, number> = {
  blocked: 0,
  risk: 1,
  broken: 2,
  watch: 3,
  loading: 4,
  fine: 5,
  unknown: 6,
};

export interface TriageRow {
  view: HarnessView;
  /** The limiting window, when the view has a reading to stand on. */
  model: WindowModel | null;
  tier: Tier;
  /** One sentence, in the words a person would use. */
  verdict: string;
  /**
   * The same verdict cut for a card: "esgotado · volta 10:41", "26 h sem cota", "antigo". Empty
   * when the reading needs no remark, which is what keeps a healthy card quiet.
   */
  short: string;
  /** Epoch seconds the quota runs out, null when it will not. */
  runsOutAt: number | null;
  /** The full projection, for the tooltip and the screen reader. */
  detail: string;
}

function windowOf(view: HarnessView, id: string | undefined): UsageWindow | null {
  return view.windows.find((window) => window.id === id) ?? null;
}

/** A source that failed, or whose numbers are old, is not a source that is fine. */
function isBroken(view: HarnessView): boolean {
  return view.state === 'error' || view.stale;
}

function tierOf(view: HarnessView, known: boolean, pace: Pace, status: Status | null): Tier {
  if (isBroken(view)) return 'broken';
  // The first snapshot puts every enabled source in `loading`; saying "sem leitura" there would
  // report a gap that does not exist yet.
  if (view.state === 'loading') return 'loading';
  if (!known) return 'unknown';
  if (status === 'out') return 'blocked';
  if (pace.kind === 'risk') return 'risk';
  if (status === 'crit' || status === 'warn') return 'watch';
  return 'fine';
}

/** A provider that runs out sooner, or a failure before an old reading, comes first. */
export function compareUrgency(a: TriageRow, b: TriageRow): number {
  const byTier = TIER_ORDER[a.tier] - TIER_ORDER[b.tier];
  if (byTier !== 0) return byTier;
  if (a.tier === 'blocked' || a.tier === 'risk') {
    return (a.runsOutAt ?? Number.POSITIVE_INFINITY) - (b.runsOutAt ?? Number.POSITIVE_INFINITY);
  }
  if (a.tier === 'broken') {
    return Number(a.view.error === null) - Number(b.view.error === null);
  }
  // Readings with no percentage stay last inside their own tier.
  const share = (row: TriageRow) => row.model?.available ?? Number.POSITIVE_INFINITY;
  return share(a) - share(b);
}

/**
 * A reading at the threshold with no projection still needs a remark of its own: colour must never
 * be the only signal the card gives, and in forced-colors it would be the only one.
 */
function statusNote(status: Status | null): string {
  if (status === 'crit') return 'crítico';
  if (status === 'warn') return 'no limite';
  return '';
}

function brokenVerdict(view: HarnessView, ctx: ViewContext): string {
  if (view.error === null) {
    return view.readAt === null
      ? 'Sem leitura'
      : `Leitura antiga · há ${formatAge(ctx.nowSecs - view.readAt)}`;
  }
  return describeError(view.error, view.id).title;
}

function tierVerdict(
  view: HarnessView,
  model: WindowModel | null,
  ctx: ViewContext,
): { verdict: string; short: string; runsOutAt: number | null } {
  if (isBroken(view)) {
    return {
      verdict: brokenVerdict(view, ctx),
      short: view.error === null ? 'antigo' : 'falha',
      runsOutAt: null,
    };
  }
  if (view.state === 'loading') {
    return { verdict: 'Lendo…', short: 'lendo', runsOutAt: null };
  }
  if (model === null) {
    return { verdict: 'Sem leitura', short: 'sem leitura', runsOutAt: null };
  }
  const fmt = durationFormat(ctx.settings);
  switch (model.pace.kind) {
    case 'out': {
      const when = model.window.resetsAt;
      const back = when === null ? null : formatClock(when, ctx.nowSecs);
      return {
        verdict: `Esgotado · volta ${back === null ? 'sem data' : back.label}`,
        short: back === null ? 'esgotado' : `esgotado · volta ${back.text}`,
        runsOutAt: ctx.nowSecs,
      };
    }
    case 'risk': {
      const at = model.pace.runwayMin;
      return {
        verdict: `Esgota ${formatClock(Math.round(ctx.nowSecs + at * 60), ctx.nowSecs).label} · ${formatDuration(model.pace.earlyMin, fmt, model.long)} sem cota`,
        short: `${formatDuration(model.pace.earlyMin, fmt, model.long)} sem cota`,
        runsOutAt: ctx.nowSecs + at * 60,
      };
    }
    case 'fine':
      return { verdict: 'Dura até renovar', short: statusNote(model.status), runsOutAt: null };
    case 'early':
      return { verdict: 'Cedo para prever', short: statusNote(model.status), runsOutAt: null };
    case 'unknown':
      return {
        verdict: 'Sem base para prever',
        short: statusNote(model.status),
        runsOutAt: null,
      };
  }
}

export function triageRow(view: HarnessView, ctx: ViewContext): TriageRow {
  const summary = summaryOf(view, ctx);
  const window = summary === null ? null : windowOf(view, summary.windowId);
  const model = window === null ? null : modelWindow(view.id, window, ctx);
  const pace: Pace = model?.pace ?? { kind: 'unknown' };
  const tier = tierOf(view, summary !== null, pace, summary?.status ?? null);
  const shown = model === null || view.stale ? null : model;
  const bare = tierVerdict(view, shown, ctx);
  return {
    view,
    model: shown,
    tier,
    verdict: bare.verdict,
    short: bare.short,
    runsOutAt: bare.runsOutAt,
    detail: shown === null ? bare.verdict : (paceLine(shown, ctx)?.text ?? bare.verdict),
  };
}

export interface TriageBoard {
  /** Everything to act on, most urgent first. */
  attention: TriageRow[];
  /** Still reading: neither a gap nor a reason to act. */
  loading: TriageRow[];
  /** Readings that are fine where they are. */
  fine: TriageRow[];
  /** No usable reading: nothing to compare, nothing to alarm about. */
  unknown: TriageRow[];
  /** Every row, in the order the list prints them. */
  rows: TriageRow[];
}

export function triageOf(views: readonly HarnessView[], ctx: ViewContext): TriageBoard {
  const pinned = ctx.settings.general.pinned;
  // Pinned providers lead in pin order; urgency still orders everything after them.
  const pinRank = (row: TriageRow) => {
    const at = pinned.indexOf(row.view.id);
    return at === -1 ? Number.POSITIVE_INFINITY : at;
  };
  const rows = views
    .map((view) => triageRow(view, ctx))
    .sort((a, b) => {
      const ra = pinRank(a);
      const rb = pinRank(b);
      if (ra !== rb) return ra < rb ? -1 : 1;
      return compareUrgency(a, b);
    });
  const inTier = (...tiers: Tier[]) => rows.filter((row) => tiers.includes(row.tier));
  return {
    attention: inTier('blocked', 'risk', 'broken', 'watch'),
    loading: inTier('loading'),
    fine: inTier('fine'),
    unknown: inTier('unknown'),
    rows,
  };
}

export interface Headline {
  tone: 'ok' | 'attention';
  text: string;
  /** The providers the headline is about, for the jump links. */
  ids: HarnessId[];
}

function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

/** The two-second answer: does anything run out before it renews? */
export function headlineOf(triage: TriageBoard): Headline {
  const runsOut = triage.attention.filter((row) => row.tier === 'blocked' || row.tier === 'risk');
  if (runsOut.length > 0) {
    const first = runsOut[0];
    if (first === undefined) return { tone: 'ok', text: 'Nada esgota antes de renovar', ids: [] };
    const rest = runsOut.length - 1;
    return {
      tone: 'attention',
      text:
        rest === 0
          ? first.verdict
          : `${first.verdict} · mais ${count(rest, 'provider', 'providers')}`,
      ids: runsOut.map((row) => row.view.id),
    };
  }
  if (triage.attention.length > 0) {
    return {
      tone: 'attention',
      text: `${count(triage.attention.length, 'provider precisa', 'providers precisam')} de atenção`,
      ids: triage.attention.map((row) => row.view.id),
    };
  }
  const reading = triage.loading.length;
  if (triage.fine.length === 0 && triage.unknown.length === 0 && reading > 0) {
    return { tone: 'ok', text: `Lendo ${count(reading, 'fonte', 'fontes')}…`, ids: [] };
  }
  const trailing = [
    triage.unknown.length > 0 ? `${triage.unknown.length} sem leitura` : null,
    reading > 0 ? `${reading} lendo` : null,
  ].filter((part): part is string => part !== null);
  return {
    tone: 'ok',
    text:
      trailing.length === 0
        ? 'Nada esgota antes de renovar'
        : `Nada esgota antes de renovar · ${trailing.join(' · ')}`,
    ids: [],
  };
}
