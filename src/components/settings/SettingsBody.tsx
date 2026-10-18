// What a settings screen shows before its own content: why the stored file could not be read, or
// that it is still being read. The fieldset freezes every control while a draft is in flight.
import type { ReactNode } from 'react';
import { useDraftGuard } from '../../app/draftContext';
import { describeError } from '../../app/errorText';
import { useSettingsApi } from '../../app/settingsContext';
import { t } from '../../i18n';
import { Icon } from '../icons';

export function SettingsBody({ children }: { children: ReactNode }) {
  const api = useSettingsApi();
  const guard = useDraftGuard();
  const failure = api.loadError ? describeError(api.loadError) : null;
  if (failure) {
    return (
      <section className="card notice" data-status="crit">
        <span className="badge">
          <Icon name="warn" />
        </span>
        <h3>{failure.title}</h3>
        <p>{failure.body}</p>
        <div className="actions">
          <button type="button" className="btn primary" onClick={api.reload}>
            {t('action.retry')}
          </button>
        </div>
      </section>
    );
  }
  if (!api.loaded) return <output className="hint">{t('settings.loading')}</output>;
  return (
    <fieldset className="settings-controls" disabled={guard?.locked}>
      {children}
    </fieldset>
  );
}
