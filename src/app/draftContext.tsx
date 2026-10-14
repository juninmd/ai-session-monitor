// Only field IDs and booleans cross this registry. Draft text and keys stay in their forms.
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react';
import { PendingChanges } from '../components/shell/PendingChanges';

interface Entry {
  id: string;
  dirty: boolean;
  pending: boolean;
}
interface Prompt {
  action: () => void;
  section?: string;
  origin: HTMLElement | null;
}
interface DraftApi {
  entries: readonly Entry[];
  locked: boolean;
  set: (token: symbol, entry: Entry | null) => void;
  request: (action: () => void, section?: string) => void;
  blocked: () => boolean;
  lock: (value: boolean) => void;
}

const DraftContext = createContext<DraftApi | null>(null);
export const useDraftGuard = () => useContext(DraftContext);
const belongs = (entry: Entry, section?: string) =>
  section === undefined || entry.id.startsWith(`${section}-`);

export function DraftProvider({ children }: { children: ReactNode }) {
  const current = useRef(new Map<symbol, Entry>());
  const frozen = useRef(false);
  const [entries, setEntries] = useState<readonly Entry[]>([]);
  const [locked, setLocked] = useState(false);
  const [prompt, setPrompt] = useState<Prompt | null>(null);
  const set = useCallback((token: symbol, entry: Entry | null) => {
    const previous = current.current.get(token);
    if (entry === null && previous === undefined) return;
    if (
      entry &&
      previous?.dirty === entry.dirty &&
      previous.pending === entry.pending &&
      previous.id === entry.id
    )
      return;
    if (entry) current.current.set(token, entry);
    else current.current.delete(token);
    setEntries([...current.current.values()]);
  }, []);
  const blocked = useCallback(
    () =>
      frozen.current || [...current.current.values()].some((entry) => entry.dirty || entry.pending),
    [],
  );
  const lock = useCallback((value: boolean) => {
    frozen.current = value;
    setLocked(value);
  }, []);
  const request = useCallback((action: () => void, section?: string) => {
    if (
      frozen.current ||
      [...current.current.values()].some(
        (entry) => belongs(entry, section) && (entry.dirty || entry.pending),
      )
    ) {
      setPrompt({
        action,
        ...(section === undefined ? {} : { section }),
        origin: document.activeElement instanceof HTMLElement ? document.activeElement : null,
      });
    } else action();
  }, []);
  const cancel = () => {
    setPrompt(null);
    prompt?.origin?.focus();
  };
  const waiting =
    locked || entries.some((entry) => belongs(entry, prompt?.section) && entry.pending);
  const dirty = entries.some((entry) => belongs(entry, prompt?.section) && entry.dirty);
  const discard = () => {
    if (
      !prompt ||
      frozen.current ||
      [...current.current.values()].some((entry) => belongs(entry, prompt.section) && entry.pending)
    )
      return;
    setPrompt(null);
    prompt.action();
    prompt.origin?.focus();
  };
  return (
    <DraftContext value={{ entries, locked, set, request, blocked, lock }}>
      {children}
      {prompt && (
        <PendingChanges
          waiting={waiting}
          dirty={dirty}
          origin={prompt.origin}
          onCancel={cancel}
          onDiscard={discard}
        />
      )}
    </DraftContext>
  );
}

export function usePendingDraft(id: string, dirty: boolean, pending: boolean) {
  const registry = useDraftGuard();
  const token = useRef(Symbol('draft'));
  const set = registry?.set;
  useEffect(() => {
    set?.(token.current, { id, dirty, pending });
  }, [set, id, dirty, pending]);
  useEffect(() => {
    const key = token.current;
    return () => set?.(key, null);
  }, [set]);
}
