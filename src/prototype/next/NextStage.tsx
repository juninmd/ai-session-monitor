// The stage around the proposal: a surface switch, the scenario links the current prototype
// already uses, the fabricated flyout itself, and the command palette. Nothing here touches a real
// profile, credential or network; the backend is the in-memory mock.
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { App } from '../../App';
import { BackendContext } from '../../app/backendContext';
import { DraftProvider } from '../../app/draftContext';
import type { SettingsSection } from '../../app/harness';
import { NavigationContext } from '../../app/navigation';
import { latestReadAt, viewsInTabOrder } from '../../app/placeholder';
import { sharePct, showsUsed } from '../../app/quota';
import { SettingsContext } from '../../app/settingsContext';
import { traySummary } from '../../app/tabModel';
import { useAsyncAction } from '../../app/useAsyncAction';
import { useNowSecs } from '../../app/useNowSecs';
import { useSettingsState } from '../../app/useSettingsState';
import { useSnapshotState } from '../../app/useSnapshotState';
import { UpdateContext, useUpdateState } from '../../app/useUpdateState';
import type { ViewContext } from '../../app/windowModel';
import type { HarnessId } from '../../lib/contract';
import type { MockBackend } from '../../mock';
import { activeViews } from '../model';
import { CommandPalette } from './CommandPalette';
import { Fly, type FlyProps, type Screen, type Surface } from './Fly';
import { commandsFor } from './paletteModel';

/** The comparison is a reading of the two interfaces, not a screen that opens settings. */
const SURFACES: readonly { id: Surface; label: string; hint: string }[] = [
  { id: 'triagem', label: 'Triagem', hint: 'preciso agir?' },
  { id: 'horizonte', label: 'Horizonte', hint: 'chego até a renovação?' },
  { id: 'comando', label: 'Comando', hint: 'Ctrl+K' },
  { id: 'primeiro', label: 'Primeiro uso', hint: 'conectar fontes' },
  { id: 'canais', label: 'Canais de cota', hint: 'o que ainda dá para ler' },
  { id: 'comparar', label: 'Comparar', hint: 'lado a lado' },
];

const SCENARIOS: readonly { name: string; label: string }[] = [
  { name: 'demo', label: 'Dia a dia' },
  { name: 'critical', label: 'Uso crítico' },
  { name: 'fresh', label: 'Sem conexões' },
  { name: 'errors', label: 'Falhas' },
  { name: 'stale', label: 'Desatualizado' },
  { name: 'loading', label: 'Carregando' },
];

