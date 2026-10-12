import { useState } from 'react';
import { fieldOf } from '../../app/errorText';
import { fieldMessage } from '../../app/fieldErrors';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import type { Settings } from '../../lib/contract';
import { SwitchRow } from '../forms/fields';
import { PathField } from '../forms/PathField';
import { Icon } from '../icons';
import { Banner } from '../panel/Banner';
import { ShimControls } from '../setup/ShimControls';
import { SourceSwitch } from './SourceSwitch';

const withOauth = (settings: Settings, oauthUsage: boolean): Settings => ({
  ...settings,
  claude: { ...settings.claude, oauthUsage },
});

const OAUTH_SWITCH = 'claude-oauth';
// Both buttons of the warning go away with it; the switch is what stays mounted.
const backToSwitch = () => document.getElementById(OAUTH_SWITCH)?.focus();

/** Turning the unofficial usage API on takes two steps: the switch, then an explicit confirmation. */
function OauthOptIn() {
  const { settings, save } = useSettingsApi();
  const enabled = settings.claude.oauthUsage;
  const [confirming, setConfirming] = useState(false);
  const change = useAsyncAction(async (value: boolean) => {
    await save((current) => withOauth(current, value));
    setConfirming(false);
  });
  const error = change.error ? fieldMessage(change.error, fieldOf(change.error)) : null;

  return (
    <>
      <SwitchRow
        id={OAUTH_SWITCH}
        label={t('claude.oauth.label')}
        hint={t('claude.oauth.hint')}
        checked={enabled}
        error={error}
        badge={
          <span className="chip warnchip">
            <Icon name="warn" />
            {t('settings.unofficial_chip')}
          </span>
        }
        onChange={(value) => {
          if (value) setConfirming(true);
          else void change.run(false);
        }}
      />
      {confirming && !enabled && (
        <div role="alert">
          <Banner tone="warn">
            <p>{t('claude.oauth.warning')}</p>
            <div className="actions">
              <button
                type="button"
                className="btn primary"
                disabled={change.pending}
                onClick={() => {
                  backToSwitch();
                  void change.run(true);
                }}
              >
                {t('claude.oauth.confirm')}
              </button>
              <button
                type="button"
                className="btn"
                onClick={() => {
                  backToSwitch();
                  setConfirming(false);
                }}
              >
                {t('claude.oauth.cancel')}
              </button>
            </div>
          </Banner>
        </div>
      )}
    </>
  );
}

export function ClaudeGroup() {
  const { settings } = useSettingsApi();
  return (
    <>
      <SourceSwitch harness="claude" />
      {/* Another folder is another settings.json: remounting makes the block look at it again. */}
      <ShimControls key={settings.claude.configDir ?? ''} />
      <PathField
        id="claude-config-dir"
        label={t('claude.config_dir')}
        hint={t('claude.config_dir_hint')}
        placeholder={t('claude.config_dir_placeholder')}
        saveLabel={t('claude.config_dir_save')}
        saved={settings.claude.configDir}
        apply={(current, configDir) => ({ ...current, claude: { ...current.claude, configDir } })}
      />
      <OauthOptIn />
    </>
  );
}
