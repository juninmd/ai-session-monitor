// The first-run wizard decides what each source needs and what a reading produced. Pure helpers,
// so the states that were wrong in the first draft stay wrong-proofed.
import { describe, expect, it } from 'vitest';
import type { ViewContext } from '../../app/windowModel';
import { defaultSettings, type HarnessId, type HarnessView } from '../../lib/contract';
import { problem, viewOf, type WindowSpec, windowOf } from '../../mock/fixtures';
import { startFor } from '../../mock/scenarios';
import { NOW } from '../../test/helpers';
import {
  candidateOf,
  candidates,
  keyedOf,
  outcomeOf,
  pendingOf,
  STEP_ORDER,
  withConsent,
} from './onboardingModel';

const settings = structuredClone(defaultSettings);
const ctx: ViewContext = { nowSecs: NOW, settings };
const env = { now: NOW, settings, secrets: startFor('demo').secrets };

function reading(id: HarnessId, spec: Partial<WindowSpec> = {}): HarnessView {
  return viewOf(id, {
    windows: [
      windowOf(env, {
        id: 'five_hour',
        kind: 'five_hour',
        usedPct: 48,
        windowMinutes: 300,
        resetsInMin: 60,
        ...spec,
      }),
    ],
  });
}

describe('what a source needs', () => {
  it('separates a missing key from a missing permission', () => {
    expect(candidateOf(viewOf('kimi', { state: 'needs_setup' })).need).toBe('key');
    expect(candidateOf(viewOf('kimi', { state: 'needs_setup' })).detected).toBe(false);
    expect(candidateOf(viewOf('codex', { state: 'needs_consent' })).need).toBe('consent');
    expect(candidateOf(viewOf('codex', { state: 'needs_consent' })).detected).toBe(true);
  });

  it('never claims a source responds before the first snapshot is in', () => {
    expect(candidateOf(viewOf('claude', { state: 'loading' })).detected).toBe(false);
    expect(candidateOf(viewOf('claude', { state: 'disabled' })).detected).toBe(false);
    expect(candidateOf(reading('claude')).detected).toBe(true);
  });

  it('knows which sources are unlocked by a pasted key alone', () => {
    expect(keyedOf('kimi')).toBe('kimi');
    expect(keyedOf('minimax')).toBe('minimax');
    expect(keyedOf('opencode')).toBeNull();
    expect(keyedOf('litellm')).toBeNull();
    expect(keyedOf('claude')).toBeNull();
  });

  it('collects the choices into the two things it has to ask for', () => {
    const views = [
      viewOf('kimi', { state: 'needs_setup' }),
      viewOf('codex', { state: 'needs_consent' }),
      reading('claude'),
    ];
    expect(pendingOf(['kimi', 'codex', 'claude'], views)).toEqual({
      keys: ['kimi'],
      consents: ['codex'],
    });
    expect(pendingOf(['claude'], views)).toEqual({ keys: [], consents: [] });
    expect(pendingOf([], views)).toEqual({ keys: [], consents: [] });
    expect(pendingOf(['kimi'], [])).toEqual({ keys: [], consents: [] });
  });

  it('lists one row per source and keeps the step order fixed', () => {
    expect(candidates([reading('claude'), viewOf('kimi', { state: 'needs_setup' })])).toHaveLength(
      2,
    );
    expect(STEP_ORDER).toEqual(['choose', 'connect', 'verify', 'done']);
  });
});

describe('permissions the wizard writes', () => {
  it('turns on exactly the flag that unlocks the source', () => {
    expect(withConsent(settings, 'claude').claude.shimConsent).toBe(true);
    expect(withConsent(settings, 'codex').codex.consent).toBe(true);
    expect(withConsent(settings, 'antigravity').antigravity.consent).toBe(true);
  });

  it('leaves the settings alone for a source that needs a key, not a permission', () => {
    expect(withConsent(settings, 'kimi')).toBe(settings);
    expect(withConsent(settings, 'litellm')).toBe(settings);
  });

  it('does not disturb the fields it is not about', () => {
    const next = withConsent(withConsent(settings, 'codex'), 'claude');
    expect(next.codex.consent).toBe(true);
    expect(next.claude.shimConsent).toBe(true);
    expect(next.general).toEqual(settings.general);
    expect(next.claude.enabled).toBe(settings.claude.enabled);
  });
});

describe('what a reading produced', () => {
  it('counts the quotas a ready source reported', () => {
    expect(outcomeOf(reading('claude'), ctx)).toEqual({ tone: 'ok', label: '1 cota' });
    expect(outcomeOf(viewOf('deepseek'), ctx)).toEqual({
      tone: 'ok',
      label: 'Lendo valores, sem cotas',
    });
  });

  it('reports a stale reading with its age, never with a fresh number', () => {
    const stale = { ...reading('claude'), stale: true, readAt: NOW - 23 * 60 };
    expect(outcomeOf(stale, ctx)).toEqual({ tone: 'warn', label: 'Leitura antiga há 23 min' });
  });

  it('says what is missing instead of failing silently', () => {
    expect(outcomeOf(viewOf('kimi', { state: 'needs_setup' }), ctx).label).toBe(
      'Falta a credencial',
    );
    expect(outcomeOf(viewOf('codex', { state: 'needs_consent' }), ctx).label).toBe(
      'Aguardando sua permissão',
    );
    expect(outcomeOf(viewOf('claude', { state: 'loading' }), ctx).label).toBe('Lendo…');
    expect(outcomeOf(viewOf('zai', { state: 'disabled' }), ctx).label).toBe('Desligado');
    expect(outcomeOf(viewOf('deepseek', { state: 'no_data' }), ctx).label).toBe('Ainda sem dados');
  });

  it('describes a failure in words and never pastes the raw error', () => {
    const failed = viewOf('kimi', {
      state: 'error',
      error: problem('unauthorized', { httpStatus: '401', reason: 'fab-token-123' }),
    });
    const outcome = outcomeOf(failed, ctx);
    expect(outcome.tone).toBe('error');
    expect(outcome.label).not.toContain('fab-token-123');
    expect(outcome.label.length).toBeGreaterThan(0);
  });
});
