// Pieces shared by the OpenCode connect card and its settings group.

import { useBackend } from '../../app/backendContext';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import type { ExternalLink } from '../../lib/contract';
import { SecretField } from '../forms/SecretField';
import { SettingSwitch } from '../forms/SettingControls';
import { Icon } from '../icons';
import { ActionError } from '../panel/ErrorActions';

export function OpencodeKey() {
  const { secrets, setSecret, clearSecret } = useSettingsApi();
  return (
    <SecretField
      id="opencode-key"
      label={t('opencode.key_label')}
      saved={secrets.opencode}
      onSave={(value) => setSecret('opencode', value)}
      onClear={() => clearSecret('opencode')}
    />
  );
}

export function OpencodeLocalSwitch() {
  const { settings } = useSettingsApi();
  return (
    <SettingSwitch
      id="opencode-local"
      label={t('opencode.local_label')}
      hint={t('opencode.local_hint')}
      checked={settings.opencode.localConsent}
      apply={(current, localConsent) => ({
        ...current,
        opencode: { ...current.opencode, localConsent },
      })}
    />
  );
}

/** Buttons that open fixed destinations; the backend maps each name to its URL. */
export function OpencodeLinks({
  showConsole = true,
  showBilling = false,
}: {
  showConsole?: boolean;
  showBilling?: boolean;
}) {
  const backend = useBackend();
  const open = useAsyncAction((link: ExternalLink) => backend.openLink(link));
  return (
    <>
      <div className="actions">
        {showConsole && (
          <button
            type="button"
            className="btn"
            onClick={() => void open.run('opencode_go_console')}
          >
            {t('opencode.console')}
            <Icon name="external" />
          </button>
        )}
        {showBilling && (
          <button type="button" className="btn" onClick={() => void open.run('opencode_billing')}>
            {t('opencode.billing')}
            <Icon name="external" />
          </button>
        )}
      </div>
      {open.error && <ActionError error={open.error} harness="opencode" />}
    </>
  );
}
