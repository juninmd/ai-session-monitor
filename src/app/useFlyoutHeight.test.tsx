import { render } from '@testing-library/react';
import { useRef } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Backend } from '../lib/backend';
import { createMockBackend } from '../mock';
import { naturalHeight, useFlyoutHeight } from './useFlyoutHeight';

class FakeResizeObserver implements ResizeObserver {
  static instances: FakeResizeObserver[] = [];
  readonly targets: Element[] = [];
  disconnected = false;

  constructor(private readonly callback: ResizeObserverCallback) {
    FakeResizeObserver.instances.push(this);
  }

  observe(target: Element): void {
    this.targets.push(target);
  }

  unobserve(): void {}

  disconnect(): void {
    this.disconnected = true;
  }

  /** The browser told us the observed box changed. */
  fire(): void {
    this.callback([], this);
  }
}

function Probe({
  backend,
  measure,
  ready = true,
}: {
  backend: Backend;
  measure: () => number;
  ready?: boolean;
}) {
  const root = useRef<HTMLDivElement>(null);
  const scroller = useRef<HTMLDivElement>(null);
  useFlyoutHeight(backend, root, scroller, measure, ready);
  return (
    <div ref={root} data-testid="root">
      <div ref={scroller}>
        <p>content</p>
      </div>
    </div>
  );
}

beforeEach(() => {
  FakeResizeObserver.instances = [];
  vi.stubGlobal('ResizeObserver', FakeResizeObserver);
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

function setup(initial: number) {
  const backend = createMockBackend();
  const report = vi.spyOn(backend, 'setFlyoutHeight');
  const state = { height: initial };
  const view = render(<Probe backend={backend} measure={() => state.height} />);
  const observer = FakeResizeObserver.instances[0];
  if (!observer) throw new Error('the hook never created an observer');
  return { backend, report, state, observer, ...view };
}

describe('useFlyoutHeight', () => {
  it('reports the measured height once the layout has been quiet for 50 ms', () => {
    const { report } = setup(640);
    expect(report).not.toHaveBeenCalled();
    vi.advanceTimersByTime(49);
    expect(report).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(report).toHaveBeenCalledExactlyOnceWith(640);
  });

  it('coalesces a burst of resizes into one report with the latest height', () => {
    const { report, state, observer } = setup(500);
    for (const height of [510, 520, 530, 540]) {
      state.height = height;
      observer.fire();
      vi.advanceTimersByTime(10);
    }
    expect(report).not.toHaveBeenCalled();
    vi.advanceTimersByTime(50);
    expect(report).toHaveBeenCalledExactlyOnceWith(540);
  });

  it('does not repeat a height the shell already has, and reports a change', () => {
    const { report, state, observer } = setup(600);
    vi.advanceTimersByTime(50);
    observer.fire();
    vi.advanceTimersByTime(50);
    expect(report).toHaveBeenCalledTimes(1);
    state.height = 720;
    observer.fire();
    vi.advanceTimersByTime(50);
    expect(report).toHaveBeenCalledTimes(2);
    expect(report).toHaveBeenLastCalledWith(720);
  });

  it.each([[0], [-10], [Number.NaN]])('never reports a height of %s', (height) => {
    const { report } = setup(height);
    vi.advanceTimersByTime(200);
    expect(report).not.toHaveBeenCalled();
  });

  it('tries again after a failed report even when the height did not change', async () => {
    const { report, observer } = setup(600);
    report.mockRejectedValueOnce(new Error('window is gone'));
    await vi.advanceTimersByTimeAsync(50);
    expect(report).toHaveBeenCalledTimes(1);
    observer.fire();
    await vi.advanceTimersByTimeAsync(50);
    expect(report).toHaveBeenCalledTimes(2);
  });

  it('watches the panel and its content, because the panel alone stops growing when clamped', () => {
    const { observer, getByTestId } = setup(600);
    const root = getByTestId('root');
    expect(observer.targets).toContain(root);
    expect(observer.targets).toContain(root.firstElementChild?.firstElementChild);
  });

  it('stops watching and drops a pending report on unmount', () => {
    const { report, observer, unmount } = setup(600);
    unmount();
    vi.advanceTimersByTime(200);
    expect(observer.disconnected).toBe(true);
    expect(report).not.toHaveBeenCalled();
  });

  it('waits until ready is true before observing and reporting height', () => {
    const backend = createMockBackend();
    const report = vi.spyOn(backend, 'setFlyoutHeight');
    const { rerender } = render(<Probe backend={backend} measure={() => 540} ready={false} />);
    vi.advanceTimersByTime(200);
    expect(FakeResizeObserver.instances).toHaveLength(0);
    expect(report).not.toHaveBeenCalled();

    rerender(<Probe backend={backend} measure={() => 540} ready={true} />);
    vi.advanceTimersByTime(50);
    expect(report).toHaveBeenCalledExactlyOnceWith(540);
  });
});

describe('naturalHeight', () => {
  function box(sizes: { client: number; scroll: number; rect?: number }) {
    const root = document.createElement('div');
    const scroller = document.createElement('div');
    root.append(scroller);
    root.getBoundingClientRect = () => ({ height: sizes.rect ?? 0 }) as DOMRect;
    Object.defineProperty(scroller, 'clientHeight', { value: sizes.client });
    Object.defineProperty(scroller, 'scrollHeight', { value: sizes.scroll });
    return { root, scroller };
  }

  it('adds the chrome around the body to the full scrollable content', () => {
    // 100 px of header, tabs and footer around a body that is clamped to 200 px.
    const { root, scroller } = box({ rect: 300, client: 200, scroll: 480 });
    expect(naturalHeight(root, scroller)).toBe(580);
  });

  it('rounds up so the shell never cuts the last pixel', () => {
    const { root, scroller } = box({ rect: 300.2, client: 200, scroll: 480.4 });
    expect(naturalHeight(root, scroller)).toBe(581);
  });

  it('equals the root height when the content fits', () => {
    const { root, scroller } = box({ rect: 420, client: 300, scroll: 300 });
    expect(naturalHeight(root, scroller)).toBe(420);
  });
});
