// The command list behind Ctrl+K. Pure: it turns views, a context and a set of callbacks into
// rankable commands, so the filtering rules can be tested without a DOM.
import { harnessName } from '../../app/harness';
import { sharePct, showsUsed } from '../../app/quota';
import { summaryOf } from '../../app/tabModel';
import type { ViewContext } from '../../app/windowModel';
import { formatPercent } from '../../i18n/format';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { SCENARIO_NAMES } from '../../mock/scenarios';
import { triageRow } from './triageModel';

export type PaletteGroup = 'Providers' | 'Ações' | 'Configurações' | 'Demonstração';

export const GROUP_ORDER: readonly PaletteGroup[] = [
  'Providers',
  'Ações',
  'Configurações',
  'Demonstração',
];

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
  compare: () => void;
  copySummary: (text: string) => void;
  scenario: (name: string) => void;
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
    group: 'Providers',
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
      group: 'Ações',
      label: 'Atualizar todas as fontes',
      hint: 'releitura imediata',
      keywords: 'refresh atualizar recarregar',
      run: () => input.refresh(),
    },
    {
      id: 'copy:status',
      group: 'Ações',
      label: 'Copiar resumo do estado',
      hint: `${active.length} fontes em texto`,
      keywords: 'clipboard copiar status relatorio',
      run: () => input.copySummary(text),
    },
    {
      id: 'view:compare',
      group: 'Ações',
      label: 'Comparar com a interface atual',
      hint: 'lado a lado, mesmo cenário',
      keywords: 'comparar diff antes depois',
      run: input.compare,
    },
    {
      id: 'settings:general',
      group: 'Configurações',
      label: 'Configurações gerais',
      hint: 'exibição, limites, bandeja',
      keywords: 'preferencias opcoes',
      run: () => input.openSettings('general'),
    },
    ...active.map<PaletteCommand>((view) => ({
      id: `settings:${view.id}`,
      group: 'Configurações',
      label: `Configurar ${harnessName(view.id)}`,
      hint: view.stale ? 'leitura antiga' : (view.plan ?? 'fonte e credenciais'),
      keywords: 'configurar ajustar fonte credencial',
      run: () => input.openSettings(view.id),
    })),
    ...SCENARIO_NAMES.map<PaletteCommand>((name) => ({
      id: `scenario:${name}`,
      group: 'Demonstração',
      label: `Cenário: ${name}`,
      hint: 'dados fictícios',
      keywords: 'mock demo dados',
      run: () => input.scenario(name),
    })),
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
    .sort((a, b) => a.score - b.score || a.command.label.localeCompare(b.command.label, 'pt-BR'))
    .map((entry) => entry.command);
}

/** Move the highlight by one step, wrapping at both ends. */
export function step(index: number, delta: number, length: number): number {
  if (length === 0) return 0;
  return (((index + delta) % length) + length) % length;
}
