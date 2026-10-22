import { useSettingsApi } from '../../app/settingsContext';
import { t } from '../../i18n';
import { LOCALES } from '../../i18n/locale';
import type { Language } from '../../lib/contract';
import type { Option } from '../forms/fields';
import { SettingSelect } from '../forms/SettingControls';

// Each language is named in itself, in every UI language: the picker must stay findable.
const OPTIONS: readonly Option<Language>[] = LOCALES.map((locale) => ({
  value: locale.id,
  label: locale.name,
}));

/** The interface language. Saved with the settings, so the desktop toasts follow it too. */
export function LanguageSetting() {
  const { settings } = useSettingsApi();
  return (
    <SettingSelect
      id="set-language"
      label={t('settings.language')}
      hint={t('settings.language_hint')}
      value={settings.general.language}
      options={OPTIONS}
      apply={(current, language) => ({
        ...current,
        general: { ...current.general, language },
      })}
    />
  );
}
