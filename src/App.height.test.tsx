import { act, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { renderApp } from './test/helpers';

class FakeResizeObserver implements ResizeObserver {
  static all: FakeResizeObserver[] = [];

  constructor(private readonly callback: ResizeObserverCallback) {
    FakeResizeObserver.all.push(this);
  }

  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}

  fire(): void {
    this.callback([], this);
  }
}

const VISIBLE_BODY = 400;

/** jsdom has no layout: report what a real window would for the root and the scrolling body. */
function layout(box: { root: number; content: number }) {
  const isBody = (el: Element) => el.classList.contains('fly-body');
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
    return { height: this.classList.contains('flyout') ? box.root : 0 } as DOMRect;
  });
  vi.spyOn(Element.prototype, 'clientHeight', 'get').mockImplementation(function (this: Element) {
    return isBody(this) ? VISIBLE_BODY : 0;
  });
  vi.spyOn(Element.prototype, 'scrollHeight', 'get').mockImplementation(function (this: Element) {
    return isBody(this) ? box.content : 0;
  });
}

const heightCalls = (calls: readonly string[]) =>
  calls.filter((call) => call === 'set_flyout_height').length;
const settle = () => new Promise((resolve) => setTimeout(resolve, 140));

beforeEach(() => {
  FakeResizeObserver.all = [];
  vi.stubGlobal('ResizeObserver', FakeResizeObserver);
});

describe('the height protocol', () => {
  it('reports the natural height of header, tabs and the whole content, rounded up', async () => {
    const { backend } = renderApp();
    layout({ root: 580.4, content: 900 });
    await screen.findByRole('heading', { name: 'Sessão de 5 horas' });
    act(() => FakeResizeObserver.all[0]?.fire());
    // root 580.4 minus the 400 px window onto the body, plus its 900 px of content
    await vi.waitFor(() => expect(backend.lastFlyoutHeight).toBe(1081));
  });

  it('is one report for a burst of changes, and none for a repeat', async () => {
    const { backend } = renderApp();
    layout({ root: 580, content: 900 });
    await screen.findByRole('heading', { name: 'Sessão de 5 horas' });
    for (let i = 0; i < 5; i++) act(() => FakeResizeObserver.all[0]?.fire());
    await vi.waitFor(() => expect(backend.lastFlyoutHeight).toBe(1080));
    await settle();
    act(() => FakeResizeObserver.all[0]?.fire());
    await settle();
    expect(heightCalls(backend.calls)).toBe(1);
  });

  it('follows the content when it grows, even though the window is clamped', async () => {
    const box = { root: 580, content: 900 };
    const { backend } = renderApp();
    layout(box);
    await screen.findByRole('heading', { name: 'Sessão de 5 horas' });
    act(() => FakeResizeObserver.all[0]?.fire());
    await vi.waitFor(() => expect(backend.lastFlyoutHeight).toBe(1080));

    box.content = 1200;
    act(() => FakeResizeObserver.all[0]?.fire());
    await vi.waitFor(() => expect(backend.lastFlyoutHeight).toBe(1380));
  });

  it('never asks for a height before there is a layout to measure', async () => {
    const { backend } = renderApp();
    await screen.findByRole('heading', { name: 'Sessão de 5 horas' });
    await settle();
    expect(heightCalls(backend.calls)).toBe(0);
  });
});
