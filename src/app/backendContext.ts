import { createContext, useContext } from 'react';
import type { Backend } from '../lib/backend';

export const BackendContext = createContext<Backend | null>(null);

export function useBackend(): Backend {
  const backend = useContext(BackendContext);
  if (backend === null) throw new Error('BackendContext.Provider is missing');
  return backend;
}
