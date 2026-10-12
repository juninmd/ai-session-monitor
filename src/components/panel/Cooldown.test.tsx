import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { HarnessView } from '../../lib/contract';
import { claudeView } from '../../mock/builders';
import { failed, problem } from '../../mock/fixtures';
import { ctxOf, envOf, NOW, renderPanel } from '../../test/helpers';
import { HarnessPanel } from '../HarnessPanel';

const limited = () => problem('rate_limited', { httpStatus: '429' }, true);
const text = /Nova tentativa em 4 min/;

function show(view: HarnessView) {
  return renderPanel(<HarnessPanel view={view} ctx={ctxOf()} />);
}

describe('a rate-limit cool-down', () => {
  it('keeps the last good numbers, says how old they are and when the next attempt is', () => {
    const good = claudeView(envOf(), 'ok');
    show({ ...good, stale: true, readAt: NOW - 600, error: limited(), nextAttemptAt: NOW + 240 });
    expect(screen.getByText(/Mostrando os últimos números lidos, de 10 min atrás/)).toBeVisible();
    expect(screen.getByText(text)).toBeVisible();
    expect(screen.getByRole('heading', { name: 'Sessão de 5 horas' })).toBeVisible();
  });

  it('is also said when there is no good reading to show', () => {
    show({ ...failed('claude', limited()), nextAttemptAt: NOW + 240 });
    expect(screen.getByText(text)).toBeVisible();
  });

  it('says nothing once the time has passed or when the failure is an ordinary one', () => {
    const good = claudeView(envOf(), 'ok');
    show({ ...good, stale: true, error: limited(), nextAttemptAt: NOW - 5 });
    expect(screen.queryByText(/Nova tentativa/)).toBeNull();
  });
});
