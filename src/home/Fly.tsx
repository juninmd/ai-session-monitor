// The flyout frame of the home: header, the switch between surfaces, the scrolling body with the
// selected surface (or the settings screen), the lateral detail drawer and the Ctrl+K palette.
// Everything it shows arrives as props; the state and the backend wiring live in App.
import type { RefObject } from 'react';
import type { SettingsSection } from '../app/harness';
import type { ViewContext } from '../app/windowModel';
import { Banner } from '../components/panel/Banner';
import { ActionError } from '../components/panel/ErrorActions';
import { SettingsView } from '../components/settings/SettingsView';
import { Footer } from '../components/shell/Footer';
import { Header } from '../components/shell/Header';
import { UpdateBanner } from '../components/shell/UpdateBanner';
import { t } from '../i18n';
import type { HarnessError, HarnessId, HarnessView } from '../lib/contract';
import type { Status } from '../lib/usage';
import { CommandPalette } from './CommandPalette';
import { DetailDrawer } from './Drawer';
import { Horizon } from './Horizon';
import { Onboarding } from './Onboarding';
import type { PaletteCommand } from './paletteModel';
import { SettingsDrawer } from './SettingsDrawer';
import { Sources } from './Sources';
import { Triage } from './Triage';

/** The surfaces the user navigates between. The first run is not one of them: it replaces them. */
export type Surface = 'triagem' | 'horizonte' | 'canais';

export const SURFACES: readonly { id: Surface; label: string; hint: string }[] = [
  { id: 'triagem', label: 'Triagem', hint: 'preciso agir?' },
  { id: 'horizonte', label: 'Horizonte', hint: 'chego até a renovação?' },
  { id: 'canais', label: 'Canais de cota', hint: 'o que ainda dá para ler' },
];

export type Screen = { kind: 'surface' } | { kind: 'settings'; section: SettingsSection | null };

export interface FlyProps {
  /** The frame itself and its scroller: App owns the height protocol they feed. */
  rootRef: RefObject<HTMLDivElement | null>;
  scrollRef: RefObject<HTMLDivElement | null>;
  /** Bumped when the empty panel asks for the wizard again: a new run starts at step one. */
  wizardRun: number;
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
  /** The stored settings are in: before that no surface can say what it is monitoring. */
  loaded: boolean;
  /** Nothing configured yet: the wizard takes the place of every surface. */
  firstRun: boolean;
  /** The first snapshot never arrived; the body shows this instead of a surface. */
  failure: string | null;
  shellFailure: HarnessError | null;
  eventsFailed: boolean;
  notice: string;
  paletteOpen: boolean;
  trend: boolean;
  /** The lateral detail is open for the selected provider. */
  detailOpen: boolean;
  commands: PaletteCommand[];
  onTrend: (on: boolean) => void;
  /** Move the highlight, as an arrow key does. */
  onSelect: (id: HarnessId) => void;
  /** Activate one provider and open its lateral detail. */
  onOpen: (id: HarnessId) => void;
  onSurface: (surface: Surface) => void;
  onRefresh: () => void;
  onRefreshView: (id: HarnessId) => void;
  onRetry: () => void;
  onQuit: () => void;
  onCloseDetail: () => void;
  onClosePalette: () => void;
  onToggleSettings: () => void;
  onOpenSettings: (section: SettingsSection | null) => void;
  /** Opens one settings section over the flyout, or closes it with null. */
  onSection: (section: SettingsSection | null) => void;
  onCloseSettings: () => void;
  onFinishOnboarding: () => void;
  onConfigure: () => void;
}