export function NextStage({ backend }: { backend: MockBackend }) {
  const snapshots = useSnapshotState(backend);
  const settingsApi = useSettingsState(backend);
  const updateApi = useUpdateState(backend);
  const { nowSecs } = useNowSecs();
  const [surface, setSurface] = useState<Surface>('triagem');
  const [preferred, setPreferred] = useState<HarnessId>('codex');
  const [screen, setScreen] = useState<Screen>({ kind: 'surface' });
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [trend, setTrend] = useState(false);
  const [notice, setNotice] = useState('');
  const topRef = useRef<HTMLDivElement>(null);
  const openerRef = useRef<Element | null>(null);

  const views = useMemo(() => viewsInTabOrder(snapshots.snapshot), [snapshots.snapshot]);
  const ctx = useMemo<ViewContext>(
    () => ({ nowSecs, settings: settingsApi.settings }),
    [nowSecs, settingsApi.settings],
  );
  const active = useMemo(
    () => (settingsApi.loaded ? activeViews(views, ctx) : []),
    [settingsApi.loaded, views, ctx],
  );
  const selected =
    active.find((candidate) => candidate.id === preferred)?.id ?? active[0]?.id ?? preferred;
  const view = active.find((candidate) => candidate.id === selected);
  const tray = useMemo(() => traySummary(active, ctx), [active, ctx]);
  const used = showsUsed(ctx.settings);
  const trayProp = tray
    ? {
        pct: sharePct(tray.summary.availablePct, used),
        status: tray.summary.status,
        showUsed: used,
      }
    : null;
  const { run: runRefresh } = useAsyncAction((id?: HarnessId) => backend.refresh(id));

  const toTop = useCallback(() => {
    if (topRef.current) topRef.current.scrollTop = 0;
  }, []);
  const openPalette = useCallback(() => {
    openerRef.current = document.activeElement;
    setPaletteOpen(true);
  }, []);
  const closePalette = useCallback(() => {
    setPaletteOpen(false);
    const opener = openerRef.current;
    if (opener instanceof HTMLElement) opener.focus();
  }, []);

  const select = useCallback(
    (id: HarnessId) => {
      setPreferred(id);
      setScreen({ kind: 'surface' });
      toTop();
    },
    [toTop],
  );
  const openSettings = useCallback(
    (section: SettingsSection | null) => {
      setScreen({ kind: 'settings', section });
      toTop();
    },
    [toTop],
  );
  const closeSettings = useCallback(() => {
    setScreen({ kind: 'surface' });
    toTop();
  }, [toTop]);
  // Selecting from the palette lands on a list; the comparison pane is a reading, not a target.
  const jumpTo = useCallback(
    (id: HarnessId) => {
      setPreferred(id);
      setScreen({ kind: 'surface' });
      setSurface((current) => (current === 'comparar' ? current : 'triagem'));
      toTop();
    },
    [toTop],
  );

  const commands = useMemo(
    () =>
      commandsFor({
        views: active,
        ctx,
        select: jumpTo,
        refresh: (id) => void runRefresh(id),
        openSettings: (section) => openSettings(section ?? 'general'),
        compare: () => setSurface('comparar'),
        copySummary: (text) => {
          const written = navigator.clipboard?.writeText(text);
          if (written === undefined) {
            setNotice('Não foi possível copiar o resumo.');
            return;
          }
          void written.then(
            () => setNotice('Resumo das fontes copiado para a área de transferência.'),
            () => setNotice('Não foi possível copiar o resumo.'),
          );
        },
        scenario: (name) => {
          const url = new URL(window.location.href);
          url.searchParams.set('mock', name);
          window.location.assign(url);
        },
      }),
    [active, ctx, jumpTo, openSettings, runRefresh],
  );

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        openerRef.current = document.activeElement;
        setPaletteOpen(true);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
  useEffect(() => {
    if (notice === '') return;
    const timer = setTimeout(() => setNotice(''), 6000);
    return () => clearTimeout(timer);
  }, [notice]);

  const navigation = useMemo(
    () => ({ openSettings: (section?: SettingsSection) => openSettings(section ?? 'general') }),
    [openSettings],
  );
  // Opening settings leaves the comparison: two SettingsView instances in one document share
  // element ids, which would break every label that points at a control.
  const comparing = surface === 'comparar' && screen.kind === 'surface';
  const flyProps: Omit<FlyProps, 'surface'> = {
    screen,
    views,
    active,
    selected,
    view,
    ctx,
    tray: trayProp,
    readAt: latestReadAt(views),
    nowSecs,
    trend,
    onTrend: setTrend,
    onSelect: select,
    onRefresh: () => void runRefresh(),
    onToggleSettings: () =>
      setScreen((current) =>
        current.kind === 'settings' ? { kind: 'surface' } : { kind: 'settings', section: null },
      ),
    onOpenPalette: openPalette,
    onOpenSettings: openSettings,
    onCloseSettings: closeSettings,
    onFinishOnboarding: () => {
      setSurface('triagem');
      toTop();
    },
    onConfigure: () => {
      setSurface('primeiro');
      toTop();
    },
    commands,
  };

  return (
    <main className="nx-stage" data-comparing={comparing ? 'true' : undefined}>
      <div className="preview-caption">
        <span className="preview-tag">Proposta de UI/UX</span>
        <small>Prévia local · dados fictícios</small>
      </div>
      <nav className="nx-surfaces" aria-label="Superfícies propostas">
        {SURFACES.map((item) => (
          <button
            key={item.id}
            type="button"
            aria-pressed={surface === item.id}
            onClick={() => {
              setSurface(item.id);
              setScreen({ kind: 'surface' });
              toTop();
            }}
          >
            {item.label}
            <small>{item.hint}</small>
          </button>
        ))}
      </nav>
      <nav className="preview-scenarios" aria-label="Cenários de demonstração">
        {SCENARIOS.map((scenario) => (
          <a key={scenario.name} href={`?prototype=next&mock=${scenario.name}`}>
            {scenario.label}
          </a>
        ))}
      </nav>

      <BackendContext value={backend}>
        <UpdateContext value={updateApi}>
          <SettingsContext value={settingsApi}>
            <DraftProvider>
              <NavigationContext value={navigation}>
                <div ref={topRef}>
                  {comparing ? (
                    <div className="nx-compare">
                      <section className="nx-compare-col" aria-label="Interface atual">
                        <h2 className="nx-compare-head">
                          Hoje · {active.length} providers em grade, previsão só no detalhe
                        </h2>
                        <App backend={backend} preview />
                      </section>
                      <section className="nx-compare-col" aria-label="Proposta">
                        <h2 className="nx-compare-head">
                          Proposta · ordenado por urgência, previsão no cartão
                        </h2>
                        <Fly {...flyProps} surface="triagem" />
                      </section>
                    </div>
                  ) : (
                    <Fly {...flyProps} surface={surface} />
                  )}
                </div>
              </NavigationContext>
            </DraftProvider>
          </SettingsContext>
        </UpdateContext>
      </BackendContext>

      <div className="nx-below">
        <p className="nx-meta">
          <kbd>Ctrl</kbd>+<kbd>K</kbd> abre o comando rápido · setas navegam a lista ·{' '}
          <kbd>Esc</kbd> fecha. Todo número vem das mesmas regras de uso de{' '}
          <code>src/lib/usage.ts</code>.
        </p>
        <p className="nx-meta" role="status">
          {notice}
        </p>
      </div>

      {paletteOpen && <CommandPalette commands={commands} onClose={closePalette} />}
    </main>
  );
}
