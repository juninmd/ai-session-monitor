// One settings section, over the whole flyout: the same lateral layer as the provider detail, but
// wide enough that the settings rows keep their full width behind a 20 px reveal of the menu.
// The menu stays mounted under the scrim, so the expansion of a source group survives closing and
// reopening the section; leaving the screen throws that state away with the component.
import { useEffect, useRef, useState } from 'react';
import { useDraftGuard } from '../app/draftContext';
import type { SettingsSection } from '../app/harness';
import { SETTINGS_TITLE_ID } from '../app/homeIds';
import { useSettingsApi } from '../app/settingsContext';
import { GeneralSettings } from '../components/settings/GeneralSettings';
import { HistorySettings } from '../components/settings/HistorySettings';
import { LEVEL_TITLE, paneOf, sourceOf } from '../components/settings/levels';
import { SettingsBody } from '../components/settings/SettingsBody';
import { SourcesSection } from '../components/settings/SourcesSection';
import { UpdateSettings } from '../components/settings/UpdateSettings';
import { t } from '../i18n';
import type { HarnessId } from '../lib/contract';
import { DrawerShell } from './DrawerShell';

interface SettingsDrawerProps {
  /** The section to show; null renders nothing but keeps this component and its state alive. */
  section: SettingsSection | null;
  /** Closes the drawer, or swaps it for another section. App guards leaving one. */
  onSection: (next: SettingsSection | null) => void;
}

export function SettingsDrawer({ section, onSection }: SettingsDrawerProps) {
  const api = useSettingsApi();
  const request = useDraftGuard()?.request;
  const rootRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState<ReadonlySet<HarnessId>>(() => {
    const source = sourceOf(section);
    return new Set(source === null ? [] : [source]);
  });
  // The source whose group takes the focus: the one the deep link named, and no other. A section
  // opened from the menu names none, so the drawer's own controls keep the keyboard.
  const source = sourceOf(section);

  // Where the focus goes once the settings are in: the deep-linked group, else nothing (the shell
  // has already put the keyboard on the close button). Idempotent, so development's second render
  // pass lands in the same place.
  useEffect(() => {
    const group =
      !api.loaded || source === null
        ? null
        : rootRef.current?.querySelector<HTMLElement>(`#grp-${source}`);
    if (!group) return;
    group.scrollIntoView({ block: 'nearest' });
    group.querySelector('summary')?.focus();
  }, [api.loaded, source]);

  // A new section arriving from outside opens its group, the way asking for it did.
  useEffect(() => {
    if (source === null) return;
    setOpen((current) => (current.has(source) ? current : new Set([...current, source])));
  }, [source]);

  const toggle = (harness: HarnessId, isOpen: boolean) => {
    const apply = () =>
      setOpen((current) => {
        if (current.has(harness) === isOpen) return current;
        const next = new Set(current);
        if (isOpen) next.add(harness);
        else next.delete(harness);
        return next;
      });
    if (!isOpen && request) request(apply, harness);
    else apply();
  };

  if (section === null) return null;
  const pane = paneOf(section);
  return (
    <DrawerShell
      titleId={SETTINGS_TITLE_ID}
      title={t(LEVEL_TITLE[pane])}
      titleFocusable
      closeLabel={t('settings.back')}
      size="wide"
      onClose={() => onSection(null)}
    >
      <div ref={rootRef}>
        <SettingsBody>
          {pane === 'general' && <GeneralSettings />}
          {pane === 'history' && <HistorySettings enabled={api.settings.general.historyEnabled} />}
          {pane === 'update' && <UpdateSettings />}
          {pane === 'sources' && (
            <SourcesSection
              open={open}
              onToggle={toggle}
              focusHeading={() => document.getElementById(SETTINGS_TITLE_ID)?.focus()}
            />
          )}
        </SettingsBody>
      </div>
    </DrawerShell>
  );
}
