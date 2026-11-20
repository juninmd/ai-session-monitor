// The flyout frame of the home: header, the scrolling body with the triage (or the settings
// screen), the detail sheet over the cards and the Ctrl+K palette.
// Everything it shows arrives as props; the state and the backend wiring live in App.
import type { RefObject } from 'react';
import type { ViewMode } from '../app/hooks/useViewMode';
import type { SettingsSection } from '../app/model/harness';
import type { ViewContext } from '../app/model/windowModel';
import { Banner } from '../components/panel/Banner';
import { ActionError } from '../components/panel/ErrorActions';
import { SettingsView } from '../components/settings/SettingsView';
import { Footer } from '../components/shell/Footer';
import { Header } from '../components/shell/Header';
import { t } from '../i18n';
import type { HarnessError, HarnessId, HarnessView } from '../lib/contract';
import type { Status } from '../lib/usage';
import { CommandPalette } from './CommandPalette';
import { DetailDrawer } from './Drawer';
import { Onboarding } from './Onboarding';
import type { PaletteCommand } from './paletteModel';
import { SettingsDrawer } from './SettingsDrawer';
import { Triage } from './Triage';

export type Screen = { kind: 'surface' } | { kind: 'settings'; section: SettingsSection | null };

export interface FlyProps {
  /** The frame itself and its scroller: App owns the height protocol they feed. */
  rootRef: RefObject<HTMLDivElement | null>;
  scrollRef: RefObject<HTMLDivElement | null>;
  /** Bumped when the empty panel asks for the wizard again: a new run starts at step one. */
  wizardRun: number;
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
  /** Both settings and the initial snapshot settled, so the frame can release its initial height. */
  settled?: boolean;
  /** Nothing configured yet: the wizard takes the place of every surface. */
  firstRun: boolean;
  /** The first snapshot never arrived; the body shows this instead of a surface. */
  failure: string | null;
  shellFailure: HarnessError | null;
  eventsFailed: boolean;
  notice: string;
  paletteOpen: boolean;
  /** Cards (two per row) or one provider per line. */
  viewMode: ViewMode;
  /** The lateral detail is open for the selected provider. */
  detailOpen: boolean;
  commands: PaletteCommand[];
  onViewMode: (mode: ViewMode) => void;
  /** Move the highlight, as an arrow key does. */
  onSelect: (id: HarnessId) => void;
  /** Activate one provider and open its lateral detail. */
  onOpen: (id: HarnessId) => void;
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
    settled,
    firstRun,
    failure,
    shellFailure,
    eventsFailed,
    notice,
    paletteOpen,
    viewMode,
    detailOpen,
    commands,
    onViewMode,
    onSelect,
    onOpen,
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
  return (
    <div
      ref={rootRef}
      className="flyout nx-fly"
      data-loaded={(settled ?? loaded) || undefined}
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
        <div
          ref={scrollRef}
          className="fly-body"
          {...(inSettings ? { role: 'region', 'aria-labelledby': 'settings-title' } : {})}
        >
          <div className="fly-content">
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
            {!inSettings && !loaded && (
              <output className="hint">{t('surface.loading_sources')}</output>
            )}
            {inSettings && (
              <SettingsView
                section={screen.section}
                onSection={onSection}
                onCloseSettings={onCloseSettings}
              />
            )}
            {!inSettings && failure !== null && (
              <section className="card notice" data-status="crit" role="alert">
                <h2>{t('load.title')}</h2>
                <p>{failure}</p>
                <div className="actions">
                  <button type="button" className="btn primary" onClick={onRetry}>
                    {t('action.retry')}
                  </button>
                </div>
              </section>
            )}
            {surfaces && (
              <Triage
                views={active}
                ctx={ctx}
                selected={selected}
                detailOpen={detailOpen}
                viewMode={viewMode}
                onViewMode={onViewMode}
                onSelect={onSelect}
                onOpen={onOpen}
                onRefreshView={onRefreshView}
                onConfigure={onConfigure}
              />
            )}
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
      {surfaces && viewMode === 'cards' && detailOpen && view !== undefined && (
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
      <Footer readAt={readAt} nowSecs={nowSecs} onOpenUpdate={() => onOpenSettings('update')} />
      {/* The palette opens from anywhere in the flyout, so it listens for Ctrl+K itself (App). */}
      {paletteOpen && <CommandPalette commands={commands} onClose={onClosePalette} />}
    </div>
  );
}
