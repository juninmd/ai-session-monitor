// What the home shows: the views in tab order, the reachable ones, the selected provider, the tray
// summary the header prints, the line the announcer reads and the first-run state.
import { useMemo } from 'react';
import { describeError } from '../app/errorText';
import { latestReadAt, viewsInTabOrder } from '../app/placeholder';
import { traySummary } from '../app/tabModel';
import { useAnnouncer } from '../app/useAnnouncer';
import type { ViewContext } from '../app/windowModel';
import type { HarnessError, HarnessId, HarnessView, Snapshot } from '../lib/contract';
import { activeViews } from './model';

const NO_VIEWS: readonly HarnessView[] = [];

export interface HomeView {
  views: readonly HarnessView[];
  active: readonly HarnessView[];
  selected: HarnessId;
  view: HarnessView | undefined;
  tray: ReturnType<typeof traySummary>;
  readAt: number | null;
  /** What the live region says after a status change; empty on the first snapshot. */
  live: string;
  /** Why the first snapshot never arrived, in the flyout's words. */
  failure: string | null;
  /** Nothing readable is configured: the wizard takes the place of the surfaces. */
  firstRun: boolean;
}

export function useHomeView(input: {
  snapshot: Snapshot | null;
  loadError: HarnessError | null;
  loaded: boolean;
  ctx: ViewContext;
  preferred: HarnessId;
  /** The user already went through the wizard once in this session. */
  runDone: boolean;
}): HomeView {
  const { snapshot, loadError, loaded, ctx, preferred, runDone } = input;
  const views = useMemo(() => viewsInTabOrder(snapshot), [snapshot]);
  const active = useMemo(() => (loaded ? activeViews(views, ctx) : NO_VIEWS), [loaded, views, ctx]);
  const selected =
    active.find((candidate) => candidate.id === preferred)?.id ?? active[0]?.id ?? preferred;
  const tray = useMemo(() => traySummary(active, ctx), [active, ctx]);
  const live = useAnnouncer(snapshot ? snapshot.views : NO_VIEWS, ctx);
  const error = snapshot === null ? loadError : null;
  const failure = error === null ? null : describeError(error).body;
  return {
    views,
    active,
    selected,
    view: active.find((candidate) => candidate.id === selected),
    tray,
    readAt: latestReadAt(views),
    live,
    failure,
    firstRun: failure === null && loaded && active.length === 0 && !runDone,
  };
}
