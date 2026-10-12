import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { HarnessView, Settings } from '../../lib/contract';
import { claudeView } from '../../mock/builders';
import { viewOf } from '../../mock/fixtures';
import { ctxOf, envOf, renderPanel, settingsWith, win } from '../../test/helpers';
import { HarnessPanel } from '../HarnessPanel';

function ready(view: HarnessView, settings?: Settings) {
  const options = settings ? { settings } : {};
  return renderPanel(<HarnessPanel view={view} ctx={ctxOf(settings)} />, options);
}

describe('meta chips', () => {
  describe('say where a budget came from', () => {
    const budgetOf = (meta: Record<string, string>) =>
      ready(viewOf('litellm', { windows: [win()], meta }));

    it.each([
      ['key', 'Orçamento da chave'],
      ['user', 'Orçamento do usuário'],
      ['team', 'Orçamento do time'],
    ])('name the %s level when it is not the one that was configured', (level, chip) => {
      budgetOf({ scope: level === 'key' ? 'team' : 'key', level, alias: 'squad' });
      expect(screen.getByText(chip)).toBeVisible();
      expect(document.body).not.toHaveTextContent('level:');
    });

    it('stay quiet when the level is the configured one', () => {
      budgetOf({ scope: 'team', level: 'team', alias: 'squad' });
      expect(screen.getByText('Time: squad')).toBeVisible();
      expect(screen.queryByText(/^Orçamento d/)).toBeNull();
    });

    it('name the level even when the scope was not reported', () => {
      budgetOf({ level: 'user' });
      expect(screen.getByText('Orçamento do usuário')).toBeVisible();
    });

    it('show a level nobody named yet as it came', () => {
      budgetOf({ scope: 'key', level: 'org' });
      expect(screen.getByText('level: org')).toBeVisible();
    });
  });

  describe('say what else a source added', () => {
    it('flag a local estimate mixed into real windows, in words', () => {
      ready(viewOf('opencode', { windows: [win()], meta: { local_estimate: 'true' } }));
      expect(screen.getByText('Inclui estimativa local')).toBeVisible();
      expect(document.body).not.toHaveTextContent('local_estimate');
    });

    it('show an estimate flag with an unexpected value as it came', () => {
      ready(viewOf('opencode', { windows: [win()], meta: { local_estimate: 'maybe' } }));
      expect(screen.getByText('local_estimate: maybe')).toBeVisible();
      expect(screen.queryByText('Inclui estimativa local')).toBeNull();
    });

    it.each([false, true])(
      'flag windows from the unofficial Claude usage API once, with the setting at %s',
      (oauthUsage) => {
        const settings = settingsWith((s) => {
          s.claude.oauthUsage = oauthUsage;
        });
        ready({ ...claudeView(envOf(), 'ok'), meta: { extra_source: 'oauth_usage' } }, settings);
        expect(screen.getAllByText('Também: API de uso, não oficial')).toHaveLength(1);
        expect(document.body).not.toHaveTextContent('extra_source');
      },
    );

    it('show an extra source nobody named yet as it came', () => {
      ready({ ...claudeView(envOf(), 'ok'), meta: { extra_source: 'something_new' } });
      expect(screen.getByText('extra_source: something_new')).toBeVisible();
    });
  });
});
