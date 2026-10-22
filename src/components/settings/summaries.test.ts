import { describe, expect, it } from 'vitest';
import { defaultSettings } from '../../lib/contract';
import { MockUpdater } from '../../mock/updater';
import { historySummary, monitoringSummary, sourcesSummary, updateSummary } from './summaries';

const settings = (change: (draft: typeof defaultSettings) => void) => {
  const draft = structuredClone(defaultSettings);
  change(draft);
  return draft;
};

describe('the menu summaries', () => {
  it('read the cadence, the adaptive pace and the alert threshold from the settings', () => {
    expect(monitoringSummary(defaultSettings)).toBe('1 min · adaptativo · avisa em 25%');
    const slow = settings((s) => {
      s.general.intervalSecs = 900;
      s.general.adaptive = false;
      s.general.warnAtPct = 40;
    });
    expect(monitoringSummary(slow)).toBe('15 min · avisa em 40%');
  });

  it('say how long the history is kept, or that nothing is kept', () => {
    expect(historySummary(defaultSettings)).toBe('Ligado · 35 dias');
    expect(
      historySummary(
        settings((s) => {
          s.general.historyEnabled = false;
        }),
      ),
    ).toBe('Desligado');
  });

  it('name the update mode, and leave the version out until the updater reports one', () => {
    expect(updateSummary(true, null)).toBe('Automáticas');
    expect(updateSummary(false, null)).toBe('Manuais');
    expect(updateSummary(true, new MockUpdater().status)).toBe('Automáticas · v0.1.0');
  });

  it('count every source and the unofficial ones the groups badge', () => {
    expect(sourcesSummary()).toBe('16 fontes · 5 não oficiais');
  });
});
