// Runs the same vectors as crates/usage-model/tests/golden.rs against the TypeScript twin.
import { describe, expect, it } from 'vitest';
import vectors from '../../contract/status-vectors.json';
import type { HarnessView, UsageWindow, ViewState } from './contract';
import { type Pace, paceOf, statusOf, summarize } from './usage';

interface VectorWindow {
  id?: string;
  windowMinutes: number | null;
  usedPct: number | null;
  resetsInMin: number | null;
}

const NOW = vectors.now;

function build(v: VectorWindow): UsageWindow {
  return {
    id: v.id ?? 'w',
    kind: 'five_hour',
    group: null,
    label: null,
    windowMinutes: v.windowMinutes,
    usedPct: v.usedPct,
    resetsAt: v.resetsInMin === null ? null : NOW + v.resetsInMin * 60,
    amount: null,
  };
}

function flat(pace: Pace) {
  return {
    pace: pace.kind,
    runwayMin: 'runwayMin' in pace ? pace.runwayMin : null,
    earlyMin: 'earlyMin' in pace ? pace.earlyMin : null,
  };
}

describe('pace and status vectors', () => {
  it.each(vectors.windows)('$name', (c) => {
    const w = build(c);
    const got = flat(paceOf(w, NOW));
    expect(got.pace).toBe(c.expected.pace);
    for (const key of ['runwayMin', 'earlyMin'] as const) {
      const want = c.expected[key];
      if (want === null) expect(got[key]).toBeNull();
      else expect(got[key]).toBeCloseTo(want, 6);
    }
    expect(statusOf(w, NOW, c.warnAt)).toBe(c.expected.status);
  });
});

describe('summary vectors', () => {
  it.each(vectors.summaries)('$name', (c) => {
    const view: HarnessView = {
      id: 'claude',
      state: c.state as ViewState,
      source: null,
      readAt: null,
      stale: c.stale,
      plan: null,
      windows: c.windows.map(build),
      facts: [],
      notices: [],
      error: null,
      meta: {},
    };
    const got = summarize(view, NOW, c.warnAt);
    if (c.expected === null) expect(got).toBeNull();
    else {
      expect(got?.windowId).toBe(c.expected.windowId);
      expect(got?.availablePct).toBeCloseTo(c.expected.availablePct, 6);
      expect(got?.status).toBe(c.expected.status);
    }
  });
});
