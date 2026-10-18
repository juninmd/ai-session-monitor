// The settings screen inside the flyout: the menu of sections, and nothing else. A section opens as
// a lateral drawer over the whole flyout, which App owns next to the provider detail, so this file
// stays the short menu — the rows report a section's state and ask App to open it.
import { useEffect, useRef } from 'react';
import type { SettingsSection } from '../../app/harness';
import { SETTINGS_TITLE_ID } from '../../app/homeIds';
import { useSettingsApi } from '../../app/settingsContext';
import { t } from '../../i18n';
import { Icon } from '../icons';
import { SettingsBody } from './SettingsBody';
import { SettingsMenu } from './SettingsMenu';

interface SettingsViewProps {
  /** The section the drawer is showing; the menu is what stays under it. */
  section: SettingsSection | null;
  onSection: (next: SettingsSection | null) => void;
  /** Leaves the whole screen; App guards it like any other navigation. */
  onCloseSettings: () => void;
}

export function SettingsView({ section, onSection, onCloseSettings }: SettingsViewProps) {
  const api = useSettingsApi();
  const headingRef = useRef<HTMLHeadingElement>(null);
  // Covered by a section drawer, the menu is scenery behind the scrim: it keeps its heading but not
  // the id — the drawer's own heading carries `settings-title` while a section is open — and it
  // stays out of the accessibility tree and out of the keyboard order.
  const covered = section !== null;
  // The heading is the shell's landing spot when the screen opens on the menu. Mount only: coming
  // back from a section hands the keyboard to the row that opened it instead.
  useEffect(() => {
    headingRef.current?.focus();
  }, []);
  return (
    <div className="settings" aria-hidden={covered || undefined} inert={covered || undefined}>
      <div className="set-head">
        <button
          type="button"
          className="icon-btn"
          aria-label={t('settings.back')}
          onClick={onCloseSettings}
        >
          <Icon name="back" />
        </button>
        <h2 id={covered ? undefined : SETTINGS_TITLE_ID} ref={headingRef} tabIndex={-1}>
          {t('settings.title')}
        </h2>
      </div>
      {/* The drawer prints the failure and the loading state while it covers the menu, so the menu
          does not repeat them behind the scrim. */}
      {covered && !api.loaded ? null : (
        <SettingsBody>
          <SettingsMenu onOpen={onSection} />
        </SettingsBody>
      )}
    </div>
  );
}
