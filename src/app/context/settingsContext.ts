import { createContext, useContext } from 'react';
import type { SettingsApi } from '../hooks/useSettingsState';

export const SettingsContext = createContext<SettingsApi | null>(null);

export function useSettingsApi(): SettingsApi {
  const api = useContext(SettingsContext);
  if (api === null) throw new Error('SettingsContext.Provider is missing');
  return api;
}
