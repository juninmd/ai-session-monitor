// The flyout: header, tab strip, one scrolling panel body and a footer. Everything the panel
// shows comes from the Backend; this component only wires state, focus and the height protocol.
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { BackendContext } from './app/backendContext';
import { cooldownLeft, waitText } from './app/cooldown';
import { DraftProvider, useDraftGuard } from './app/draftContext';
import { describeError } from './app/errorText';
import { harnessName, type SettingsSection, TAB_ORDER } from './app/harness';
import { NavigationContext } from './app/navigation';
import { latestReadAt, viewsInTabOrder } from './app/placeholder';
import { sharePct, showsUsed } from './app/quota';
import { SettingsContext } from './app/settingsContext';
import { traySummary } from './app/tabModel';
import { useAnnouncer } from './app/useAnnouncer';
import { useAsyncAction } from './app/useAsyncAction';
import { useEscape } from './app/useEscape';
import { useFlyoutHeight } from './app/useFlyoutHeight';
import { useNowSecs } from './app/useNowSecs';
import { useSettingsState } from './app/useSettingsState';
import { useSnapshotState } from './app/useSnapshotState';
import { useSubscription } from './app/useSubscription';
import { UpdateContext, useUpdateState } from './app/useUpdateState';
import type { ViewContext } from './app/windowModel';
import { HarnessPanel } from './components/HarnessPanel';
import { Banner } from './components/panel/Banner';
import { ActionError } from './components/panel/ErrorActions';
import { SettingsView } from './components/settings/SettingsView';
import { Footer } from './components/shell/Footer';
import { Header } from './components/shell/Header';
import { LiveRegion } from './components/shell/LiveRegion';
import { ProviderPicker } from './components/shell/ProviderPicker';
import { focusTab, PANEL_ID, TabStrip, tabNameId } from './components/shell/TabStrip';
import { UpdateBanner } from './components/shell/UpdateBanner';
import { t } from './i18n';
import type { Backend } from './lib/backend';
import type { HarnessId, HarnessView } from './lib/contract';
import { activeViews } from './prototype/model';
import { PreviewNavigation, PreviewPanel } from './prototype/PreviewNavigation';
import { PreviewSettings } from './prototype/PreviewSettings';

type Screen = { kind: 'tabs' } | { kind: 'settings'; section: SettingsSection | null };

const NO_VIEWS: readonly HarnessView[] = [];

export function App({ backend, preview = false }: { backend: Backend; preview?: boolean }) {
  return (
    <DraftProvider>
      <AppBody backend={backend} preview={preview} />
    </DraftProvider>
  );
}

