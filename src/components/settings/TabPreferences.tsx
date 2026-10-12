import { useEffect, useRef } from 'react';
import { harnessName, TAB_ORDER } from '../../app/harness';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { withVisibleTab, withVisibleTabs } from '../../app/visibleTabs';
import { t } from '../../i18n';
import type { Settings } from '../../lib/contract';
import { HarnessLogo } from '../brand/HarnessLogo';
import { ActionError } from '../panel/ErrorActions';

export function TabPreferences() {
  const { settings, save } = useSettingsApi();
  const action = useAsyncAction((patch: (current: Settings) => Settings) => save(patch));
  const visible = settings.general.visibleTabs;
  const restoreFocus = useRef<HTMLElement | null>(null);
  useEffect(() => {
    if (action.pending || restoreFocus.current === null) return;
    const target = restoreFocus.current;
    restoreFocus.current = null;
    if (document.activeElement !== document.body && document.activeElement !== target) return;
    if (target.matches(':disabled')) document.getElementById('sg-tabs')?.focus();
    else target.focus();
  }, [action.pending]);
  return (
    <section className="sg tab-preferences" aria-labelledby="sg-tabs">
      <div className="preference-heading">
        <h3 id="sg-tabs" tabIndex={-1}>
          {t('settings.tabs')}
        </h3>
        <span className="preference-count">
          {t('settings.tabs_count', { count: visible.length, total: TAB_ORDER.length })}
        </span>
      </div>
      <p className="hint" id="tabs-hint">
        {t('settings.tabs_hint')}
      </p>
      <fieldset className="provider-picker" disabled={action.pending} aria-describedby="tabs-hint">
        <legend className="vh">{t('settings.tabs')}</legend>
        <div className="provider-grid">
          {TAB_ORDER.map((id) => (
            <label className="provider-option" key={id}>
              <HarnessLogo id={id} size={20} />
              <span>{harnessName(id)}</span>
              <input
                type="checkbox"
                checked={visible.includes(id)}
                onChange={(event) => {
                  const checked = event.currentTarget.checked;
                  restoreFocus.current = event.currentTarget;
                  void action.run((current) => withVisibleTab(current, id, checked));
                }}
              />
            </label>
          ))}
        </div>
        <div className="preference-actions">
          <button
            type="button"
            className="link-btn"
            disabled={visible.length === TAB_ORDER.length}
            onClick={(event) => {
              restoreFocus.current = event.currentTarget;
              void action.run((current) => withVisibleTabs(current, TAB_ORDER));
            }}
          >
            {t('settings.tabs_all')}
          </button>
          <button
            type="button"
            className="link-btn"
            disabled={visible.length === 0}
            onClick={(event) => {
              restoreFocus.current = event.currentTarget;
              void action.run((current) => withVisibleTabs(current, []));
            }}
          >
            {t('settings.tabs_none')}
          </button>
        </div>
      </fieldset>
      <p className="hint" role="status">
        {action.pending ? t('settings.tabs_saving') : t('settings.tabs_autosave')}
      </p>
      {action.error && <ActionError error={action.error} />}
    </section>
  );
}
