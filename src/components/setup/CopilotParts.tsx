// Pieces shared by the GitHub Copilot connect card and its settings group: an optional pasted
// token, or the consent to read the one the Copilot extension keeps on disk.
import { useSettingsApi } from '../../app/settingsContext';
import { t } from '../../i18n';
import { SecretField } from '../forms/SecretField';
import { SettingSwitch } from '../forms/SettingControls';
import { LinkButton } from './LinkButton';

export function CopilotKey() {
  const { secrets, setSecret, clearSecret } = useSettingsApi();
  return (
    <SecretField
      id="copilot-key"
      label={t('copilot.key_label')}
      saved={secrets.copilot}
      onSave={(value) => setSecret('copilot', value)}
      onClear={() => clearSecret('copilot')}
    />
  );
}

export function CopilotLocalSwitch() {
  const { settings } = useSettingsApi();
  return (
    <SettingSwitch
      id="copilot-local"
      label={t('copilot.local_label')}
      hint={t('copilot.local_hint')}
      checked={settings.copilot.localConsent}
      apply={(current, localConsent) => ({
        ...current,
        copilot: { ...current.copilot, localConsent },
      })}
    />
  );
}

export function CopilotLinks() {
  return <LinkButton link="copilot_settings" label={t('copilot.console')} harness="copilot" />;
}
