// The card on the DOM: a stale source keeps its last numbers in sight, a failed one says why, and a
// source that lost its setup is a card with a reason instead of an absence.
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { ViewContext } from '../app/model/windowModel';
import { noticeText } from '../app/text/noticeText';
import { t } from '../i18n';
import { defaultSettings, type HarnessView } from '../lib/contract';
import { asStale, failed, notice, problem, viewOf, windowOf } from '../mock/fixtures';
import { startFor } from '../mock/scenarios';
import { NOW } from '../test/helpers';
import { activeViews } from './model';
import { Triage } from './Triage';

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

function card(view: HarnessView): HTMLElement {
  render(
    <Triage
      views={activeViews([view], ctx)}
      ctx={ctx}
      selected="claude"
      onSelect={() => {}}
      onOpen={() => {}}
      onConfigure={() => {}}
    />,
  );
  const tile = document.querySelector<HTMLElement>('button[data-tier]');
  if (tile === null) throw new Error('no card was drawn');
  return tile;
}

describe('triage card', () => {
  it('keeps the last numbers of a stale source on the card, marked as old', () => {
    const stale = asStale(viewOf('claude', { windows: [weekly] }), env, 12, problem('network'));
    const tile = card(stale);
    expect(tile).toHaveAttribute('data-old', 'true');
    expect(tile.querySelector('.nx-pct')?.textContent).toMatch(/\d+\s?%/);
    expect(tile.querySelector('.nx-q[data-old] .nx-q-pct')?.textContent).toMatch(/\d+\s?%/);
    // Old numbers never carry a status colour, and the card says how old they are.
    expect(tile.querySelector('.nx-q-pct')).not.toHaveAttribute('data-status');
    expect(tile.textContent).toContain(t('meta.read_ago', { age: '12 min' }));
    expect(tile.textContent).toContain(t('error.network.title'));
  });

  it('says why a source failed instead of "failed" twice', () => {
    const tile = card(failed('claude', problem('oauth_expired')));
    expect(tile.textContent).toContain(t('error.oauth_expired.title'));
    expect(tile.textContent).toContain(t('error.oauth_expired.body'));
    expect(tile.textContent).not.toContain(t('tab.error'));
  });

  it('shows a source whose status line was replaced, with the reason', () => {
    const replaced = notice('claude_status_line_replaced', 'warn');
    const tile = card(viewOf('claude', { state: 'needs_setup', notices: [replaced] }));
    expect(tile).toHaveAttribute('data-tier', 'broken');
    expect(tile.textContent).toContain(t('triage.short.setup'));
    // The sentence is on the card once: the screen reader does not hear it twice.
    expect(tile.textContent?.split(noticeText(replaced))).toHaveLength(2);
  });
});
