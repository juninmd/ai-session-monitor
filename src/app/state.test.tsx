import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../lib/backend';
import type { Snapshot } from '../lib/contract';
import { createMockBackend } from '../mock';
import { problem } from '../mock/fixtures';
import { NOW } from '../test/helpers';
import { useSnapshotState } from './useSnapshotState';

const snapshotAt = (generatedAt: number): Snapshot => ({ generatedAt, views: [] });

describe('useSnapshotState', () => {
  it('loads the first snapshot', async () => {
    const backend = createMockBackend({ scenario: 'healthy', now: () => NOW });
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.snapshot).not.toBeNull());
    expect(result.current.snapshot?.views).toHaveLength(9);
    expect(result.current.error).toBeNull();
  });

  it('keeps an event that arrived before a slower first read', async () => {
    const backend = createMockBackend();
    let deliver: (snapshot: Snapshot) => void = () => {};
    let resolveFirst: (snapshot: Snapshot) => void = () => {};
    vi.spyOn(backend, 'onSnapshot').mockImplementation((callback) => {
      deliver = callback;
      return Promise.resolve(() => {});
    });
    vi.spyOn(backend, 'getSnapshot').mockImplementation(
      () =>
        new Promise<Snapshot>((resolve) => {
          resolveFirst = resolve;
        }),
    );
    const { result } = renderHook(() => useSnapshotState(backend));
    await act(async () => {});
    act(() => deliver(snapshotAt(200)));
    await act(async () => resolveFirst(snapshotAt(100)));
    expect(result.current.snapshot?.generatedAt).toBe(200);
  });

  it('reports a failed first read as a mapped error and recovers on reload', async () => {
    const backend = createMockBackend({ now: () => NOW });
    vi.spyOn(backend, 'getSnapshot').mockRejectedValueOnce(
      new BackendError(problem('timeout', {}, true)),
    );
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.error).toEqual(problem('timeout', {}, true)));
    expect(result.current.snapshot).toBeNull();
    act(() => result.current.reload());
    await waitFor(() => expect(result.current.snapshot).not.toBeNull());
    expect(result.current.error).toBeNull();
  });

  it('collapses an unexpected failure to an internal error', async () => {
    const backend = createMockBackend();
    vi.spyOn(backend, 'getSnapshot').mockRejectedValue(new Error('disk path C:\\private'));
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.error).not.toBeNull());
    expect(result.current.error).toEqual(problem('internal'));
  });

  it('flags a broken event bridge, because the numbers would silently stop moving', async () => {
    const backend = createMockBackend();
    vi.spyOn(backend, 'onSnapshot').mockRejectedValue(new Error('no bridge'));
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.eventsFailed).toBe(true));
  });

  it('surfaces a malformed snapshot event as a protocol error and a broken bridge', async () => {
    const backend = createMockBackend();
    let invalid: () => void = () => {};
    vi.spyOn(backend, 'getSnapshot').mockRejectedValue(new Error('x'));
    vi.spyOn(backend, 'onSnapshot').mockImplementation(async (_cb, onInvalid) => {
      invalid = onInvalid ?? (() => {});
      return () => {};
    });
    const { result } = renderHook(() => useSnapshotState(backend));
    await act(async () => {});
    act(() => invalid());
    expect(result.current.eventsFailed).toBe(true);
    expect(result.current.error?.code).toBe('protocol_error');
  });

  it('follows later snapshots', async () => {
    const backend = createMockBackend({ now: () => NOW });
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.snapshot).not.toBeNull());
    await act(async () => {
      await backend.refresh('codex');
    });
    await waitFor(() => expect(result.current.snapshot?.generatedAt).toBe(NOW));
  });
});

describe('useSnapshotState listener before read', () => {
  it('does not lose a snapshot published between the first read and the listener', async () => {
    const backend = createMockBackend();
    let hub = snapshotAt(100);
    let register: () => void = () => {};
    vi.spyOn(backend, 'getSnapshot').mockImplementation(async () => hub);
    vi.spyOn(backend, 'onSnapshot').mockImplementation(
      () =>
        new Promise((resolve) => {
          register = () => resolve(() => {});
        }),
    );
    const { result } = renderHook(() => useSnapshotState(backend));
    await act(async () => {});
    // The shell publishes while the listener is still being registered: nobody hears it, and it
    // emits only on change, so it never repeats it.
    hub = snapshotAt(200);
    await act(async () => register());
    await waitFor(() => expect(result.current.snapshot?.generatedAt).toBe(200));
  });

  it('keeps the newer snapshot when a read answers after an event that was already newer', async () => {
    const backend = createMockBackend();
    let emit: (s: Snapshot) => void = () => {};
    vi.spyOn(backend, 'getSnapshot').mockResolvedValue(snapshotAt(100));
    vi.spyOn(backend, 'onSnapshot').mockImplementation(async (cb) => {
      emit = cb;
      return () => {};
    });
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.snapshot?.generatedAt).toBe(100));
    act(() => emit(snapshotAt(300)));
    act(() => emit(snapshotAt(200)));
    expect(result.current.snapshot?.generatedAt).toBe(300);
    vi.spyOn(backend, 'getSnapshot').mockResolvedValue(snapshotAt(250));
    await act(async () => result.current.reload());
    expect(result.current.snapshot?.generatedAt).toBe(300);
  });

  it('keeps an event over a read from the same second', async () => {
    const backend = createMockBackend();
    let emit: (s: Snapshot) => void = () => {};
    vi.spyOn(backend, 'getSnapshot').mockResolvedValue(snapshotAt(100));
    vi.spyOn(backend, 'onSnapshot').mockImplementation(async (cb) => {
      emit = cb;
      return () => {};
    });
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.snapshot).not.toBeNull());
    const fresh: Snapshot = { generatedAt: 200, views: [] };
    act(() => emit(fresh));
    vi.spyOn(backend, 'getSnapshot').mockResolvedValue({ generatedAt: 200, views: [] });
    await act(async () => result.current.reload());
    expect(result.current.snapshot).toBe(fresh);
  });

  it('lets a later event from the same second replace the current one', async () => {
    const backend = createMockBackend();
    let emit: (s: Snapshot) => void = () => {};
    vi.spyOn(backend, 'getSnapshot').mockResolvedValue(snapshotAt(100));
    vi.spyOn(backend, 'onSnapshot').mockImplementation(async (cb) => {
      emit = cb;
      return () => {};
    });
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.snapshot).not.toBeNull());
    const first: Snapshot = { generatedAt: 200, views: [] };
    const second: Snapshot = { generatedAt: 200, views: [] };
    act(() => emit(first));
    act(() => emit(second));
    expect(result.current.snapshot).toBe(second);
  });

  it('still reads the first snapshot when the event bridge could not be set up', async () => {
    const backend = createMockBackend({ now: () => NOW });
    vi.spyOn(backend, 'onSnapshot').mockRejectedValue(new Error('no bridge'));
    const { result } = renderHook(() => useSnapshotState(backend));
    await waitFor(() => expect(result.current.snapshot).not.toBeNull());
    expect(result.current.eventsFailed).toBe(true);
  });
});
