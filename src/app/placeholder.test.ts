import { describe, expect, it } from 'vitest';
import { claudeView, codexView } from '../mock/builders';
import { viewOf } from '../mock/fixtures';
import { envOf, NOW } from '../test/helpers';
import { TAB_ORDER } from './harness';
import { latestReadAt, placeholderView, viewsInTabOrder } from './placeholder';

describe('viewsInTabOrder', () => {
  it('shows every tab as loading before the first snapshot', () => {
    const views = viewsInTabOrder(null);
    expect(views.map((v) => v.id)).toEqual([...TAB_ORDER]);
    expect(views.every((v) => v.state === 'loading')).toBe(true);
  });

  it('follows the tab order whatever order the backend sent', () => {
    const env = envOf();
    const shuffled = [codexView(env, 'ok'), claudeView(env, 'ok')];
    const views = viewsInTabOrder({ generatedAt: NOW, views: shuffled });
    expect(views.map((v) => `${v.id}:${v.state}`)).toEqual([
      'claude:ready',
      'codex:ready',
      'antigravity:loading',
      'opencode:loading',
      'kimi:loading',
      'zai:loading',
      'openrouter:loading',
      'deepseek:loading',
      'deepinfra:loading',
      'moonshot:loading',
      'novita:loading',
      'ollama:loading',
      'minimax:loading',
      'litellm:loading',
    ]);
  });

  it('gives a placeholder that is valid and empty', () => {
    expect(placeholderView('litellm')).toEqual(viewOf('litellm', { state: 'loading' }));
  });
});

describe('latestReadAt', () => {
  it('is the newest reading of any source', () => {
    const views = [
      viewOf('claude', { readAt: NOW - 240 }),
      viewOf('codex', { readAt: NOW - 20 }),
      viewOf('litellm', { readAt: null }),
    ];
    expect(latestReadAt(views)).toBe(NOW - 20);
  });

  it('is null when nothing was read yet', () => {
    expect(latestReadAt([])).toBeNull();
    expect(latestReadAt([viewOf('claude')])).toBeNull();
  });
});
