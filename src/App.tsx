// The flyout: the home surfaces (triage, horizon, channels), the settings screen, the lateral
// detail drawer and the Ctrl+K palette. Everything it shows comes from the Backend; this component
// only wires state, focus and the height protocol.
import { useCallback, useMemo, useRef, useState } from 'react';
import { cooldownLeft, waitText } from './app/cooldown';
import { DraftProvider, useDraftGuard } from './app/draftContext';
import { harnessName, TAB_ORDER } from './app/harness';
import { sharePct, showsUsed } from './app/quota';
import { useAsyncAction } from './app/useAsyncAction';
import { useFlyoutHeight } from './app/useFlyoutHeight';
import { useLocaleSync } from './app/useLocaleSync';
import { useNowSecs } from './app/useNowSecs';
import { useSettingsState } from './app/useSettingsState';
import { useSnapshotState } from './app/useSnapshotState';
import { useTimedText } from './app/useTimedText';
import { useUpdateState } from './app/useUpdateState';
import type { ViewContext } from './app/windowModel';
import { Home } from './home/Home';
import { useCommands } from './home/useCommands';
import { useHomeShell } from './home/useHomeShell';
import { useHomeView } from './home/useHomeView';
import { useOverlays } from './home/useOverlays';
import { useSettingsScreen } from './home/useSettingsScreen';
import { useSurface } from './home/useSurface';
import { t } from './i18n';
import type { Backend } from './lib/backend';
import type { HarnessId } from './lib/contract';

export function App({ backend }: { backend: Backend }) {
  return (
    <DraftProvider>
      <AppBody backend={backend} />
    </DraftProvider>
  );
}