function AppBody({ backend, preview }: { backend: Backend; preview: boolean }) {
  const request = useDraftGuard()?.request;
  const snapshotState = useSnapshotState(backend);
  const settingsApi = useSettingsState(backend);
  const updateApi = useUpdateState(backend);
  const { nowSecs, sync } = useNowSecs();
  const [preferred, setSelected] = useState<HarnessId>(TAB_ORDER[0] ?? 'claude');
  const [screen, setScreen] = useState<Screen>({ kind: 'tabs' });
  const rootRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  const refresh = useAsyncAction(() => backend.refresh());
  const quit = useAsyncAction(() => backend.quit());
  const hide = useAsyncAction(() => backend.hideFlyout());

  const ctx = useMemo<ViewContext>(
    () => ({ nowSecs, settings: settingsApi.settings }),
    [nowSecs, settingsApi.settings],
  );
  const snapshot = snapshotState.snapshot;
  const views = useMemo(() => viewsInTabOrder(snapshot), [snapshot]);
  const visible = preview ? (settingsApi.loaded ? activeViews(views, ctx) : []) : views;
  const selected =
    visible.find((candidate) => candidate.id === preferred)?.id ?? visible[0]?.id ?? preferred;
  // Before the first snapshot there is nothing to compare with, so nothing is announced.
  const live = useAnnouncer(snapshot ? snapshot.views : NO_VIEWS, ctx);
  const tray = useMemo(
    () => traySummary(preview ? activeViews(views, ctx) : views, ctx),
    [preview, views, ctx],
  );
  const view = visible.find((candidate) => candidate.id === selected);

  useFlyoutHeight(backend, rootRef, scrollRef);

  // A new panel starts at the top, whichever tab or screen it replaced.
  const toTop = useCallback(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
  }, []);
  const [detailsEpoch, setDetailsEpoch] = useState(0);
  const selectTab = useCallback(
    (id: HarnessId) => {
      const select = () => {
        setSelected(id);
        setDetailsEpoch((epoch) => epoch + 1);
        toTop();
      };
      if (id !== selected && request) request(select);
      else select();
    },
    [toTop, selected, request],
  );
  const openSettings = useCallback(
    (section?: SettingsSection) => {
      const open = () => {
        setScreen({ kind: 'settings', section: section ?? null });
        toTop();
      };
      if (screen.kind === 'tabs' && request) request(open);
      else open();
    },
    [toTop, screen.kind, request],
  );
  const closeSettings = useCallback(() => {
    setScreen({ kind: 'tabs' });
    toTop();
  }, [toTop]);
  const navigation = useMemo(() => ({ openSettings }), [openSettings]);

  // Leaving settings puts focus back on the tab the user came from.
  const wasSettings = useRef(false);
  useEffect(() => {
    if (wasSettings.current && screen.kind === 'tabs') focusTab(selected);
    wasSettings.current = screen.kind === 'settings';
  }, [screen.kind, selected]);

  // A manual refresh during a cool-down is dropped by the scheduler; say why instead of nothing.
  const [blocked, setBlocked] = useState('');
  useEffect(() => {
    if (blocked === '') return;
    const timer = setTimeout(() => setBlocked(''), 8000);
    return () => clearTimeout(timer);
  }, [blocked]);
  const refreshNow = () => {
    const left = view ? cooldownLeft(view, nowSecs) : null;
    setBlocked(
      view && left !== null
        ? t('cooldown.refresh_blocked', { name: harnessName(view.id), wait: waitText(left) })
        : '',
    );
    void refresh.run();
  };

  const { run: refreshAll } = refresh;
  const subscribeShown = useCallback(
    (callback: () => void) => backend.onFlyoutShown(callback),
    [backend],
  );
  useSubscription(subscribeShown, () => {
    sync();
    if (screen.kind === 'tabs') focusTab(selected);
    else document.getElementById('settings-title')?.focus();
    void refreshAll();
  });

  useEscape(() => {
    if (screen.kind === 'settings') request ? request(closeSettings) : closeSettings();
    else void hide.run();
  });

  const shellFailure = refresh.error ?? quit.error ?? hide.error;
  const loadFailure =
    snapshotState.error && snapshot === null ? describeError(snapshotState.error) : null;
  const inSettings = screen.kind === 'settings';
  return (
    <BackendContext value={backend}>
      <UpdateContext value={updateApi}>
        <SettingsContext value={settingsApi}>
          <NavigationContext value={navigation}>
            <div ref={rootRef} className="flyout">
              <Header
                tray={
                  tray
                    ? {
                        pct: sharePct(tray.summary.availablePct, showsUsed(ctx.settings)),
                        status: tray.summary.status,
                        showUsed: showsUsed(ctx.settings),
                      }
                    : null
                }
                settingsOpen={inSettings}
                onRefresh={refreshNow}
                onToggleSettings={
                  inSettings
                    ? () => (request ? request(closeSettings) : closeSettings())
                    : () => openSettings()
                }
              />
              {!inSettings && !preview && (
                <TabStrip views={views} selected={selected} ctx={ctx} onSelect={selectTab} />
              )}
              <div
                ref={scrollRef}
                className="fly-body"
                {...(inSettings
                  ? { role: 'region', 'aria-labelledby': 'settings-title' }
                  : preview
                    ? { role: 'region', 'aria-label': 'Monitoramento' }
                    : { role: 'tabpanel', id: PANEL_ID, 'aria-labelledby': tabNameId(selected) })}
              >
                <div className="fly-content">
                  {!inSettings && <UpdateBanner onOpen={() => openSettings('general')} />}
                  {!inSettings && preview && (
                    <PreviewNavigation
                      views={views}
                      active={visible}
                      selected={selected}
                      ctx={ctx}
                      onSelect={selectTab}
                      loaded={settingsApi.loaded}
                      onSettings={() => openSettings()}
                    />
                  )}
                  {!inSettings && !preview && (
                    <ProviderPicker
                      views={views}
                      selected={selected}
                      ctx={ctx}
                      onSelect={selectTab}
                    />
                  )}
                  {shellFailure && <ActionError error={shellFailure} />}
                  {snapshotState.eventsFailed && (
                    <Banner tone="warn">
                      <p>{t('load.events')}</p>
                    </Banner>
                  )}
                  {inSettings &&
                    (preview ? (
                      <PreviewSettings
                        views={views}
                        selected={selected}
                        ctx={ctx}
                        section={screen.section}
                        onBack={closeSettings}
                      />
                    ) : (
                      <SettingsView section={screen.section} onBack={closeSettings} />
                    ))}
                  {!inSettings && loadFailure && (
                    <section className="card notice" data-status="crit">
                      <h2>{t('load.title')}</h2>
                      <p>{loadFailure.body}</p>
                      <div className="actions">
                        <button
                          type="button"
                          className="btn primary"
                          onClick={snapshotState.reload}
                        >
                          {t('action.retry')}
                        </button>
                      </div>
                    </section>
                  )}
                  {!inSettings &&
                    !loadFailure &&
                    view &&
                    (preview ? (
                      <PreviewPanel view={view} ctx={ctx} epoch={detailsEpoch} />
                    ) : (
                      <HarnessPanel view={view} ctx={ctx} />
                    ))}
                </div>
              </div>
              <Footer
                readAt={latestReadAt(views)}
                nowSecs={nowSecs}
                onRefresh={refreshNow}
                onQuit={() => (request ? request(() => void quit.run()) : void quit.run())}
              />
              <LiveRegion message={blocked || live} />
            </div>
          </NavigationContext>
        </SettingsContext>
      </UpdateContext>
    </BackendContext>
  );
}
