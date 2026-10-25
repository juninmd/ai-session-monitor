import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DraftProvider, useDraftGuard } from './draftContext';

describe('the draft registry', () => {
  // A parent re-render with nothing new in the drafts must not re-render every field that reads them.
  it('keeps its consumers still when the provider re-renders without a change', () => {
    let renders = 0;
    function Consumer() {
      useDraftGuard();
      renders += 1;
      return null;
    }
    const content = <Consumer />;
    function Host() {
      return <DraftProvider>{content}</DraftProvider>;
    }
    const { rerender } = render(<Host />);
    const before = renders;
    rerender(<Host />);
    expect(renders).toBe(before);
  });
});
