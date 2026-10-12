import { createContext, useContext } from 'react';
import type { SettingsSection } from './harness';

export interface Navigation {
  /** Opens the settings view, optionally on one group (an error action lands there). */
  openSettings: (section?: SettingsSection) => void;
}

export const NavigationContext = createContext<Navigation>({ openSettings: () => {} });

export function useNavigation(): Navigation {
  return useContext(NavigationContext);
}
