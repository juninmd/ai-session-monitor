import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError, type Unsubscribe } from '../lib/backend';
import { problem } from '../mock/fixtures';
import { freezeClock, NOW } from '../test/helpers';
import { useAsyncAction } from './useAsyncAction';
import { useEscape } from './useEscape';
import { TICK_MS, useNowSecs } from './useNowSecs';
import { useSubscription } from './useSubscription';

/** A subscription whose registration and delivery the test controls. */
function channel<T>() {
  const callbacks: Array<(value: T) => void> = [];
  const stop = vi.fn();
  const subscribe = vi.fn((callback: (value: T) => void): Promise<Unsubscribe> => {
    callbacks.push(callback);
    return Promise.resolve(stop);
  });
  const send = (value: T) => {
    for (const callback of callbacks) callback(value);
  };
  return { subscribe, stop, callbacks, send };
}

describe('useSubscription', () => {
  it('delivers events to the latest handler without resubscribing', () => {
    const events = channel<number>();
    const first = vi.fn();
    const second = vi.fn();
    const { rerender } = renderHook(({ handler }) => useSubscription(events.subscribe, handler), {
      initialProps: { handler: first },
    });
    rerender({ handler: second });
    events.send(7);
    expect(events.subscribe).toHaveBeenCalledTimes(1);
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledExactlyOnceWith(7);
  });

  it('unsubscribes on unmount and ignores late events', async () => {
    const events = channel<number>();
    const handler = vi.fn();
    const { unmount } = renderHook(() => useSubscription(events.subscribe, handler));
    await act(async () => {});
    unmount();
    events.send(1);
    expect(events.stop).toHaveBeenCalledTimes(1);
    expect(handler).not.toHaveBeenCalled();
  });

  it('releases a subscription that only resolves after the component is gone', async () => {
    const stop = vi.fn();
    let resolve: (stop: Unsubscribe) => void = () => {};
    const subscribe = vi.fn(
      () =>
        new Promise<Unsubscribe>((done) => {
          resolve = done;
        }),
    );
    const { unmount } = renderHook(() => useSubscription(subscribe, vi.fn()));
    unmount();
    resolve(stop);
    await act(async () => {});
    expect(stop).toHaveBeenCalledTimes(1);
  });

  it('keeps exactly one live listener under StrictMode double mounting', async () => {
    const events = channel<number>();
    const handler = vi.fn();
    renderHook(() => useSubscription(events.subscribe, handler), { reactStrictMode: true });
    await act(async () => {});
    expect(events.callbacks).toHaveLength(2);
    events.send(3);
    expect(handler).toHaveBeenCalledExactlyOnceWith(3);
    expect(events.stop).toHaveBeenCalledTimes(1);
  });

  it('reports a subscription that fails, and stays quiet if nobody is listening anymore', async () => {
    const boom = new Error('bridge down');
    const subscribe = vi.fn(() => Promise.reject(boom));
    const onError = vi.fn();
    renderHook(() => useSubscription(subscribe, vi.fn(), onError));
    await act(async () => {});
    expect(onError).toHaveBeenCalledExactlyOnceWith(boom);

    const late = vi.fn();
    let fail: (reason: unknown) => void = () => {};
    const slow = vi.fn(
      () =>
        new Promise<Unsubscribe>((_resolve, reject) => {
          fail = reject;
        }),
    );
    const { unmount } = renderHook(() => useSubscription(slow, vi.fn(), late));
    unmount();
    fail(boom);
    await act(async () => {});
    expect(late).not.toHaveBeenCalled();
  });
});

