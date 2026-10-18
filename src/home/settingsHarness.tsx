// The settings screen the way the flyout composes it: the menu in the scrolling body, the section
// drawer over the frame. Panel tests drive the same two siblings App renders, without the shell.
import { useEffect, useState } from 'react';
import type { SettingsSection } from '../app/harness';
import { SettingsView } from '../components/settings/SettingsView';
import { SettingsDrawer } from './SettingsDrawer';

export interface SettingsScreenProps {
  initial?: SettingsSection | null;
  onCloseSettings?: () => void;
  /** Handed the state setter, so a case can steer a section in from outside. */
  steer?: { current: (next: SettingsSection | null) => void };
}

export function SettingsScreen({ initial = null, onCloseSettings, steer }: SettingsScreenProps) {
  const [section, setSection] = useState<SettingsSection | null>(initial);
  useEffect(() => {
    if (steer) steer.current = setSection;
  });
  return (
    <>
      <SettingsView
        section={section}
        onSection={setSection}
        onCloseSettings={onCloseSettings ?? (() => {})}
      />
      <SettingsDrawer section={section} onSection={setSection} />
    </>
  );
}
