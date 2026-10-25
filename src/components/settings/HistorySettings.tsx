import { useState } from 'react';
import { useBackend } from '../../app/backendContext';
import { describeError } from '../../app/errorText';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import { SettingSwitch } from '../forms/SettingControls';
import { Banner } from '../panel/Banner';

const CLEAR_BUTTON = 'history-clear';
const CONFIRM_REGION = 'history-clear-confirm';

/** The switch for the stored quota history and the two-step button that erases it. */
export function HistorySettings({ enabled }: { enabled: boolean }) {
  const backend = useBackend();
  const [confirming, setConfirming] = useState(false);
  const [cleared, setCleared] = useState(false);
  const clear = useAsyncAction(() => backend.clearHistory());
  const error = clear.error ? describeError(clear.error) : null;
  const backToButton = () => document.getElementById(CLEAR_BUTTON)?.focus();

  return (
    <section className="sg">
      <SettingSwitch
        id="set-history"
        label={t('settings.history_enabled')}
        hint={t('settings.history_hint')}
        checked={enabled}
        apply={(current, historyEnabled) => ({
          ...current,
          general: { ...current.general, historyEnabled },
        })}
      />
      <div className="actions">
        <button
          id={CLEAR_BUTTON}
          type="button"
          className="btn"
          aria-expanded={confirming}
          aria-controls={confirming ? CONFIRM_REGION : undefined}
          onClick={() => {
            setCleared(false);
            setConfirming(true);
          }}
        >
          {t('settings.history_clear')}
        </button>
      </div>
      {confirming && (
        <div id={CONFIRM_REGION} role="alert">
          <Banner tone="warn">
            <p>{t('settings.history_clear_confirm')}</p>
            <div className="actions">
              <button
                type="button"
                className="btn primary"
                disabled={clear.pending}
                onClick={async () => {
                  const done = await clear.run();
                  setConfirming(false);
                  setCleared(done);
                  backToButton();
                }}
              >
                {t('settings.history_clear_yes')}
              </button>
              <button
                type="button"
                className="btn"
                onClick={() => {
                  setConfirming(false);
                  backToButton();
                }}
              >
                {t('settings.history_clear_no')}
              </button>
            </div>
          </Banner>
        </div>
      )}
      {cleared && (
        <p className="hint" role="status">
          {t('settings.history_cleared')}
        </p>
      )}
      {error && (
        <p className="hint" role="alert">
          {error.title}. {error.body}
        </p>
      )}
    </section>
  );
}
