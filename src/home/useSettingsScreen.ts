// The settings screen as the flyout wires it: whether the screen is up, which section covers it,
// the guard on leaving either, and the keyboard that goes back to whatever opened the section.
// App owns the pieces around it (the detail, the body's scroll position) and passes them in.
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { SettingsSection } from '../app/harness';
import { SETTINGS_TITLE_ID } from '../app/homeIds';
import type { Navigation } from '../app/navigation';
import type { Screen } from './Fly';

export interface SettingsScreenApi {
  screen: Screen;
  /** The settings screen is what the body shows. */
  inSettings: boolean;
  /** One section is over the screen, and leaves before the screen itself does. */
  sectionOpen: boolean;
  openSettings: (section: SettingsSection | null) => void;
  closeSettings: () => void;
  /** The menu's own arrow: leaving the screen is guarded like any other navigation. */
  requestCloseSettings: () => void;
  toggleSettings: () => void;
  /** Opens a section over the screen, or closes the open one with null. */
  onSection: (next: SettingsSection | null) => void;
  /** Leaves the open section, back to the menu. Unguarded: Escape guards it. */
  closeSection: () => void;
  navigation: Navigation;
}

export function useSettingsScreen(input: {
  /** Runs an action, asking first when a form in it holds unsaved edits. */
  guarded: (action: () => void) => void;
  /** The provider detail gives way to the settings screen; the two are never open together. */
  hideDetail: () => void;
  /** A new screen starts at the top of the body. */
  toTop: () => void;
}): SettingsScreenApi {
  const { guarded, hideDetail, toTop } = input;
  const [screen, setScreen] = useState<Screen>({ kind: 'surface' });
  const sectionOpener = useRef<Element | null>(null);
  const inSettings = screen.kind === 'settings';
  const sectionOpen = screen.kind === 'settings' && screen.section !== null;

  const closeSettings = useCallback(() => {
    sectionOpener.current = null;
    setScreen({ kind: 'surface' });
    toTop();
  }, [toTop]);
  const openSettings = useCallback(
    (section: SettingsSection | null) => {
      const open = () => {
        hideDetail();
        // A section opened straight from a deep link still needs to know where the keyboard was.
        if (section !== null) sectionOpener.current = document.activeElement;
        setScreen({ kind: 'settings', section });
        toTop();
      };
      if (screen.kind === 'surface') guarded(open);
      else open();
    },
    [guarded, hideDetail, screen.kind, toTop],
  );
  // Leaving a section is guarded like any other navigation. The keyboard goes back to whatever
  // opened it — the menu row, or the menu's own heading when the opener went away with the panel it
  // lived in — once the menu is out of its inert state again, which is why the restore waits for
  // the commit instead of running inside the click.
  const closeSection = useCallback(() => {
    setScreen((current) =>
      current.kind === 'settings' ? { kind: 'settings', section: null } : current,
    );
  }, []);
  const openSection = useCallback((next: SettingsSection) => {
    sectionOpener.current = document.activeElement;
    setScreen({ kind: 'settings', section: next });
  }, []);
  const onSection = useCallback(
    (next: SettingsSection | null) => {
      if (next === null) guarded(closeSection);
      else openSection(next);
    },
    [guarded, closeSection, openSection],
  );
  const hadSection = useRef(false);
  useEffect(() => {
    if (hadSection.current && !sectionOpen) {
      const opener = sectionOpener.current;
      if (opener instanceof HTMLElement && opener.isConnected) opener.focus();
      else document.getElementById(SETTINGS_TITLE_ID)?.focus();
    }
    hadSection.current = sectionOpen;
  }, [sectionOpen]);
  const toggleSettings = useCallback(() => {
    hideDetail();
    if (screen.kind === 'settings') guarded(closeSettings);
    else openSettings(null);
  }, [guarded, closeSettings, hideDetail, openSettings, screen.kind]);
  const requestCloseSettings = useCallback(() => guarded(closeSettings), [guarded, closeSettings]);
  const navigation = useMemo(
    () => ({ openSettings: (section?: SettingsSection) => openSettings(section ?? null) }),
    [openSettings],
  );
  return {
    screen,
    inSettings,
    sectionOpen,
    openSettings,
    closeSettings,
    requestCloseSettings,
    toggleSettings,
    onSection,
    closeSection,
    navigation,
  };
}
