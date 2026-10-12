import { useSettingsApi } from '../../app/settingsContext';
import { isEnabled, withEnabled } from '../../app/settingsPatch';
import { t } from '../../i18n';
import type { HarnessId } from '../../lib/contract';
import { SettingSwitch } from '../forms/SettingControls';

/** The on/off switch at the top of every source group. */
export function SourceSwitch({ harness }: { harness: HarnessId }) {
  const { settings } = useSettingsApi();
  return (
    <SettingSwitch
      id={`src-${harness}`}
      label={t('settings.enabled')}
      checked={isEnabled(settings, harness)}
      apply={(current, value) => withEnabled(current, harness, value)}
    />
  );
}