export function Fly(props: FlyProps) {
  const {
    rootRef,
    scrollRef,
    wizardRun,
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
    loaded,
    firstRun,
    failure,
    shellFailure,
    eventsFailed,
    notice,
    paletteOpen,
    trend,
    detailOpen,
    commands,
    onTrend,
    onSelect,
    onOpen,
    onSurface,
    onRefresh,
    onRefreshView,
    onRetry,
    onQuit,
    onCloseDetail,
    onClosePalette,
    onToggleSettings,
    onOpenSettings,
    onSection,
    onCloseSettings,
    onFinishOnboarding,
    onConfigure,
  } = props;
  const inSettings = screen.kind === 'settings';
  const surfaces = loaded && !inSettings && !firstRun && failure === null;
  const listSurface = surfaces && (surface === 'triagem' || surface === 'horizonte');
  return (
    <div
      ref={rootRef}
      className="flyout nx-fly"
      data-surface={surface}
      data-detail={detailOpen || undefined}
      data-density={ctx.settings.general.compact ? 'compact' : undefined}
      data-section={(inSettings && screen.section !== null) || undefined}
    >
      <Header
        tray={tray}
        settingsOpen={inSettings}
        errorCount={active.filter((item) => item.state === 'error').length}
        onRefresh={onRefresh}
        onToggleSettings={onToggleSettings}
        onQuit={onQuit}
      />
      <div className="nx-main">
        {surfaces && (
          <nav className="nx-surfaces" aria-label="Painéis do monitor">
            {SURFACES.map((item, index) => (
              <button
                key={item.id}
                type="button"
                aria-pressed={surface === item.id}
                title={`${item.hint} (Alt+${index + 1})`}
                aria-keyshortcuts={`Alt+${index + 1}`}
                onClick={() => onSurface(item.id)}
              >
                {item.label}
              </button>
            ))}
          </nav>
        )}
        <div
          ref={scrollRef}
          className="fly-body"
          {...(inSettings ? { role: 'region', 'aria-labelledby': 'settings-title' } : {})}
        >
          <div className="fly-content">
            {!inSettings && <UpdateBanner onOpen={() => onOpenSettings('update')} />}
            {shellFailure && <ActionError error={shellFailure} />}
            {eventsFailed && (
              <Banner tone="warn">
                <p>{t('load.events')}</p>
              </Banner>
            )}
            {notice !== '' && (
              <p className="nx-notice" role="status">
                {notice}
              </p>
            )}
            {!inSettings && !loaded && <output className="hint">Lendo as fontes…</output>}
            {inSettings && (
              <SettingsView
                section={screen.section}
                onSection={onSection}
                onCloseSettings={onCloseSettings}
              />
            )}
            {!inSettings && failure !== null && (
              <section className="card notice" data-status="crit">
                <h2>{t('load.title')}</h2>
                <p>{failure}</p>
                <div className="actions">
                  <button type="button" className="btn primary" onClick={onRetry}>
                    {t('action.retry')}
                  </button>
                </div>
              </section>
            )}
            {surfaces && surface === 'triagem' && (
              <Triage
                views={active}
                ctx={ctx}
                selected={selected}
                onSelect={onSelect}
                onOpen={onOpen}
                onConfigure={onConfigure}
              />
            )}
            {surfaces && surface === 'horizonte' && (
              <Horizon
                views={active}
                ctx={ctx}
                selected={selected}
                onSelect={onSelect}
                onOpen={onOpen}
                trend={trend}
                onTrend={onTrend}
              />
            )}
            {surfaces && surface === 'canais' && <Sources />}
            {/* Kept mounted whichever surface is on screen: opening a source's settings from the
                wizard would otherwise throw the picked list and the step away. */}
            <div className="nx-slot" hidden={inSettings || !firstRun || failure !== null}>
              <Onboarding
                key={wizardRun}
                views={views}
                ctx={ctx}
                onOpenSettings={onOpenSettings}
                onFinish={onFinishOnboarding}
              />
            </div>
          </div>
        </div>
      </div>
      {listSurface && detailOpen && view !== undefined && (
        <DetailDrawer
          view={view}
          ctx={ctx}
          onClose={onCloseDetail}
          onRefresh={() => onRefreshView(view.id)}
        />
      )}
      {/* A section covers the whole frame, not the scrolling body: it must not scroll away with
          the menu it lands on. */}
      {inSettings && <SettingsDrawer section={screen.section} onSection={onSection} />}
      <Footer readAt={readAt} nowSecs={nowSecs} />
      {/* The palette opens from anywhere in the flyout, so it listens for Ctrl+K itself (App). */}
      {paletteOpen && <CommandPalette commands={commands} onClose={onClosePalette} />}
    </div>
  );
}
