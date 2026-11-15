// A card must say why its source is not fine: the reason of a failure (a login that expired, a
// command that was replaced), never a bare "failed", and the last numbers of a stale view rather
// than a blank. Pure functions on fabricated views.
import { describe, expect, it } from 'vitest';
import type { ViewContext } from '../app/model/windowModel';
import { describeError } from '../app/text/errorText';
import { noticeText } from '../app/text/noticeText';
import { t } from '../i18n';
import { defaultSettings, type HarnessView } from '../lib/contract';
import { asStale, failed, notice, problem, viewOf, windowOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';
import { NOW } from '../test/helpers';
import { activeViews, lostSetup } from './model';
import { compactQuota } from './quotaModel';
import { brokenHint, readAge, signInNote } from './tileNote';
import { triageRow } from './triageModel';

const settings = structuredClone(defaultSettings);
settings.claude.enabled = true;
const ctx: ViewContext = { nowSecs: NOW, settings };
const env = { now: NOW, settings, secrets: startFor('demo').secrets };

const weekly = windowOf(env, {
  id: 'weekly',
  kind: 'weekly',
  usedPct: 30,
  windowMinutes: 10080,
  resetsInMin: 5000,
});
const ready = (patch: Partial<HarnessView> = {}) =>
  viewOf('claude', { windows: [weekly], readAt: NOW, ...patch });

describe('a failed source', () => {
  it('names the reason on the card and what to do about it', () => {
    const view = failed('claude', problem('oauth_expired'));
    const text = describeError(view.error ?? problem('internal'), 'claude');
    const row = triageRow(view, ctx);
    expect(row.tier).toBe('broken');
    expect(row.short).toBe(text.title);
    expect(row.short).not.toBe(t('triage.short.failed'));
    expect(brokenHint(view)).toBe(text.body);
  });

  it('keeps the last numbers of a stale view, muted, with the reason of the failed refresh', () => {
    const view = asStale(ready(), env, 12, problem('network', {}, true));
    const row = triageRow(view, ctx);
    expect(row.model).toBeNull();
    expect(row.last?.window.id).toBe('weekly');
    expect(row.short).toBe(describeError(problem('network', {}, true), 'claude').title);

    const quota = compactQuota(view, weekly, ctx);
    expect(quota).toMatchObject({ old: true, status: undefined });
    expect(quota.shown).not.toBeNull();
    expect(quota.reading).toMatch(/\d/);
    expect(readAge(view, ctx)).toBe(t('meta.read_ago', { age: '12 min' }));
  });

  it('says nothing about the age of a reading from this poll', () => {
    expect(readAge(ready(), ctx)).toBeNull();
    expect(readAge(ready({ readAt: NOW - 3600 }), ctx)).not.toBeNull();
  });
});

describe('a source that lost its setup', () => {
  const replaced = notice('claude_status_line_replaced', 'warn');
  const lost = viewOf('claude', { state: 'needs_setup', notices: [replaced] });
  const never = viewOf('claude', { state: 'needs_setup' });

  it('stays on the home as broken, with the reason, while a first setup stays out', () => {
    expect(lostSetup(lost)).toBe(true);
    expect(lostSetup(never)).toBe(false);
    expect(activeViews([lost], ctx)).toEqual([lost]);
    expect(activeViews([never], ctx)).toEqual([]);

    const row = triageRow(lost, ctx);
    expect(row.tier).toBe('broken');
    expect(row.short).toBe(t('triage.short.setup'));
    expect(row.verdict).toBe(noticeText(replaced));
    expect(brokenHint(lost)).toBe(noticeText(replaced));
  });
});

describe('a sign-in problem behind a healthy reading', () => {
  it('reaches the card when the optional per-model source failed on the login', () => {
    for (const reason of ['oauth_expired', 'not_signed_in', 'unauthorized'] as const) {
      const view = ready({ notices: [notice('claude_oauth_partial', 'info', { reason })] });
      expect(signInNote(view)).toBe(t(`error.${reason}.title`));
      expect(triageRow(view, ctx).short).toBe(t(`error.${reason}.title`));
    }
  });

  it('stays quiet for a failure that is not about the login', () => {
    const view = ready({
      notices: [notice('claude_oauth_partial', 'info', { reason: 'timeout' })],
    });
    expect(signInNote(view)).toBe('');
    expect(triageRow(view, ctx).short).toBe('');
  });
});
