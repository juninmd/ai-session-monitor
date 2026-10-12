import { Component, type ErrorInfo, type ReactNode } from 'react';
import { t } from '../../i18n';
import { Icon } from '../icons';

interface State {
  failed: boolean;
}

/** A render bug must not leave a blank flyout: say so and offer a way back. */
export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  override state: State = { failed: false };

  static getDerivedStateFromError(): State {
    return { failed: true };
  }

  override componentDidCatch(_error: Error, info: ErrorInfo): void {
    // Only the component stack is logged: it names components, never data or settings.
    console.error('render failed', info.componentStack);
  }

  override render(): ReactNode {
    if (!this.state.failed) return this.props.children;
    return (
      <div className="flyout">
        <div className="fly-body">
          <section className="card notice" data-status="crit" role="alert">
            <span className="badge">
              <Icon name="warn" />
            </span>
            <h2>{t('boundary.title')}</h2>
            <p>{t('boundary.body')}</p>
            <div className="actions">
              <button
                type="button"
                className="btn primary"
                onClick={() => this.setState({ failed: false })}
              >
                {t('boundary.retry')}
              </button>
            </div>
          </section>
        </div>
      </div>
    );
  }
}
