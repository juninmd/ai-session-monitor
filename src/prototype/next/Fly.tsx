// The proposed flyout itself. The comparison view simply renders it next to the current interface,
// so everything it needs arrives as props and nothing is read from a global.

import type { SettingsSection } from '../../app/harness';
import { harnessName } from '../../app/harness';
import type { ViewContext } from '../../app/windowModel';
import { SettingsView } from '../../components/settings/SettingsView';
import { Footer } from '../../components/shell/Footer';
import { Header } from '../../components/shell/Header';
import type { HarnessId, HarnessView } from '../../lib/contract';
import type { Status } from '../../lib/usage';
import { Horizon } from './Horizon';
import { Onboarding } from './Onboarding';
import { Panel } from './Panel';
import type { PaletteCommand } from './paletteModel';
import { Sources } from './Sources';
import { Triage } from './Triage';

export type Surface = 'triagem' | 'horizonte' | 'comando' | 'primeiro' | 'canais' | 'comparar';

export type Screen = { kind: 'surface' } | { kind: 'settings'; section: SettingsSection | null };

export interface FlyProps {
  surface: Surface;
  screen: Screen;
  views: readonly HarnessView[];
  active: readonly HarnessView[];
  selected: HarnessId;
  view: HarnessView | undefined;
  ctx: ViewContext;
  tray: { pct: number; status: Status; showUsed: boolean } | null;
  readAt: number | null;
  nowSecs: number;
  trend: boolean;
  onTrend: (on: boolean) => void;
  onSelect: (id: HarnessId) => void;
  onRefresh: () => void;
  onToggleSettings: () => void;
  onOpenPalette: () => void;
  onOpenSettings: (section: SettingsSection | null) => void;
  onCloseSettings: () => void;
  onFinishOnboarding: () => void;
  onConfigure: () => void;
  commands: PaletteCommand[];
}

export function PalettePreview({
  commands,
  onOpen,
}: {
  commands: PaletteCommand[];
  onOpen: () => void;
}) {
  return (
    <section className="nx-section">
      <h3 className="nx-section-head">
        Comando rápido
        <span className="nx-count">{commands.length}</span>
      </h3>
      <p className="nx-note-line nx-meta">
        O flyout fica aberto por poucos segundos. Pular para um provider, atualizar uma única fonte
        ou copiar o estado em texto não deveria exigir mira de mouse nem rolagem.
      </p>
      <div className="actions">
        <button type="button" className="btn primary" onClick={onOpen}>
          Abrir comando rápido
        </button>
      </div>
      <ul className="nx-cmd-list">
        {commands.map((command) => (
          <li key={command.id}>
            <span className="nx-cmd-label">{command.label}</span>
            <span className="nx-cmd-hint">{command.hint}</span>
            <span className="nx-tag">{command.group}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function Fly(props: FlyProps) {
  const {
    surface,
    screen,
    views,
    active,
    selected,
    view,
    ctx,
    tray,
    readAt,
    nowSecs,
    trend,
    onTrend,
    onSelect,
    onRefresh,
    onToggleSettings,
    onOpenPalette,
    onOpenSettings,
    onCloseSettings,
    onFinishOnboarding,
    onConfigure,
    commands,
  } = props;
  const inSettings = screen.kind === 'settings';
  const listSurface = !inSettings && (surface === 'triagem' || surface === 'horizonte');
  return (
    <div className="flyout nx-fly" data-surface={surface}>
      <Header
        tray={tray}
        settingsOpen={inSettings}
        onRefresh={onRefresh}
        onToggleSettings={onToggleSettings}
      />
      <div className="fly-body">
        <div className="fly-content">
          {inSettings && <SettingsView section={screen.section} onBack={onCloseSettings} />}
          {surface === 'triagem' && !inSettings && (
            <Triage
              views={active}
              ctx={ctx}
              selected={selected}
              onSelect={onSelect}
              onConfigure={onConfigure}
            />
          )}
          {surface === 'horizonte' && !inSettings && (
            <Horizon
              views={active}
              ctx={ctx}
              selected={selected}
              onSelect={onSelect}
              trend={trend}
              onTrend={onTrend}
            />
          )}
          {surface === 'comando' && !inSettings && (
            <PalettePreview commands={commands} onOpen={onOpenPalette} />
          )}
          {surface === 'canais' && !inSettings && <Sources />}
          {/* Kept mounted whichever surface is on screen: opening a source's settings from the
              wizard would otherwise throw the picked list and the step away. */}
          <div className="nx-slot" hidden={inSettings || surface !== 'primeiro'}>
            <Onboarding
              views={views}
              ctx={ctx}
              onOpenSettings={onOpenSettings}
              onFinish={onFinishOnboarding}
            />
          </div>
          {listSurface && view !== undefined && <Panel view={view} ctx={ctx} />}
          {listSurface && view === undefined && (
            <p className="hint">Nada selecionado em {harnessName(selected)}.</p>
          )}
        </div>
      </div>
      <Footer readAt={readAt} nowSecs={nowSecs} onRefresh={onRefresh} onQuit={() => undefined} />
    </div>
  );
}
