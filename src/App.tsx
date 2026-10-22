// The flyout: the home surfaces (triagem, horizonte, canais), the settings screen, the lateral
// detail drawer and the Ctrl+K palette. Everything it shows comes from the Backend; this component
// only wires state, focus and the height protocol.
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
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
import { useUpdateState } from './app/useUpdateState';
import type { ViewContext } from './app/windowModel';
import { SURFACES, type Surface } from './home/Fly';
import { Home } from './home/Home';
import { useCommands } from './home/useCommands';
import { useHomeShell } from './home/useHomeShell';
import { useHomeView } from './home/useHomeView';
import { useSettingsScreen } from './home/useSettingsScreen';
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
  const [surface, setSurface] = useState<Surface>('triagem');
  const [preferred, setPreferred] = useState<HarnessId>(TAB_ORDER[0] ?? 'claude');
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [detailOpen, setDetailOpen] = useState(false);
  const [trend, setTrend] = useState(false);
  // The first run shows the wizard until the user finishes it once; the empty triage then keeps a
  // way back to it. Nothing is written: the flag is about this session, not about the profile.
  const [runDone, setRunDone] = useState(false);
  const [wizardRun, setWizardRun] = useState(0);
  const [notice, setNotice] = useState('');
  const [blocked, setBlocked] = useState('');
  const rootRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const paletteOpener = useRef<Element | null>(null);
  const detailOpener = useRef<Element | null>(null);

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

  useFlyoutHeight(backend, rootRef, scrollRef);

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

  const hideDetail = useCallback(() => setDetailOpen(false), []);
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

  const changeSurface = useCallback(
    (next: Surface) => {
      guarded(() => {
        setSurface(next);
        setDetailOpen(false);
        toTop();
      });
    },
    [guarded, toTop],
  );
  // Arrow keys move the highlight; activating a provider opens its lateral detail, so browsing the
  // list with the keyboard never steals focus into the drawer.
  const highlight = useCallback((id: HarnessId) => setPreferred(id), []);
  const openDetail = useCallback((id: HarnessId) => {
    detailOpener.current = document.activeElement;
    setPreferred(id);
    setDetailOpen(true);
  }, []);
  const closeDetail = useCallback(() => {
    setDetailOpen(false);
    const opener = detailOpener.current;
    if (opener instanceof HTMLElement) opener.focus();
  }, []);

  const openPalette = useCallback(() => {
    paletteOpener.current = document.activeElement;
    setPaletteOpen(true);
  }, []);
  const closePalette = useCallback(() => {
    setPaletteOpen(false);
    const opener = paletteOpener.current;
    if (opener instanceof HTMLElement) opener.focus();
  }, []);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() !== 'k' || !(event.ctrlKey || event.metaKey)) return;
      event.preventDefault();
      openPalette();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [openPalette]);

  // Alt+1..3 jump to a surface; the digit maps to its position in the vertical tab list.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!event.altKey || event.ctrlKey || event.metaKey || paletteOpen) return;
      const target = SURFACES[Number(event.key) - 1];
      if (target === undefined) return;
      event.preventDefault();
      changeSurface(target.id);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [changeSurface, paletteOpen]);

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
    [active, view, nowSecs, refresh],
  );
  useEffect(() => {
    if (blocked === '') return;
    const timer = setTimeout(() => setBlocked(''), 8000);
    return () => clearTimeout(timer);
  }, [blocked]);
  useEffect(() => {
    if (notice === '') return;
    const timer = setTimeout(() => setNotice(''), 6000);
    return () => clearTimeout(timer);
  }, [notice]);

  const copySummary = useCallback((text: string) => {
    const written = navigator.clipboard?.writeText(text);
    if (written === undefined) {
      setNotice('Não foi possível copiar o resumo.');
      return;
    }
    void written.then(
      () => setNotice('Resumo das fontes copiado para a área de transferência.'),
      () => setNotice('Não foi possível copiar o resumo.'),
    );
  }, []);
  const commands = useCommands({
    views: active,
    ctx,
    select: (id) => {
      setPreferred(id);
      setSurface('triagem');
      setDetailOpen(true);
      toTop();
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
          setSurface('triagem');
          toTop();
        },
        onConfigure: () => {
          setRunDone(false);
          setWizardRun((run) => run + 1);
          setSurface('triagem');
          toTop();
        },
      }}
    />
  );
}
