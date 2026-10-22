// The command list behind Ctrl+K. Pure: it turns views, a context and a set of callbacks into
// rankable commands, so the filtering rules can be tested without a DOM.
import { harnessName } from '../app/harness';
import { sharePct, showsUsed } from '../app/quota';
import { summaryOf } from '../app/tabModel';
import type { ViewContext } from '../app/windowModel';
import { t, tn } from '../i18n';
import { formatPercent } from '../i18n/format';
import { getLocale } from '../i18n/locale';
import type { HarnessId, HarnessView } from '../lib/contract';
import { SCENARIO_NAMES } from '../mock/scenarios';
import { triageRow } from './triageModel';

/** The group's id; the heading is `paletteGroupLabel(group)`, so the language can change. */
export type PaletteGroup = 'providers' | 'actions' | 'settings' | 'demo';

export const GROUP_ORDER: readonly PaletteGroup[] = ['providers', 'actions', 'settings', 'demo'];

export const paletteGroupLabel = (group: PaletteGroup): string => t(`palette.group.${group}`);

export interface PaletteCommand {
  id: string;
  group: PaletteGroup;
  label: string;
  hint: string;
  /** Extra words that should match, never shown. */
  keywords: string;
  run: () => void;
}

export interface PaletteInput {
  views: readonly HarnessView[];
  ctx: ViewContext;
  select: (id: HarnessId) => void;
  refresh: (id?: HarnessId) => void;
  openSettings: (section: HarnessId | 'general' | null) => void;
  copySummary: (text: string) => void;
  /** Browser-only dev loop (`?mock=`); the packaged app has no scenario to switch to. */
  scenario?: (name: string) => void;
}

/**
 * Providers the palette offers. The app already passes a filtered list, so this only guards a
 * caller that hands over raw snapshot views.
 */
function reachable(view: HarnessView): boolean {
  return !['disabled', 'needs_consent', 'needs_setup'].includes(view.state);
}

function providerCommand(view: HarnessView, input: PaletteInput): PaletteCommand {
  const { ctx } = input;
  const summary = summaryOf(view, ctx);
  const percent =
    summary === null
      ? null
      : formatPercent(sharePct(summary.availablePct, showsUsed(ctx.settings)));
  return {
    id: `go:${view.id}`,
    group: 'providers',
    label: harnessName(view.id),
    hint: [triageRow(view, ctx).verdict, percent].filter((part) => part !== null).join(' · '),
    keywords: view.plan ?? '',
    run: () => input.select(view.id),
  };
}

export function summaryText(views: readonly HarnessView[], ctx: ViewContext): string {
  const lines = views.map((view) => {
    const summary = summaryOf(view, ctx);
    const percent =
      summary === null
        ? null
        : formatPercent(sharePct(summary.availablePct, showsUsed(ctx.settings)));
    const verdict = triageRow(view, ctx).verdict;
    return `- ${harnessName(view.id)}: ${percent === null ? verdict : `${percent} · ${verdict}`}`;
  });
  return ['AI Session Monitor', ...lines].join('\n');
}

export function commandsFor(input: PaletteInput): PaletteCommand[] {
  const { views, ctx } = input;
  const active = views.filter(reachable);
  const text = summaryText(active, ctx);
  return [
    ...active.map((view) => providerCommand(view, input)),
    {
      id: 'refresh:all',
      group: 'actions',
      label: t('palette.refresh.label'),
      hint: t('palette.refresh.hint'),
      keywords: t('palette.refresh.keywords'),
      run: () => input.refresh(),
    },
    {
      id: 'copy:status',
      group: 'actions',
      label: t('palette.copy.label'),
      hint: tn('palette.copy.hint', active.length),
      keywords: t('palette.copy.keywords'),
      run: () => input.copySummary(text),
    },
    {
      id: 'settings:general',
      group: 'settings',
      label: t('palette.general.label'),
      hint: t('palette.general.hint'),
      keywords: t('palette.general.keywords'),
      run: () => input.openSettings('general'),
    },
    ...active.map<PaletteCommand>((view) => ({
      id: `settings:${view.id}`,
      group: 'settings',
      label: t('palette.configure.label', { name: harnessName(view.id) }),
      hint: view.stale
        ? t('palette.configure.hint_stale')
        : (view.plan ?? t('palette.configure.hint_default')),
      keywords: t('palette.configure.keywords'),
      run: () => input.openSettings(view.id),
    })),
    ...(input.scenario === undefined
      ? []
      : SCENARIO_NAMES.map<PaletteCommand>((name) => ({
          id: `scenario:${name}`,
          group: 'demo',
          label: t('palette.scenario.label', { name }),
          hint: t('palette.scenario.hint'),
          keywords: t('palette.scenario.keywords'),
          run: () => input.scenario?.(name),
        }))),
  ];
}

function fold(value: string): string {
  return value
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase();
}

/** Empty query keeps the built-in order; otherwise label matches come before keyword matches. */
export function filterCommands(
  commands: readonly PaletteCommand[],
  query: string,
): PaletteCommand[] {
  const needle = fold(query.trim());
  if (needle === '') return [...commands];
  const scored: { command: PaletteCommand; score: number }[] = [];
  for (const command of commands) {
    const label = fold(command.label);
    const keywords = fold(`${command.keywords} ${command.hint}`);
    if (label.startsWith(needle)) scored.push({ command, score: 0 });
    else if (label.includes(needle)) scored.push({ command, score: 1 });
    else if (keywords.includes(needle)) scored.push({ command, score: 2 });
  }
  return scored
    .sort(
      (a, b) => a.score - b.score || a.command.label.localeCompare(b.command.label, getLocale()),
    )
    .map((entry) => entry.command);
}

/** Move the highlight by one step, wrapping at both ends. */
export function step(index: number, delta: number, length: number): number {
  if (length === 0) return 0;
  return (((index + delta) % length) + length) % length;
}
