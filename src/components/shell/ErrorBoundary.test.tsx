import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ErrorBoundary } from './ErrorBoundary';

const SECRET = 'fab-secret-in-state';

function Bomb({ armed }: { armed: { current: boolean } }) {
  if (armed.current) throw new Error(`render failed with ${SECRET}`);
  return <p>all good</p>;
}

const quiet = () => vi.spyOn(console, 'error').mockImplementation(() => {});

describe('the error boundary', () => {
  it('renders its children when nothing is wrong', () => {
    render(
      <ErrorBoundary>
        <p>all good</p>
      </ErrorBoundary>,
    );
    expect(screen.getByText('all good')).toBeVisible();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('replaces a crashed screen with words and one way back, never a blank flyout', () => {
    quiet();
    render(
      <ErrorBoundary>
        <Bomb armed={{ current: true }} />
      </ErrorBoundary>,
    );
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('A interface falhou');
    expect(alert).toHaveTextContent('Seus dados não foram alterados');
    expect(screen.getAllByRole('button')).toHaveLength(1);
  });

  it('keeps the error text and the data off the screen and out of the log', () => {
    const log = quiet();
    render(
      <ErrorBoundary>
        <Bomb armed={{ current: true }} />
      </ErrorBoundary>,
    );
    expect(document.body).not.toHaveTextContent(SECRET);
    const ours = log.mock.calls.filter(([first]) => first === 'render failed');
    expect(ours).toHaveLength(1);
    expect(JSON.stringify(ours)).not.toContain(SECRET);
    expect(JSON.stringify(ours)).toContain('Bomb');
  });

  it('draws the children again on request, and stays up if they are fixed', async () => {
    quiet();
    const armed = { current: true };
    const user = userEvent.setup();
    render(
      <ErrorBoundary>
        <Bomb armed={armed} />
      </ErrorBoundary>,
    );
    armed.current = false;
    await user.click(screen.getByRole('button', { name: 'Recarregar a tela' }));
    expect(screen.getByText('all good')).toBeVisible();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('shows the message again when the retry crashes as well', async () => {
    quiet();
    const user = userEvent.setup();
    render(
      <ErrorBoundary>
        <Bomb armed={{ current: true }} />
      </ErrorBoundary>,
    );
    await user.click(screen.getByRole('button', { name: 'Recarregar a tela' }));
    expect(screen.getByRole('alert')).toHaveTextContent('A interface falhou');
  });
});