function AppBody({ backend }: { backend: Backend }) {
  const request = useDraftGuard()?.request;
  const snapshotState = useSnapshotState(backend);
  const settingsApi = useSettingsState(backend);
  const updateApi = useUpdateState(backend);
  const locale = useLocaleSync(settingsApi.settings.general.language, settingsApi.loaded);
  const { nowSecs, sync } = useNowSecs();
  const [preferred, setPreferred] = useState<HarnessId>(TAB_ORDER[0] ?? 'claude');
  const [trend, setTrend] = useState(false);
  // The first run shows the wizard until the user finishes it once; the empty triage then keeps a
  // way back to it. Nothing is written: the flag is about this session, not about the profile.
  const [runDone, setRunDone] = useState(false);
  const [wizardRun, setWizardRun] = useState(0);
  const [notice, setNotice] = useTimedText(6000);
  const [blocked, setBlocked] = useTimedText(8000);
  const overlays = useOverlays();
  const { paletteOpen, detailOpen, closePalette, hideDetail, showDetail } = overlays;
  const rootRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  const refresh = useAsyncAction((id?: HarnessId) => backend.refresh(id));
  const quit = useAsyncAction(() => backend.quit());
  const hide = useAsyncAction(() => backend.hideFlyout());

  const ctx = useMemo<ViewContext>(
    () => ({ nowSecs, settings: settingsApi.settings }),
    [nowSecs, settingsApi.settings],
  );
  const home = useHomeView({
    snapshot: snapshotState.snapshot,
    loadError: snapshotState.error,
    loaded: settingsApi.loaded,
    ctx,
    preferred,
    runDone,
  });
  const { views, active, selected, view, tray, firstRun, failure, live } = home;
  const heightReady =
    (settingsApi.loaded || settingsApi.loadError !== null) &&
    (snapshotState.snapshot !== null || snapshotState.error !== null);

  useFlyoutHeight(backend, rootRef, scrollRef, undefined, heightReady);

  // A new surface or panel starts at the top.
  const toTop = useCallback(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
  }, []);
  // Leaving a form that has unsaved edits asks first, wherever the navigation comes from.
  const guarded = useCallback(
    (action: () => void) => {
      if (request) request(action);
      else action();
    },
    [request],
  );

  const {
    screen,
    inSettings,
    sectionOpen,
    openSettings,
    closeSettings,
    requestCloseSettings,
    toggleSettings,
    onSection,
    closeSection,
    navigation,
  } = useSettingsScreen({ guarded, hideDetail, toTop });

  const { surface, changeSurface, showTriage } = useSurface({
    guarded,
    hideDetail,
    toTop,
    shortcutsOff: paletteOpen,
  });
  // Arrow keys move the highlight; activating a provider opens its lateral detail, so browsing the
  // list with the keyboard never steals focus into the drawer.
  const highlight = useCallback((id: HarnessId) => setPreferred(id), []);
  const { openDetail: rememberAndOpen, closeDetail } = overlays;
  const openDetail = useCallback(
    (id: HarnessId) => {
      setPreferred(id);
      rememberAndOpen();
    },
    [rememberAndOpen],
  );

  // A manual refresh during a cool-down is dropped by the scheduler; say why instead of nothing.
  const refreshNow = useCallback(
    (id?: HarnessId) => {
      const target = id === undefined ? view : active.find((candidate) => candidate.id === id);
      const left = target ? cooldownLeft(target, nowSecs) : null;
      setBlocked(
        target && left !== null
          ? t('cooldown.refresh_blocked', { name: harnessName(target.id), wait: waitText(left) })
          : '',
      );
      void refresh.run(id);
    },
    [active, view, nowSecs, refresh, setBlocked],
  );

  const copySummary = useCallback(
    (text: string) => {
      const written = navigator.clipboard?.writeText(text);
      if (written === undefined) {
        setNotice(t('copy.failed'));
        return;
      }
      void written.then(
        () => setNotice(t('copy.done')),
        () => setNotice(t('copy.failed')),
      );
    },
    [setNotice],
  );
  const commands = useCommands({
    views: active,
    ctx,
    select: (id) => {
      setPreferred(id);
      showTriage();
      showDetail();
    },
    refresh: (id) => refreshNow(id),
    openSettings: (section) => openSettings(section),
    copySummary,
  });

  const { run: refreshAll } = refresh;
  useHomeShell({
    backend,
    request,
    settingsOpen: inSettings,
    sectionOpen,
    selected,
    detailOpen,
    paletteOpen,
    closePalette,
    closeSection,
    closeSettings,
    closeDetail,
    hide: () => void hide.run(),
    sync,
    refresh: () => void refreshAll(),
  });

  const shellFailure = refresh.error ?? quit.error ?? hide.error;
  return (
    <Home
      key={locale}
      backend={backend}
      settings={settingsApi}
      update={updateApi}
      navigation={navigation}
      message={blocked || live}
      fly={{
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
        tray: tray
          ? {
              pct: sharePct(tray.summary.availablePct, showsUsed(ctx.settings)),
              status: tray.summary.status,
              showUsed: showsUsed(ctx.settings),
            }
          : null,
        readAt: home.readAt,
        nowSecs,
        loaded: settingsApi.loaded,
        settled: heightReady,
        firstRun,
        failure,
        shellFailure,
        eventsFailed: snapshotState.eventsFailed,
        notice,
        paletteOpen,
        trend,
        detailOpen,
        commands,
        onTrend: setTrend,
        onSelect: highlight,
        onOpen: openDetail,
        onSurface: changeSurface,
        onRefresh: () => refreshNow(),
        onRefreshView: refreshNow,
        onRetry: snapshotState.reload,
        onQuit: () => (request ? request(() => void quit.run()) : void quit.run()),
        onCloseDetail: closeDetail,
        onClosePalette: closePalette,
        onToggleSettings: toggleSettings,
        onOpenSettings: openSettings,
        onSection,
        onCloseSettings: requestCloseSettings,
        onFinishOnboarding: () => {
          setRunDone(true);
          showTriage();
        },
        onConfigure: () => {
          setRunDone(false);
          setWizardRun((run) => run + 1);
          showTriage();
        },
      }}
    />
  );
}
