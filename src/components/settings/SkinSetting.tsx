import { useState } from 'react';
import { applySkin, loadSkin, SKINS, type SkinId } from '../../app/skin';
import { t } from '../../i18n';
import { SelectRow } from '../forms/fields';

const OPTIONS = SKINS.map((skin) => ({ value: skin.id, label: skin.label }));

/** The colour theme of the flyout. Applies at once and is remembered by this webview only. */
export function SkinSetting() {
  const [skin, setSkin] = useState<SkinId>(loadSkin);
  return (
    <SelectRow
      id="set-skin"
      label={t('settings.skin')}
      hint={t('settings.skin_hint')}
      value={skin}
      options={OPTIONS}
      onChange={(next) => {
        setSkin(next);
        applySkin(next);
      }}
    />
  );
}