describe('useAsyncAction', () => {
  it('runs the action, reports success and clears an earlier error', async () => {
    const action = vi
      .fn()
      .mockRejectedValueOnce(new BackendError(problem('timeout', {}, true)))
      .mockResolvedValueOnce('done');
    const { result } = renderHook(() => useAsyncAction(action));

    await act(async () => {
      expect(await result.current.run()).toBe(false);
    });
    expect(result.current.error).toEqual(problem('timeout', {}, true));

    await act(async () => {
      expect(await result.current.run()).toBe(true);
    });
    expect(result.current.error).toBeNull();
  });

  it('is pending while the action runs', async () => {
    let release: () => void = () => {};
    const action = vi.fn(
      () =>
        new Promise<void>((done) => {
          release = done;
        }),
    );
    const { result } = renderHook(() => useAsyncAction(action));
    let finished: Promise<boolean> | undefined;
    act(() => {
      finished = result.current.run();
    });
    expect(result.current.pending).toBe(true);
    await act(async () => {
      release();
      await finished;
    });
    expect(result.current.pending).toBe(false);
  });

  it('collapses an unknown failure to an internal error and never exposes its text', async () => {
    const action = vi.fn().mockRejectedValue(new Error('token=s3cret-value leaked'));
    const { result } = renderHook(() => useAsyncAction(action));
    await act(async () => {
      await result.current.run();
    });
    expect(result.current.error).toEqual(problem('internal'));
    expect(JSON.stringify(result.current.error)).not.toContain('s3cret');
  });

  it('forwards the arguments and uses the newest action with a stable run', async () => {
    const first = vi.fn().mockResolvedValue(undefined);
    const second = vi.fn().mockResolvedValue(undefined);
    const { result, rerender } = renderHook(({ action }) => useAsyncAction(action), {
      initialProps: { action: first },
    });
    const run = result.current.run;
    rerender({ action: second });
    await act(async () => {
      await result.current.run('a', 2);
    });
    expect(result.current.run).toBe(run);
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledExactlyOnceWith('a', 2);
  });

  it('lets the caller dismiss the error', async () => {
    const { result } = renderHook(() => useAsyncAction(vi.fn().mockRejectedValue(new Error('x'))));
    await act(async () => {
      await result.current.run();
    });
    expect(result.current.error).not.toBeNull();
    act(() => result.current.clear());
    expect(result.current.error).toBeNull();
  });
});

describe('useNowSecs', () => {
  it('follows the clock on a 20 second cadence', () => {
    freezeClock(['setInterval', 'clearInterval']);
    const { result } = renderHook(() => useNowSecs());
    expect(result.current.nowSecs).toBe(NOW);
    act(() => vi.advanceTimersByTime(TICK_MS - 1));
    expect(result.current.nowSecs).toBe(NOW);
    act(() => vi.advanceTimersByTime(1));
    expect(result.current.nowSecs).toBe(NOW + 20);
  });

  it('syncs on demand and stops ticking when unmounted', () => {
    freezeClock(['setInterval', 'clearInterval']);
    const { result, unmount } = renderHook(() => useNowSecs());
    act(() => {
      vi.setSystemTime((NOW + 5) * 1000);
      result.current.sync();
    });
    expect(result.current.nowSecs).toBe(NOW + 5);
    unmount();
    expect(vi.getTimerCount()).toBe(0);
  });
});

describe('useEscape', () => {
  const press = (init: KeyboardEventInit = {}) => {
    const event = new KeyboardEvent('keydown', {
      key: 'Escape',
      bubbles: true,
      cancelable: true,
      ...init,
    });
    document.body.dispatchEvent(event);
    return event;
  };

  it('fires once per press, from anywhere in the page, and marks the key handled', () => {
    const handler = vi.fn();
    renderHook(() => useEscape(handler));
    const event = press();
    expect(handler).toHaveBeenCalledTimes(1);
    expect(event.defaultPrevented).toBe(true);
  });

  it('ignores other keys', () => {
    const handler = vi.fn();
    renderHook(() => useEscape(handler));
    press({ key: 'Enter' });
    press({ key: 'Esc' });
    expect(handler).not.toHaveBeenCalled();
  });

  it('leaves Esc to an input method that is composing', () => {
    const handler = vi.fn();
    renderHook(() => useEscape(handler));
    press({ isComposing: true });
    expect(handler).not.toHaveBeenCalled();
  });

  it('leaves Esc alone when something closer already handled it', () => {
    const closer = (event: Event) => event.preventDefault();
    document.addEventListener('keydown', closer);
    const handler = vi.fn();
    renderHook(() => useEscape(handler));
    press();
    document.removeEventListener('keydown', closer);
    expect(handler).not.toHaveBeenCalled();
  });

  it('calls the newest handler and stops listening on unmount', () => {
    const first = vi.fn();
    const second = vi.fn();
    const { rerender, unmount } = renderHook(({ handler }) => useEscape(handler), {
      initialProps: { handler: first },
    });
    rerender({ handler: second });
    press();
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledTimes(1);
    unmount();
    press();
    expect(second).toHaveBeenCalledTimes(1);
  });
});
